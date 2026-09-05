//! Model Context Protocol (MCP) native implementation.
//!
//! Exposes endpoints and data structures conforming to the Anthropic MCP specification
//! (JSON-RPC 2.0 and REST modes) enabling Claude Desktop, Cursor, Windsurf and autonomous agents
//! to discover and execute tools, read resources and inspect server status.

use crate::Result;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::Arc;

/// Type de fonction d'exécution d'un outil MCP.
pub type McpToolFn = Arc<dyn Fn(Value) -> Result<Value> + Send + Sync>;

/// Outil exposé via le protocole MCP.
#[derive(Clone)]
pub struct McpTool {
    /// Nom unique de l'outil (ex: `"fetch_weather"`, `"execute_sql"`).
    pub name: String,
    /// Description claire destinée au LLM / Agent.
    pub description: String,
    /// Schéma JSON décrivant les arguments d'entrée (`inputSchema`).
    pub input_schema: Value,
    /// Fonction de traitement exécutée lors de l'appel `tools/call`.
    pub handler: McpToolFn,
}

impl std::fmt::Debug for McpTool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("McpTool")
            .field("name", &self.name)
            .field("description", &self.description)
            .field("input_schema", &self.input_schema)
            .finish()
    }
}

impl McpTool {
    /// Crée un nouvel outil MCP.
    pub fn new(
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
        handler: impl Fn(Value) -> Result<Value> + Send + Sync + 'static,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema,
            handler: Arc::new(handler),
        }
    }

    /// Exporte la définition de l'outil au format MCP `Tool`.
    pub fn to_mcp_json(&self) -> Value {
        json!({
            "name": self.name,
            "description": self.description,
            "inputSchema": self.input_schema,
        })
    }
}

/// Ressource exposée via MCP (`resources/list` et `resources/read`).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct McpResource {
    /// URI de la ressource (ex: `"grio://app/state"`, `"file:///data/report.md"`).
    pub uri: String,
    /// Nom affiché.
    pub name: String,
    /// Description de la ressource.
    pub description: Option<String>,
    /// Type MIME.
    pub mime_type: Option<String>,
}

/// Requête JSON-RPC 2.0 MCP standard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRpcRequest {
    /// Version du protocole (doit être `"2.0"`).
    #[serde(default = "default_jsonrpc_version")]
    pub jsonrpc: String,
    /// Identifiant de la requête (numérique ou chaîne).
    pub id: Option<Value>,
    /// Méthode invoquée (`"initialize"`, `"tools/list"`, `"tools/call"`, etc.).
    pub method: String,
    /// Paramètres optionnels de la méthode.
    #[serde(default)]
    pub params: Option<Value>,
}

fn default_jsonrpc_version() -> String {
    "2.0".to_string()
}

/// Réponse JSON-RPC 2.0 MCP standard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRpcResponse {
    /// Version du protocole.
    pub jsonrpc: String,
    /// Identifiant associé.
    pub id: Option<Value>,
    /// Résultat en cas de succès.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    /// Erreur en cas d'échec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<McpRpcError>,
}

/// Erreur standardisée JSON-RPC 2.0.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpRpcError {
    /// Code d'erreur (-32600 = Invalid Request, -32601 = Method not found, etc.).
    pub code: i32,
    /// Message lisible.
    pub message: String,
    /// Données complémentaires.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

impl McpRpcResponse {
    /// Crée une réponse avec succès.
    pub fn success(id: Option<Value>, result: Value) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: Some(result),
            error: None,
        }
    }

    /// Crée une réponse avec erreur.
    pub fn error(
        id: Option<Value>,
        code: i32,
        message: impl Into<String>,
        data: Option<Value>,
    ) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id,
            result: None,
            error: Some(McpRpcError {
                code,
                message: message.into(),
                data,
            }),
        }
    }
}

/// Traite une requête MCP entrante (JSON-RPC 2.0).
pub fn handle_mcp_request(
    req: McpRpcRequest,
    app_title: &str,
    tools: &[McpTool],
) -> McpRpcResponse {
    let id = req.id.clone();

    match req.method.as_str() {
        // Handshake initial du client MCP (Claude Desktop / Cursor)
        "initialize" => McpRpcResponse::success(
            id,
            json!({
                "protocolVersion": "2024-11-05",
                "capabilities": {
                    "tools": { "listChanged": false },
                    "resources": { "subscribe": false, "listChanged": false },
                    "logging": {}
                },
                "serverInfo": {
                    "name": format!("grio-mcp-{app_title}"),
                    "version": "0.1.0",
                }
            }),
        ),

        // Notification d'initialisation réussie
        "notifications/initialized" => McpRpcResponse::success(id, json!({ "status": "ok" })),

        // Ping de santé
        "ping" => McpRpcResponse::success(id, json!({})),

        // Découverte des outils disponibles
        "tools/list" => {
            let tools_json: Vec<Value> = tools.iter().map(|t| t.to_mcp_json()).collect();
            McpRpcResponse::success(
                id,
                json!({
                    "tools": tools_json
                }),
            )
        }

        // Exécution d'un outil
        "tools/call" => {
            let Some(params) = req.params else {
                return McpRpcResponse::error(id, -32602, "Missing params for tools/call", None);
            };

            let tool_name = params.get("name").and_then(|v| v.as_str()).unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            let Some(tool) = tools.iter().find(|t| t.name == tool_name) else {
                return McpRpcResponse::error(
                    id,
                    -32601,
                    format!("Tool not found: '{tool_name}'"),
                    None,
                );
            };

            match (tool.handler)(arguments) {
                Ok(output) => {
                    let text = if let Value::String(s) = output {
                        s
                    } else {
                        output.to_string()
                    };
                    McpRpcResponse::success(
                        id,
                        json!({
                            "content": [
                                {
                                    "type": "text",
                                    "text": text
                                }
                            ],
                            "isError": false
                        }),
                    )
                }
                Err(e) => McpRpcResponse::success(
                    id,
                    json!({
                        "content": [
                            {
                                "type": "text",
                                "text": format!("Error executing tool '{tool_name}': {e}")
                            }
                        ],
                        "isError": true
                    }),
                ),
            }
        }

        // Liste des ressources
        "resources/list" => McpRpcResponse::success(
            id,
            json!({
                "resources": [
                    {
                        "uri": "grio://app/manifest",
                        "name": "Application Manifest",
                        "description": "grio active component tree and API schema",
                        "mimeType": "application/json"
                    }
                ]
            }),
        ),

        // Ressource non trouvée ou méthode inconnue
        other => McpRpcResponse::error(
            id,
            -32601,
            format!("Method not found or unsupported: '{other}'"),
            None,
        ),
    }
}
