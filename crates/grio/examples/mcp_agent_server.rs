//! Phase 15.1 Demo : Official Model Context Protocol (MCP) Server Endpoint (`/mcp/v1`).
//!
//! Lancez avec :
//! ```bash
//! cargo run --example mcp_agent_server
//! ```
//!
//! Testez avec une requête MCP JSON-RPC 2.0 (ex: Claude Desktop, Cursor ou curl) :
//! ```bash
//! curl -X POST http://localhost:7860/mcp/v1 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":1,"method":"tools/list"}'
//! curl -X POST http://localhost:7860/mcp/v1 -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"query_database","arguments":{"query":"SELECT * FROM users"}}}'
//! ```

use grio::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    App::new("Grio MCP Agent & Tool Server")
        .subtitle("Official Model Context Protocol v2024-11-05 (/mcp/v1) for Claude Desktop, Cursor & AI Agents")
        .theme(Theme::tokyo_night())
        .mcp(true)
        // 1. Outil MCP : Interrogation de Base de Données
        .mcp_tool(
            "query_database",
            "Execute read-only SQL queries on the analytical data warehouse",
            json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "SQL query to run (e.g. SELECT department, AVG(salary) FROM employees GROUP BY department)"
                    }
                },
                "required": ["query"]
            }),
            |args| {
                let q = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                Ok(json!({
                    "status": "success",
                    "rows_matched": 3,
                    "columns": ["department", "avg_salary"],
                    "data": [
                        ["Engineering", 115000],
                        ["AI Research", 142000],
                        ["Product", 98000]
                    ],
                    "query_executed": q
                }))
            },
        )
        // 2. Outil MCP : Météo & Climat
        .mcp_tool(
            "fetch_weather",
            "Fetch real-time weather metrics and temperature for a given city",
            json!({
                "type": "object",
                "properties": {
                    "city": { "type": "string", "description": "Target city name (e.g. Paris, Tokyo, San Francisco)" }
                },
                "required": ["city"]
            }),
            |args| {
                let city = args.get("city").and_then(|v| v.as_str()).unwrap_or("Paris");
                Ok(json!({
                    "city": city,
                    "temperature_celsius": 21.5,
                    "condition": "Sunny",
                    "humidity_percent": 45,
                    "wind_kmh": 12.0
                }))
            },
        )
        // 3. Outil MCP : Générateur de Rapport IA
        .mcp_tool(
            "generate_report",
            "Compile and format a structured technical evaluation report in Markdown",
            json!({
                "type": "object",
                "properties": {
                    "title": { "type": "string", "description": "Report title" },
                    "key_findings": {
                        "type": "array",
                        "items": { "type": "string" },
                        "description": "List of key bullet points"
                    }
                },
                "required": ["title"]
            }),
            |args| {
                let title = args.get("title").and_then(|v| v.as_str()).unwrap_or("System Report");
                let findings = args.get("key_findings").and_then(|v| v.as_array());
                let mut md = format!("# 📋 {}\n\n**Generated via Grio MCP Tool Calling**\n\n### Key Findings:\n", title);
                if let Some(items) = findings {
                    for it in items {
                        md.push_str(&format!("- {}\n", it.as_str().unwrap_or("")));
                    }
                } else {
                    md.push_str("- System healthy and zero errors observed.\n");
                }
                Ok(json!(md))
            },
        )
        // UI Miroir pour inspecter les outils
        .row(|r| {
            r.item(
                HighlightedText::new("mcp_banner")
                    .label("📡 MCP Endpoint Status")
                    .segments(&[
                        ("POST /mcp/v1 ", Some("ENDPOINT")),
                        ("-> Standard JSON-RPC 2.0 for Claude Desktop & Cursor.\n", None),
                        ("GET /mcp/tools ", Some("DISCOVERY")),
                        ("-> Instant JSON Schema inspection.\n", None),
                    ]),
            );
        })
        .row(|r| {
            r.item(
                DataEditor::new("tools_table")
                    .label("🛠️ Registered MCP Tools in this Server")
                    .column("tool_name", "Tool Name", ColumnType::Text)
                    .column("description", "Description", ColumnType::Text)
                    .column("input_params", "Input Parameters", ColumnType::Text)
                    .data(vec![
                        vec![
                            json!("query_database"),
                            json!("Execute read-only SQL queries on data warehouse"),
                            json!("query (string)"),
                        ],
                        vec![
                            json!("fetch_weather"),
                            json!("Fetch real-time weather metrics for a city"),
                            json!("city (string)"),
                        ],
                        vec![
                            json!("generate_report"),
                            json!("Compile structured Markdown reports"),
                            json!("title (string), key_findings (array)"),
                        ],
                    ])
                    .interactive(false),
            );
        })
        .launch("127.0.0.1:7860")
}
