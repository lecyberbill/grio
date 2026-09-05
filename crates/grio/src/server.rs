//! Serveur web : routes statiques (page + assets), WebSocket temps réel et
//! API REST automatique générée depuis l'arbre de composants.
//!
//! Depuis la Phase 2 (temps réel), chaque événement est mis dans une **file
//! d'attente** et exécuté dans l'ordre sur un pool de threads : les handlers
//! peuvent donc rester synchrones (avec `sleep`, etc.) sans bloquer le
//! serveur. Les mises à jour poussées (`set`, `append`, `progress`, `alert`)
//! transitent par un canal vers le broadcast WebSocket, ce qui permet le
//! **streaming**, les **barres de progression** et les **alertes**.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Json, Query, State};
use axum::http::header;
use axum::response::{Html, IntoResponse};
use axum::routing::{get, post};
use axum::Router;
use futures::{SinkExt, StreamExt};
use serde_json::{json, Value};
use tokio::sync::{broadcast, mpsc, oneshot};

use crate::app::{process_event, App};
use crate::components::{Component, Role};
use crate::context::Context;
use crate::events::WireEvent;
use crate::media;
use crate::Result;

const STYLES: &str = include_str!("assets/styles.css");
const APP_JS: &str = concat!(
    "(function () {\n'use strict';\n\n",
    include_str!("assets/js/core.js"),
    "\n\n",
    include_str!("assets/js/forms.js"),
    "\n\n",
    include_str!("assets/js/data.js"),
    "\n\n",
    include_str!("assets/js/media.js"),
    "\n\n",
    include_str!("assets/js/canvas_editor.js"),
    "\n\n",
    include_str!("assets/js/special.js"),
    "\n\n",
    include_str!("assets/js/router.js"),
    "\n\n",
    include_str!("assets/js/i18n.js"),
    "\n\n})();"
);

type SessionMap = HashMap<String, Value>;
type SessionStore = HashMap<String, Arc<Mutex<SessionMap>>>;

/// État partagé du serveur (application + valeurs courantes + bus temps réel).
pub struct AppServer {
    /// Application déclarée par l'utilisateur.
    pub app: Arc<App>,
    /// Dernier instantané des valeurs d'entrée globales (fallback).
    pub values: Arc<Mutex<HashMap<String, Value>>>,
    /// Sessions isolées des clients : session_id -> HashMap<id_composant, valeur>.
    pub sessions: Arc<Mutex<SessionStore>>,
    /// Canal de poussée global / ciblé.
    push_tx: mpsc::UnboundedSender<(Option<String>, Value)>,
    /// Broadcast vers les clients connectés.
    tx: broadcast::Sender<(Option<String>, String)>,
    /// File d'attente d'événements (ordre FIFO).
    job_tx: mpsc::UnboundedSender<Job>,
    /// Job en cours d'exécution (pour l'annulation sur re-déclenchement).
    current: Mutex<Option<(String, String, Arc<AtomicBool>)>>,
    clients: AtomicUsize,
}

/// Événement en attente d'exécution dans la file.
struct Job {
    session_id: Option<String>,
    wire: WireEvent,
    cancel: Arc<AtomicBool>,
    finish: Option<oneshot::Sender<Value>>,
}

/// Démarre l'écoute HTTP sur `addr` (ex. `"127.0.0.1:7860"`). Bloque.
pub async fn serve(app: App, addr: String) -> Result<()> {
    let (tx, _rx) = broadcast::channel::<(Option<String>, String)>(256);
    let (push_tx, push_rx) = mpsc::unbounded_channel::<(Option<String>, Value)>();
    let (job_tx, job_rx) = mpsc::unbounded_channel::<Job>();

    let server = Arc::new(AppServer {
        app: Arc::new(app),
        values: Arc::new(Mutex::new(HashMap::new())),
        sessions: Arc::new(Mutex::new(HashMap::new())),
        push_tx,
        tx: tx.clone(),
        job_tx,
        current: Mutex::new(None),
        clients: AtomicUsize::new(0),
    });

    tokio::spawn(forwarder(push_rx, tx));
    tokio::spawn(dispatcher(Arc::clone(&server), job_rx));

    let mut router = Router::new()
        .route("/", get(index))
        .route("/ws", get(ws_upgrade))
        .route("/assets/app.js", get(js_asset))
        .route("/assets/styles.css", get(css_asset))
        .route("/api/predict", post(predict))
        .route("/api/schema", get(schema))
        .route("/api/openapi.json", get(openapi_spec))
        .route("/docs", get(docs_page))
        .route("/api/explore", get(explore));

    if server.app.enable_mcp {
        router = router
            .route("/mcp/v1", post(mcp_endpoint).get(mcp_discovery))
            .route("/mcp/tools", get(mcp_tools_list));
    }

    // Register all declared multi-page routes for SPA deep-linking
    for page in &server.app.pages {
        if page.route != "/" {
            router = router.route(&page.route, get(index));
        }
    }

    if server.app.allow_cors {
        router = router.layer(axum::middleware::from_fn(cors_middleware));
    }

    let router = router.with_state(Arc::clone(&server));

    let listener = tokio::net::TcpListener::bind(&addr).await?;

    let io = collect_io(&server.app);
    let host = if let Some(port) = addr.strip_prefix("0.0.0.0:") {
        format!("localhost:{port}")
    } else {
        addr.clone()
    };

    eprintln!();
    eprintln!("  +----------------------------------------------");
    eprintln!("  |  grio - {}", server.app.title);
    if !server.app.subtitle.is_empty() {
        eprintln!("  |  {}", server.app.subtitle);
    }
    eprintln!("  +----------------------------------------------");
    eprintln!("  |  UI   ->  http://{host}");
    eprintln!("  |  API  ->  POST http://{host}/api/predict");
    eprintln!("  |       ->  GET  http://{host}/api/schema");
    if server.app.enable_mcp {
        eprintln!("  |  MCP  ->  POST http://{host}/mcp/v1 (Claude Desktop / Cursor)");
    }
    if server.app.enable_docs {
        eprintln!("  |  Docs ->  GET  http://{host}/docs");
        eprintln!("  |       ->  GET  http://{host}/api/openapi.json");
    }
    if server.app.api_key.is_some() {
        eprintln!("  |  Auth ->  Cle API requise pour /api/predict & /mcp/v1");
    }
    eprintln!("  +----------------------------------------------");
    eprintln!(
        "  |  Entrees [{}] : {}",
        io.inputs.len(),
        io.inputs.join(", ")
    );
    eprintln!(
        "  |  Sorties [{}] : {}",
        io.outputs.len(),
        io.outputs.join(", ")
    );
    eprintln!("  |  Listeners  [{}]", server.app.handlers.len());
    for h in &server.app.handlers {
        let src = match (&h.event, &h.component) {
            (crate::events::EventName::Submit, _) => "Run/API".to_string(),
            (crate::events::EventName::Load, _) => "load (page)".to_string(),
            (crate::events::EventName::Change, Some(id)) => format!("change sur `{id}`"),
            (crate::events::EventName::Click, Some(id)) => format!("click sur `{id}`"),
            (crate::events::EventName::Play, Some(id)) => format!("play sur `{id}`"),
            (crate::events::EventName::Pause, Some(id)) => format!("pause sur `{id}`"),
            (crate::events::EventName::Stop, Some(id)) => format!("stop sur `{id}`"),
            (crate::events::EventName::Stream, Some(id)) => format!("stream sur `{id}`"),
            (crate::events::EventName::Custom(n), _) => format!("event `{n}`"),
            _ => "-".to_string(),
        };
        eprintln!("  |    - {src}");
    }
    eprintln!("  +----------------------------------------------");
    eprintln!();

    axum::serve(listener, router).await?;
    Ok(())
}

async fn cors_middleware(
    request: axum::extract::Request,
    next: axum::middleware::Next,
) -> impl IntoResponse {
    let mut response = if request.method() == axum::http::Method::OPTIONS {
        (axum::http::StatusCode::NO_CONTENT, ()).into_response()
    } else {
        next.run(request).await
    };
    let headers = response.headers_mut();
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_ORIGIN,
        header::HeaderValue::from_static("*"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_METHODS,
        header::HeaderValue::from_static("GET, POST, OPTIONS"),
    );
    headers.insert(
        header::ACCESS_CONTROL_ALLOW_HEADERS,
        header::HeaderValue::from_static("Content-Type, Authorization, X-API-Key, X-Session-ID"),
    );
    response
}

/// Relayeur : pousse les messages temps réel vers le broadcast WebSocket.
async fn forwarder(
    mut rx: mpsc::UnboundedReceiver<(Option<String>, Value)>,
    tx: broadcast::Sender<(Option<String>, String)>,
) {
    while let Some((sess, v)) = rx.recv().await {
        let _ = tx.send((sess, v.to_string()));
    }
}

/// Autre client du `AppServer` : version propre du trailer de classe.
impl AppServer {
    /// Met un événement dans la file (ordre FIFO) et, si un **même**
    /// composant+événement est déjà en cours d'exécution, demande son
    /// annulation immédiate (drapeau `cancel` consultable via `ctx.cancelled`).
    fn enqueue(
        &self,
        session_id: Option<String>,
        wire: WireEvent,
        finish: Option<oneshot::Sender<Value>>,
    ) {
        {
            let cur = self.current.lock().unwrap();
            if let Some((c, e, flag)) = cur.as_ref() {
                if c == &wire.c && e == &wire.e {
                    flag.store(true, Ordering::SeqCst);
                }
            }
        }
        let job = Job {
            session_id,
            wire,
            cancel: Arc::new(AtomicBool::new(false)),
            finish,
        };
        let _ = self.job_tx.send(job);
    }

    /// Récupère ou crée la table de valeurs pour une session donnée.
    fn get_session_values(
        &self,
        session_id: &Option<String>,
    ) -> Arc<Mutex<HashMap<String, Value>>> {
        if !self.app.isolated_sessions {
            return Arc::clone(&self.values);
        }
        let Some(sess) = session_id else {
            return Arc::clone(&self.values);
        };
        let mut map = self.sessions.lock().unwrap();
        map.entry(sess.clone())
            .or_insert_with(|| Arc::new(Mutex::new(HashMap::new())))
            .clone()
    }
}

/// File d'attente : exécute les événements un à un, dans l'ordre.
async fn dispatcher(server: Arc<AppServer>, mut rx: mpsc::UnboundedReceiver<Job>) {
    while let Some(job) = rx.recv().await {
        {
            let mut cur = server.current.lock().unwrap();
            cur.replace((job.wire.c.clone(), job.wire.e.clone(), job.cancel.clone()));
        }

        let app = Arc::clone(&server.app);
        let session_values = server.get_session_values(&job.session_id);
        let push_tx = server.push_tx.clone();
        let job_cancel = job.cancel.clone();
        let wire = job.wire.clone();
        let sess_id = job.session_id.clone();

        let (chan_tx, mut chan_rx) = mpsc::unbounded_channel::<Value>();
        let target_sess = sess_id.clone();
        tokio::spawn(async move {
            while let Some(v) = chan_rx.recv().await {
                let _ = push_tx.send((target_sess.clone(), v));
            }
        });

        let out = tokio::task::spawn_blocking(move || {
            run_event(&wire, &app, &session_values, &chan_tx, job_cancel)
        })
        .await
        .unwrap_or_else(|_| json!({ "t": "error", "m": "tâche interrompue" }));

        vlog(
            &server.app,
            "[run]",
            &format!("{} · {} -> {}", job.wire.c, job.wire.e, summarize(&out)),
        );

        {
            let mut cur = server.current.lock().unwrap();
            cur.take();
        }

        // Pousse la réponse finale ciblée sur la session de l'émetteur.
        let _ = server.push_tx.send((sess_id, out.clone()));
        if let Some(fin) = job.finish {
            let _ = fin.send(out);
        }
    }
}

/// Exécute un événement : instantané des entrées, handlers, et fusion des
/// valeurs dans l'état partagé. Tourne dans un thread (`spawn_blocking`).
fn run_event(
    wire: &WireEvent,
    app: &App,
    values: &Mutex<HashMap<String, Value>>,
    push: &mpsc::UnboundedSender<Value>,
    cancel: Arc<AtomicBool>,
) -> Value {
    let mut inputs = match values.lock() {
        Ok(v) => v.clone(),
        Err(_) => HashMap::new(),
    };
    for (k, v) in wire.v.iter() {
        inputs.insert(k.clone(), v.clone());
    }

    let mut ctx = Context::new(
        Arc::new(inputs),
        Some(push.clone()),
        cancel,
        Some(wire.clone()),
    )
    .with_wasm(Arc::clone(&app.wasm_registry));
    let out = process_event(wire, app, &mut ctx);

    // Fusionne les nouvelles valeurs dans l'état partagé.
    if let Ok(mut v) = values.lock() {
        if let Some(u) = out.get("u").and_then(|u| u.as_array()) {
            for item in u {
                if let Some(id) = item.get("id").and_then(|x| x.as_str()) {
                    if let Some(val) = item.pointer("/p/value") {
                        v.insert(id.to_string(), val.clone());
                    }
                }
            }
        }
    }

    out
}

/// Log d'activité de la console (désactivable via `App::quiet`).
fn vlog(app: &App, tag: &str, msg: &str) {
    if app.verbose {
        eprintln!("\x1b[90m  {tag}\x1b[0m {msg}");
    }
}

/// Formate une valeur de façon compacte (tronquée pour le log).
fn short(v: &Value) -> String {
    let s = match v {
        Value::String(s) => s.clone(),
        other => other.to_string(),
    };
    let taken: String = s.chars().take(48).collect();
    if taken.len() < s.len() {
        format!("{taken}…")
    } else {
        taken
    }
}

/// Résume une réponse serveur (updates / erreur) pour le log.
fn summarize(out: &Value) -> String {
    match out.get("t").and_then(|v| v.as_str()) {
        Some("update") => {
            let Some(u) = out.get("u").and_then(|u| u.as_array()) else {
                return "ok, sans mise à jour".to_string();
            };
            if u.is_empty() {
                return "aucune mise à jour".to_string();
            }
            let mut parts = Vec::new();
            for upd in u {
                let id = upd.get("id").and_then(|v| v.as_str()).unwrap_or("?");
                let val = upd.pointer("/p/value").map(short).unwrap_or_default();
                parts.push(format!("{id}={val}"));
            }
            format!("{} mise(s) à jour → {}", u.len(), parts.join(", "))
        }
        Some("error") => {
            let m = out.get("m").map(short).unwrap_or_default();
            format!("erreur → {m}")
        }
        _ => "ok".to_string(),
    }
}

async fn index(State(server): State<Arc<AppServer>>) -> impl IntoResponse {
    let page = render_page(&server.app);
    vlog(
        &server.app,
        "[http]",
        &format!("GET / 200 · {} octets", page.len()),
    );
    Html(page)
}

async fn js_asset(State(server): State<Arc<AppServer>>) -> impl IntoResponse {
    vlog(&server.app, "[http]", "GET /assets/app.js 200");
    (
        [(header::CONTENT_TYPE, "text/javascript; charset=utf-8")],
        APP_JS,
    )
}

async fn css_asset(State(server): State<Arc<AppServer>>) -> impl IntoResponse {
    vlog(&server.app, "[http]", "GET /assets/styles.css 200");
    ([(header::CONTENT_TYPE, "text/css; charset=utf-8")], STYLES)
}

/// `GET /api/openapi.json` — Spécification OpenAPI 3.0.3 générée automatiquement.
async fn openapi_spec(State(server): State<Arc<AppServer>>) -> impl IntoResponse {
    let io = collect_io(&server.app);
    let mut all: Vec<&dyn Component> = Vec::new();
    for c in server.app.root.children() {
        walk(c, &mut all);
    }

    let mut input_props = serde_json::Map::new();
    for id in &io.inputs {
        input_props.insert(
            id.clone(),
            json!({ "type": "string", "description": format!("Input component `{id}`") }),
        );
    }

    let mut output_props = serde_json::Map::new();
    for id in &io.outputs {
        output_props.insert(
            id.clone(),
            json!({ "type": "string", "description": format!("Output component `{id}`") }),
        );
    }

    let spec = json!({
        "openapi": "3.0.3",
        "info": {
            "title": server.app.title,
            "description": if server.app.subtitle.is_empty() { "Auto-generated REST API by grio." } else { &server.app.subtitle },
            "version": "1.0.0"
        },
        "paths": {
            "/api/predict": {
                "post": {
                    "summary": "Run prediction pipeline",
                    "description": "Execute the main application pipeline with the provided input parameters.",
                    "security": if server.app.api_key.is_some() { json!([{ "ApiKeyAuth": [] }]) } else { json!([]) },
                    "requestBody": {
                        "required": true,
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object",
                                    "properties": {
                                        "data": {
                                            "type": "array",
                                            "description": "Positional values ordered matching inputs",
                                            "items": { "type": "string" }
                                        },
                                        "inputs": {
                                            "type": "object",
                                            "properties": input_props
                                        }
                                    }
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "Successful prediction response",
                            "content": {
                                "application/json": {
                                    "schema": {
                                        "type": "object",
                                        "properties": {
                                            "ok": { "type": "boolean" },
                                            "data": { "type": "array", "items": { "type": "string" } },
                                            "outputs": { "type": "object", "properties": output_props }
                                        }
                                    }
                                }
                            }
                        },
                        "401": { "description": "Unauthorized — Invalid or missing API key" }
                    }
                }
            },
            "/api/schema": {
                "get": {
                    "summary": "Get application components manifest",
                    "responses": { "200": { "description": "Manifest schema" } }
                }
            }
        },
        "components": {
            "securitySchemes": {
                "ApiKeyAuth": {
                    "type": "apiKey",
                    "in": "header",
                    "name": "X-API-Key"
                }
            }
        }
    });

    (
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        Json(spec),
    )
}

/// `GET /docs` — Documentation Swagger UI légère.
async fn docs_page(State(server): State<Arc<AppServer>>) -> impl IntoResponse {
    let title = esc_html(&server.app.title);
    let html = format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="utf-8" />
  <meta name="viewport" content="width=device-width, initial-scale=1" />
  <title>{title} — API Documentation</title>
  <link rel="stylesheet" href="https://unpkg.com/swagger-ui-dist@5/swagger-ui.css" />
  <style>
    body {{ margin: 0; padding: 0; background: #fafafa; font-family: sans-serif; }}
    .topbar {{ display: none; }}
  </style>
</head>
<body>
  <div id="swagger-ui"></div>
  <script src="https://unpkg.com/swagger-ui-dist@5/swagger-ui-bundle.js" crossorigin></script>
  <script>
    window.onload = () => {{
      window.ui = SwaggerUIBundle({{
        url: '/api/openapi.json',
        dom_id: '#swagger-ui',
        deepLinking: true,
        presets: [SwaggerUIBundle.presets.apis],
      }});
    }};
  </script>
</body>
</html>"#
    );
    Html(html)
}

/// `GET /api/schema` — manifeste auto-généré de l'application et de son API.
async fn schema(State(server): State<Arc<AppServer>>) -> Json<Value> {
    let io = collect_io(&server.app);
    let mut all: Vec<&dyn Component> = Vec::new();
    for c in server.app.root.children() {
        walk(c, &mut all);
    }
    let components: Vec<Value> = all
        .iter()
        .map(|c| { let c = *c; json!({ "id": c.id(), "kind": c.kind(), "role": role_str(c.role()), "props": merge_props(c) }) })
        .collect();

    vlog(
        &server.app,
        "[api]",
        &format!(
            "GET /api/schema · {} composants, {} entrées, {} sorties",
            components.len(),
            io.inputs.len(),
            io.outputs.len()
        ),
    );

    Json(json!({
        "app": {
            "title": server.app.title,
            "subtitle": server.app.subtitle,
            "live": server.app.live,
        },
        "endpoints": {
            "predict": {
                "method": "POST",
                "path": "/api/predict",
                "request": {
                    "data": "array — une valeur par input, dans l'ordre de `inputs`",
                    "alternatives": "object `{id: valeur}` ou clé `inputs`"
                },
                "response": {
                    "data": "array — une valeur par output, dans l'ordre de `outputs`",
                    "outputs": "object `{id: valeur}` (toutes les mises à jour)"
                }
            },
            "schema": { "method": "GET", "path": "/api/schema" },
            "mcp": { "method": "POST", "path": "/mcp/v1" },
            "openapi": { "method": "GET", "path": "/api/openapi.json" },
            "docs": { "method": "GET", "path": "/docs" }
        },
        "inputs": io.inputs,
        "outputs": io.outputs,
        "components": components,
    }))
}

/// `POST /api/predict` — exécute la même pipeline que l'UI (on_submit) avec vérification de clé API optionnelle.
async fn predict(
    State(server): State<Arc<AppServer>>,
    headers: axum::http::HeaderMap,
    Json(body): Json<Value>,
) -> impl IntoResponse {
    // Vérification de clé API si requise
    if let Some(ref required_key) = server.app.api_key {
        let auth_header = headers
            .get("X-API-Key")
            .or_else(|| headers.get("authorization"))
            .and_then(|v| v.to_str().ok());

        let valid = match auth_header {
            Some(key) if key == required_key => true,
            Some(bearer) if bearer.starts_with("Bearer ") && &bearer[7..] == required_key => true,
            _ => false,
        };

        if !valid {
            vlog(
                &server.app,
                "[api]",
                "accès refusé (clé API manquante ou invalide)",
            );
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                Json(json!({ "ok": false, "error": "Unauthorized — Invalid or missing API key" })),
            )
                .into_response();
        }
    }

    let io = collect_io(&server.app);
    let mut inputs: HashMap<String, Value> = HashMap::new();

    match body.get("data").cloned().unwrap_or(Value::Null) {
        Value::Array(arr) => {
            for (idx, v) in arr.iter().enumerate() {
                if let Some(id) = io.inputs.get(idx) {
                    inputs.insert(id.clone(), v.clone());
                }
            }
        }
        Value::Object(map) => inputs.extend(map),
        _ => {}
    }
    if let Some(Value::Object(map)) = body.get("inputs") {
        for (k, v) in map {
            inputs.insert(k.clone(), v.clone());
        }
    }

    let run_id = server
        .app
        .run_button
        .clone()
        .unwrap_or_else(|| "run".to_string());
    let wire = WireEvent {
        t: "event".to_string(),
        c: run_id,
        e: "click".to_string(),
        d: None,
        v: inputs,
    };

    // /api/predict est isolé : on n'exécute que sur les entrées fournies et on
    // exige la présence de toutes les entrées déclarées (comportement Gradio).
    for id in &io.inputs {
        if !wire.v.contains_key(id) {
            return (
                axum::http::StatusCode::BAD_REQUEST,
                Json(json!({ "ok": false, "error": json!({"missing_input": id}) })),
            )
                .into_response();
        }
    }

    vlog(
        &server.app,
        "[api]",
        &format!(
            "POST /api/predict · entrées {{{}}}",
            wire.v
                .iter()
                .map(|(k, v)| format!("{k}={}", short(v)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    );

    let (fin_tx, fin_rx) = oneshot::channel();
    server.enqueue(None, wire, Some(fin_tx));

    let out = match fin_rx.await {
        Ok(o) => o,
        Err(_) => json!({ "t": "error", "m": "exécution interrompue" }),
    };

    if out.get("t").and_then(|v| v.as_str()) == Some("error") {
        let m = out.get("m").map(short).unwrap_or_default();
        vlog(&server.app, "[api]", &format!("! {m}"));
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(json!({ "ok": false, "error": out.get("m").cloned().unwrap_or(Value::Null) })),
        )
            .into_response();
    }

    let mut by_id: HashMap<String, Value> = HashMap::new();
    if let Some(updates) = out.get("u").and_then(|u| u.as_array()) {
        for u in updates {
            if let Some(id) = u.get("id").and_then(|v| v.as_str()) {
                if let Some(v) = u.pointer("/p/value") {
                    by_id.insert(id.to_string(), v.clone());
                }
            }
        }
    }
    vlog(&server.app, "[api]", &format!("ok - {}", summarize(&out)));

    let data: Vec<Value> = io
        .outputs
        .iter()
        .map(|id| by_id.get(id).cloned().unwrap_or(Value::Null))
        .collect();
    (
        axum::http::StatusCode::OK,
        Json(json!({
            "ok": true,
            "data": data,
            "outputs": Value::Object(by_id.into_iter().collect()),
        })),
    )
        .into_response()
}

/// `POST /mcp/v1` — Point d'entrée Model Context Protocol (JSON-RPC 2.0).
async fn mcp_endpoint(
    State(server): State<Arc<AppServer>>,
    headers: axum::http::HeaderMap,
    Json(body): Json<crate::mcp::McpRpcRequest>,
) -> impl IntoResponse {
    // Vérification de clé API si requise
    if let Some(ref required_key) = server.app.api_key {
        let auth_header = headers
            .get("X-API-Key")
            .or_else(|| headers.get("authorization"))
            .and_then(|v| v.to_str().ok());

        let valid = match auth_header {
            Some(key) if key == required_key => true,
            Some(bearer) if bearer.starts_with("Bearer ") && &bearer[7..] == required_key => true,
            _ => false,
        };

        if !valid {
            vlog(
                &server.app,
                "[mcp]",
                "accès refusé (clé API manquante ou invalide)",
            );
            return (
                axum::http::StatusCode::UNAUTHORIZED,
                Json(crate::mcp::McpRpcResponse::error(
                    body.id,
                    -32000,
                    "Unauthorized — Invalid or missing API key",
                    None,
                )),
            )
                .into_response();
        }
    }

    vlog(
        &server.app,
        "[mcp]",
        &format!("RPC method: '{}'", body.method),
    );

    let resp = crate::mcp::handle_mcp_request(body, &server.app.title, &server.app.mcp_tools);

    (
        axum::http::StatusCode::OK,
        [(header::CONTENT_TYPE, "application/json; charset=utf-8")],
        Json(resp),
    )
        .into_response()
}

/// `GET /mcp/v1` — Découverte de métadonnées MCP.
async fn mcp_discovery(State(server): State<Arc<AppServer>>) -> impl IntoResponse {
    Json(json!({
        "protocol": "mcp",
        "version": "2024-11-05",
        "name": format!("grio-mcp-{}", server.app.title),
        "tools_count": server.app.mcp_tools.len(),
        "endpoints": {
            "rpc": "/mcp/v1",
            "tools": "/mcp/tools"
        }
    }))
}

/// `GET /mcp/tools` — Liste directe des outils MCP enregistrés.
async fn mcp_tools_list(State(server): State<Arc<AppServer>>) -> impl IntoResponse {
    let tools_json: Vec<Value> = server
        .app
        .mcp_tools
        .iter()
        .map(|t| t.to_mcp_json())
        .collect();
    Json(json!({ "tools": tools_json }))
}

fn role_str(r: Role) -> &'static str {
    match r {
        Role::Input => "input",
        Role::Output => "output",
        Role::None => "none",
    }
}

/// Parcourt l'arbre et collecte les composants (ordre de rendu).
fn walk<'a>(c: &'a dyn Component, out: &mut Vec<&'a dyn Component>) {
    out.push(c);
    for ch in c.children() {
        walk(ch, out);
    }
}

/// Liste ordonnée des entrées et sorties (déduites du rôle déclaratif).
fn collect_io(app: &App) -> IoParts {
    let mut nodes: Vec<&dyn Component> = Vec::new();
    for c in app.root.children() {
        walk(c, &mut nodes);
    }
    let mut io = IoParts::default();
    for n in nodes {
        match n.role() {
            Role::Input => io.inputs.push(n.id().to_string()),
            Role::Output => io.outputs.push(n.id().to_string()),
            Role::None => {}
        }
    }
    io
}

#[derive(Default)]
struct IoParts {
    inputs: Vec<String>,
    outputs: Vec<String>,
}

async fn ws_upgrade(
    ws: WebSocketUpgrade,
    State(server): State<Arc<AppServer>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| socket_loop(socket, server))
}

async fn socket_loop(socket: WebSocket, server: Arc<AppServer>) {
    let client_id = server.clients.fetch_add(1, Ordering::SeqCst) + 1;
    let session_id = format!("sess_{client_id}");
    vlog(
        &server.app,
        "[ws]",
        &format!("client #{client_id} connecté (session: {session_id})"),
    );

    let (mut sink, mut stream) = socket.split();
    let rx = server.tx.subscribe();

    let srv_task = Arc::clone(&server);
    let my_session = session_id.clone();
    tokio::spawn(async move {
        let mut rx = rx;
        while let Ok((target_sess, msg)) = rx.recv().await {
            // Si le message est ciblé sur une session précise, on ne l'envoie qu'à elle
            if let Some(ref target) = target_sess {
                if target != &my_session {
                    continue;
                }
            }
            if sink.send(Message::text(msg)).await.is_err() {
                break;
            }
        }
        drop(srv_task);
    });

    while let Some(Ok(msg)) = stream.next().await {
        match msg {
            Message::Text(t) => {
                let Ok(envelope) = serde_json::from_str::<Value>(t.as_str()) else {
                    vlog(&server.app, "[ws]", "message illisible (JSON invalide)");
                    let _ = server.push_tx.send((
                        Some(session_id.clone()),
                        json!({ "t": "error", "m": "bad message" }),
                    ));
                    continue;
                };
                if envelope.get("t").and_then(|v| v.as_str()) == Some("stream") {
                    handle_stream(&server, &envelope);
                    continue;
                }
                match serde_json::from_value::<WireEvent>(envelope) {
                    Ok(wire) if wire.t == "event" => {
                        let data = wire.d.as_ref().map(short).unwrap_or_default();
                        if wire.e == "load" {
                            vlog(
                                &server.app,
                                "[ws]",
                                &format!("client #{client_id} · load (page)"),
                            );
                        } else {
                            vlog(
                                &server.app,
                                "[ws]",
                                &format!("client #{client_id} · {} = {}", wire.c, wire.e),
                            );
                        }
                        if !data.is_empty() {
                            vlog(&server.app, "[ws]", &format!("   donnée : {data}"));
                        }
                        server.enqueue(Some(session_id.clone()), wire, None);
                    }
                    Ok(_) => {}
                    Err(_) => {
                        vlog(&server.app, "[ws]", "message illisible (JSON invalide)");
                        let _ = server.push_tx.send((
                            Some(session_id.clone()),
                            json!({ "t": "error", "m": "bad message" }),
                        ));
                    }
                }
            }
            Message::Close(_) => {
                vlog(
                    &server.app,
                    "[ws]",
                    &format!("client #{client_id} déconnecté"),
                );
                break;
            }
            _ => {}
        }
    }
}

/// Fragment de flux streaming reçu (`{t:"stream", c, p:{mime, b64}}`).
#[derive(serde::Deserialize)]
struct WireStream {
    c: String,
    p: StreamPart,
}

/// Contenu d'un fragment : type MIME et données base64.
#[derive(serde::Deserialize)]
struct StreamPart {
    mime: String,
    b64: String,
}

/// Traite un fragment de flux : met à jour les statistiques serveur du
/// composant, pousse une mise à jour live ma le **total**, et déclenche le
/// handler `"stream"` du composant (via la file classique).
fn handle_stream(server: &AppServer, envelope: &Value) {
    let Ok(wire) = serde_json::from_value::<WireStream>(envelope.clone()) else {
        vlog(&server.app, "[ws]", "flux streaming illisible");
        return;
    };
    if wire.p.b64.len() > 20_000_000 {
        vlog(&server.app, "[ws]", "fragment trop volumineux (ignoré)");
        return;
    }

    let bytes = media::decode(&wire.p.b64)
        .map(|b| b.len() as u64)
        .unwrap_or(0);
    let stats = {
        let mut vals = server.values.lock().unwrap();
        let entry = vals
            .entry(wire.c.clone())
            .or_insert_with(|| json!({ "mime": wire.p.mime.clone(), "bytes": 0, "chunks": 0 }));
        entry["mime"] = json!(wire.p.mime.clone());
        entry["bytes"] = json!(entry.get("bytes").and_then(|x| x.as_u64()).unwrap_or(0) + bytes);
        entry["chunks"] = json!(entry.get("chunks").and_then(|x| x.as_u64()).unwrap_or(0) + 1);
        entry.clone()
    };

    let _ = server.push_tx.send((
        None,
        json!({
            "t": "update",
            "u": [{ "id": wire.c, "p": { "stream": stats } }]
        }),
    ));

    server.enqueue(
        None,
        WireEvent {
            t: "event".to_string(),
            c: wire.c,
            e: "stream".to_string(),
            d: None,
            v: HashMap::new(),
        },
        None,
    );
}

fn render_page(app: &App) -> String {
    let title = esc_html(&app.title);
    let sub = if app.subtitle.is_empty() {
        String::new()
    } else {
        format!("<p class=\"mg-subtitle\">{}</p>", esc_html(&app.subtitle))
    };

    let theme_mode_attr = match app.theme.mode {
        crate::app::ThemeMode::Dark => " data-theme=\"dark\"",
        crate::app::ThemeMode::Light => " data-theme=\"light\"",
        crate::app::ThemeMode::System => "",
    };

    let mut theme_css_vars = Vec::new();
    if let Some(ref p) = app.theme.primary {
        theme_css_vars.push(format!(
            "--mg-primary: {p}; --mg-accent: {p}; --mg-accent-2: {p}; --mg-primary-hover: {p}ee;"
        ));
    }
    if let Some(ref r) = app.theme.radius {
        theme_css_vars.push(format!(
            "--mg-radius: {r}; --mg-radius-sm: calc({r} * 0.6); --mg-radius-lg: calc({r} * 1.5);"
        ));
    }
    if let Some(ref f) = app.theme.font {
        theme_css_vars.push(format!("--mg-font-family: {f}; --mg-font: {f};"));
    }
    let theme_style = if theme_css_vars.is_empty() {
        String::new()
    } else {
        format!("<style>:root {{ {} }}</style>", theme_css_vars.join(" "))
    };

    let toggle_html = if app.theme.toggle {
        r#"<button id="mg-theme-toggle" class="mg-theme-toggle" type="button" title="Toggle Theme" aria-label="Toggle Theme">🌓</button>"#
    } else {
        ""
    };

    let shell_style = if let Some(mw) = app.max_width {
        format!(" style=\"max-width: {mw}px;\"")
    } else {
        String::new()
    };

    let is_multipage = !app.pages.is_empty();
    let multipage_cls = if is_multipage { " mg-has-sidebar" } else { "" };

    let sidebar_toggle = if is_multipage {
        r#"<button id="mg-sidebar-toggle" class="mg-sidebar-toggle" type="button" title="Toggle Navigation Menu" aria-label="Toggle Navigation">☰</button>"#
    } else {
        ""
    };

    let mut sidebar_html = String::new();
    if is_multipage {
        sidebar_html.push_str("<aside class=\"mg-sidebar\" id=\"mg-sidebar\"><div class=\"mg-sidebar-header\"><span class=\"mg-sidebar-brand\">📌 Pages</span><button id=\"mg-sidebar-close\" class=\"mg-sidebar-close\" type=\"button\">✕</button></div><nav class=\"mg-nav-list\">");
        for (i, p) in app.pages.iter().enumerate() {
            let active = if i == 0 { " active" } else { "" };
            let icon_str = p.icon.as_deref().unwrap_or("📄");
            sidebar_html.push_str(&format!(
                r#"<a href="{}" class="mg-nav-item{active}" data-grio-route="{}" data-page-target="{}"><span class="mg-nav-icon">{}</span><span class="mg-nav-title">{}</span></a>"#,
                esc_html(&p.route),
                esc_html(&p.route),
                esc_html(&p.id),
                esc_html(icon_str),
                esc_html(&p.title)
            ));
        }
        sidebar_html.push_str("</nav></aside><div id=\"mg-sidebar-backdrop\" class=\"mg-sidebar-backdrop\" hidden></div>");
    }

    let mut body = String::new();
    if is_multipage {
        for (i, p) in app.pages.iter().enumerate() {
            let active = if i == 0 { " active" } else { "" };
            body.push_str(&format!(
                r#"<section class="mg-page-view{active}" id="{}" data-route="{}">"#,
                esc_html(&p.id),
                esc_html(&p.route)
            ));
            if let Some(ch) = app.root.children().get(i) {
                render_component(*ch, &mut body);
            }
            body.push_str("</section>");
        }
        // Render any global components added at root level (e.g. Drawers, floating modals)
        let root_children = app.root.children();
        if root_children.len() > app.pages.len() {
            for ch in &root_children[app.pages.len()..] {
                render_component(*ch, &mut body);
            }
        }
    } else {
        for c in app.root.children() {
            render_component(c, &mut body);
        }
    }

    format!(
        r#"<!doctype html>
<html lang="en"{theme_mode_attr}>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{title}</title>
<link rel="stylesheet" href="/assets/styles.css">
{theme_style}
</head>
<body class="{multipage_cls}">
{sidebar_html}
<div class="mg-app-container">
<main class="mg-shell"{shell_style}>
  <header class="mg-header">
    <div class="mg-header-row">
      <div class="mg-header-left">
        {sidebar_toggle}
        <h1 class="mg-title">{title}</h1>
      </div>
      {toggle_html}
    </div>
    {sub}
  </header>
  <div class="mg-root" id="mg-root">
{body}
  </div>
  <footer class="mg-footer">
    <div class="mg-footer-brand">
      <span>Powered by <strong class="mg-brand-gradient">⚙️ grio</strong> · Native Rust Web Framework</span>
    </div>
    <div class="mg-footer-actions">
      <button id="mg-api-btn" class="mg-footer-btn mg-api-launcher" type="button" title="View API Code Snippets (Python, JS, cURL, MCP)">⚡ <span data-i18n="use_api">Use via API</span></button>
      <a href="/docs" target="_blank" class="mg-footer-link" title="OpenAPI Documentation">📖 <span data-i18n="api_docs">API Docs</span></a>
      <a href="/api/schema" target="_blank" class="mg-footer-link" title="JSON Schema">⚙️ <span data-i18n="schema">Schema</span></a>
      <button id="mg-prefs-btn" class="mg-footer-btn" type="button" title="Preferences & Settings">⚙️ <span data-i18n="settings">Settings</span></button>
    </div>
  </footer>
</main>
</div>

<!-- Modal: Preferences & Language -->
<div id="mg-prefs-modal" class="mg-modal-overlay" hidden>
  <div class="mg-modal-dialog">
    <header class="mg-modal-header">
      <h3>⚙️ <span data-i18n="settings_title">Application Settings</span></h3>
      <button id="mg-prefs-close" class="mg-modal-close" type="button">✕</button>
    </header>
    <div class="mg-modal-body">
      <div class="mg-modal-section">
        <h4>🌐 <span data-i18n="language">Language / Langue</span></h4>
        <div class="mg-lang-switch-group">
          <button class="mg-lang-btn active" data-set-lang="en">🇬🇧 English</button>
          <button class="mg-lang-btn" data-set-lang="fr">🇫🇷 Français</button>
          <button class="mg-lang-btn" data-set-lang="es">🇪🇸 Español</button>
          <button class="mg-lang-btn" data-set-lang="de">🇩🇪 Deutsch</button>
        </div>
      </div>
      <div class="mg-modal-section">
        <h4>🎨 <span data-i18n="theme">Theme Customization</span></h4>
        <div class="mg-theme-switch-group">
          <button class="mg-theme-btn" data-set-theme="system">💻 System</button>
          <button class="mg-theme-btn" data-set-theme="light">☀️ Light</button>
          <button class="mg-theme-btn" data-set-theme="dark">🌙 Dark</button>
        </div>
      </div>
    </div>
  </div>
</div>

<!-- Modal: Use via API (Python, JS, cURL, MCP Snippets) -->
<div id="mg-api-modal" class="mg-modal-overlay" hidden>
  <div class="mg-modal-dialog mg-modal-lg">
    <header class="mg-modal-header">
      <div class="mg-modal-title-group">
        <h3>⚡ <span data-i18n="api_title">API Documentation & Client Code</span></h3>
        <span class="mg-badge-endpoint">POST /api/predict</span>
      </div>
      <button id="mg-api-close" class="mg-modal-close" type="button">✕</button>
    </header>
    <div class="mg-modal-body">
      <p class="mg-api-intro" data-i18n="api_intro">Interact with this grio AI application programmatically via Python, JavaScript, cURL or Model Context Protocol (MCP).</p>
      
      <!-- Snippet Tabs -->
      <div class="mg-api-tabs">
        <button class="mg-api-tab active" data-tab="python">🐍 Python</button>
        <button class="mg-api-tab" data-tab="js">🟨 JavaScript</button>
        <button class="mg-api-tab" data-tab="curl">💻 cURL</button>
        <button class="mg-api-tab" data-tab="mcp">🤖 MCP Tool</button>
      </div>

      <!-- Code Snippet Area -->
      <div class="mg-api-code-wrapper">
        <div class="mg-api-code-header">
          <span id="mg-api-lang-tag">python</span>
          <button id="mg-copy-snippet-btn" class="mg-copy-btn" type="button">📋 <span data-i18n="copy_code">Copy Snippet</span></button>
        </div>
        <pre class="mg-api-code-block"><code id="mg-api-code-content"></code></pre>
      </div>

      <div class="mg-api-meta-info">
        <div class="mg-api-meta-item">
          <strong>Endpoint URL:</strong> <code id="mg-api-full-url">http://127.0.0.1:7860/api/predict</code>
        </div>
        <div class="mg-api-meta-item">
          <strong>Content-Type:</strong> <code>application/json</code>
        </div>
      </div>
    </div>
  </div>
</div>

<script src="/assets/app.js"></script>
</body>
</html>"#
    )
}

/// Génère le HTML autonome complet avec styles et JS inlinés pour un Space statique.
pub(crate) fn render_standalone_html(app: &App) -> String {
    let raw = render_page(app);
    let with_css = raw.replace(
        r#"<link rel="stylesheet" href="/assets/styles.css">"#,
        &format!("<style>\n{STYLES}\n</style>"),
    );
    with_css.replace(
        r#"<script src="/assets/app.js"></script>"#,
        &format!("<script>\n{APP_JS}\n</script>"),
    )
}

/// Props du composant **avec** le réglage de mise en page fusionné
/// (`prop.layout` omis s'il est vide).
fn merge_props(c: &dyn Component) -> Value {
    let mut p = c.props();
    let extra = c.layout().json();
    if let (Value::Object(props), Value::Object(layout)) = (&mut p, extra) {
        if !layout.is_empty() {
            props.insert("layout".to_string(), Value::Object(layout));
        }
    }
    p
}

fn render_component(c: &dyn Component, out: &mut String) {
    let props_json = merge_props(c);
    let props = attr_escape(&props_json.to_string());
    let role = role_str(c.role());
    match c.kind() {
        "row" => {
            out.push_str(&format!(
                r#"<section class="mg-row" data-kind="row" data-id="{}" data-role="{role}" data-props='{}'>"#,
                attrs(c.id()),
                props
            ));
            for ch in c.children() {
                render_component(ch, out);
            }
            out.push_str("</section>");
        }
        "column" => {
            out.push_str(&format!(
                r#"<div class="mg-column" data-kind="column" data-id="{}" data-role="{role}" data-props='{}'>"#,
                attrs(c.id()),
                props
            ));
            for ch in c.children() {
                render_component(ch, out);
            }
            out.push_str("</div>");
        }
        "grid" => {
            out.push_str(&format!(
                r#"<div class="mg-grid" data-kind="grid" data-id="{}" data-role="{role}" data-props='{}'>"#,
                attrs(c.id()),
                props
            ));
            for ch in c.children() {
                render_component(ch, out);
            }
            out.push_str("</div>");
        }
        "panel" => {
            let label = props_json["label"].as_str().unwrap_or("");
            out.push_str(&format!(
                r#"<section class="mg-panel" data-kind="panel" data-id="{}" data-role="{role}" data-props='{}'><header class="mg-panel-title">{}</header><div class="mg-panel-body">"#,
                attrs(c.id()),
                props,
                esc_html(label)
            ));
            for ch in c.children() {
                render_component(ch, out);
            }
            out.push_str("</div></section>");
        }
        "drawer" => {
            let title = props_json["title"].as_str().unwrap_or("");
            let placement = props_json["placement"].as_str().unwrap_or("right");
            let size = props_json["size"].as_u64().unwrap_or(380);
            let open = props_json["open"].as_bool().unwrap_or(false);
            let backdrop = props_json["backdrop"].as_bool().unwrap_or(true);
            let is_vertical = placement == "top" || placement == "bottom";
            let size_style = if is_vertical {
                format!("height: {size}px; max-height: 80vh;")
            } else {
                format!("width: {size}px; max-width: 90vw;")
            };
            let open_cls = if open { " mg-drawer-open" } else { "" };
            let backdrop_html = if backdrop {
                r#"<div class="mg-drawer-backdrop"></div>"#
            } else {
                ""
            };

            let mut inner = String::new();
            for ch in c.children() {
                render_component(ch, &mut inner);
            }

            out.push_str(&format!(
                r#"<div class="mg-drawer-container mg-drawer-{placement}{open_cls}" data-kind="drawer" data-id="{}" data-role="{role}" data-props='{}'>{backdrop_html}<div class="mg-drawer-panel" style="{size_style}"><header class="mg-drawer-header"><h3 class="mg-drawer-title">{}</h3><button class="mg-drawer-close" type="button" aria-label="Close">✕</button></header><div class="mg-drawer-body">{inner}</div></div></div>"#,
                attrs(c.id()),
                props,
                esc_html(title)
            ));
        }
        "tabitem" => {
            out.push_str(&format!(
                r#"<div class="mg-tab-pane" data-kind="tabitem" data-id="{}" data-role="{role}" data-props='{}'>"#,
                attrs(c.id()),
                props
            ));
            for ch in c.children() {
                render_component(ch, out);
            }
            out.push_str("</div>");
        }
        "tabs" => {
            let _labels: Vec<String> = props_json
                .get("labels")
                .and_then(|l| l.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let selected = props_json
                .get("selected")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as usize;

            out.push_str(&format!(
                r#"<div class="mg-tabs" data-kind="tabs" data-id="{}" data-role="{role}" data-props='{}'>"#,
                attrs(c.id()),
                props
            ));

            for (i, ch) in c.children().into_iter().enumerate() {
                let active = if i == selected { " mg-active" } else { "" };
                out.push_str(&format!(
                    r#"<div class="mg-tab-pane{active}" data-tab-index="{i}">"#
                ));
                render_component(ch, out);
                out.push_str("</div>");
            }

            out.push_str("</div>");
        }
        "accordion" => {
            let labels: Vec<String> = props_json
                .get("labels")
                .and_then(|l| l.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect()
                })
                .unwrap_or_default();
            let open_first = props_json
                .get("open")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            for (i, ch) in c.children().into_iter().enumerate() {
                let label = labels.get(i).map(String::as_str).unwrap_or("");
                let open = if i == 0 && open_first { " open" } else { "" };
                let mut inner = String::new();
                render_component(ch, &mut inner);
                out.push_str(&format!(
                    r#"<details class="mg-accordion-item" data-kind="accordion" data-id="{}" data-role="{role}" data-props='{}'{open}><summary>{}</summary><div class="mg-accordion-body">{inner}</div></details>"#,
                    attrs(c.id()),
                    props,
                    esc_html(label)
                ));
            }
        }
        "dynamic_container" => {
            let dir = props_json["direction"].as_str().unwrap_or("column");
            let cls = if dir == "row" {
                "mg-slot-row"
            } else {
                "mg-slot-column"
            };
            out.push_str(&format!(
                r#"<div class="mg-slot {cls}" data-kind="dynamic_container" data-id="{}" data-role="{role}" data-props='{}'>"#,
                attrs(c.id()),
                props
            ));
            for ch in c.children() {
                render_component(ch, out);
            }
            out.push_str("</div>");
        }
        kind => {
            out.push_str(&format!(
                r#"<div class="mg-field mg-{kind}" data-kind="{kind}" data-id="{}" data-role="{role}" data-props='{}'></div>"#,
                attrs(c.id()),
                props
            ));
        }
    }
}

pub(crate) fn render_fragment(c: &dyn Component) -> String {
    let mut out = String::new();
    render_component(c, &mut out);
    out
}

fn attrs(id: &str) -> String {
    esc_html(id)
}

fn esc_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn attr_escape(s: &str) -> String {
    s.replace('\'', "&#39;")
}

/// Paramètres de `GET /api/explore` : la racine (`root`) du composant et le
/// chemin relatif courant (`path`).
#[derive(serde::Deserialize)]
struct ExploreQuery {
    /// Dossier racine du composant `Explorer`.
    root: Option<String>,
    /// Chemin relatif à lister (vide = racine).
    path: Option<String>,
    /// Filtre de noms de fichiers (globe simple, ex. `*.rs`).
    pattern: Option<String>,
}

/// Fait correspondre `name` à un globe simple (`*` = n'importe quoi, `?` = un
/// caractère). Sans `*`/`?`, la correspondance est exacte.
fn glob_match(name: &str, pat: &str) -> bool {
    fn rec(n: &[char], p: &[char]) -> bool {
        match (n.first(), p.first()) {
            (_, None) => n.is_empty(),
            (None, Some(_)) => p.iter().all(|&c| c == '*'),
            (Some(_), Some('*')) => rec(n, &p[1..]) || rec(&n[1..], p),
            (Some(&nc), Some(&pc)) if nc == pc || pc == '?' => rec(&n[1..], &p[1..]),
            _ => false,
        }
    }
    let n: Vec<char> = name.chars().collect();
    let p: Vec<char> = pat.chars().collect();
    rec(&n, &p)
}

/// `GET /api/explore` — liste un dossier (racine bornée), pour les composants
/// `Explorer`. Un chemin hors de la racine est rejeté.
async fn explore(Query(q): Query<ExploreQuery>) -> Json<Value> {
    let root = q
        .root
        .filter(|r| !r.is_empty())
        .unwrap_or_else(|| ".".to_string());
    let base = match std::fs::canonicalize(&root) {
        Ok(p) => p,
        Err(e) => return Json(json!({ "t": "error", "m": format!("racine invalide : {e}") })),
    };

    let target = match q.path.as_deref().filter(|s| !s.is_empty()) {
        None => base.clone(),
        Some(rel) => {
            let cand_abs = std::fs::canonicalize(base.join(rel));
            match cand_abs {
                Ok(c) if c.starts_with(&base) => c,
                Ok(_) => return Json(json!({ "t": "error", "m": "chemin hors de la racine" })),
                Err(e) => {
                    return Json(json!({ "t": "error", "m": format!("chemin invalide : {e}") }))
                }
            }
        }
    };

    let mut dirs: Vec<String> = Vec::new();
    let mut files: Vec<String> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&target) {
        for entry in rd.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }
            if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
                dirs.push(name);
            } else {
                let matched = q
                    .pattern
                    .as_deref()
                    .filter(|p| !p.is_empty())
                    .map(|p| glob_match(&name, p))
                    .unwrap_or(true);
                if matched {
                    files.push(name);
                }
            }
        }
        dirs.sort();
        files.sort();
    }

    let rel = target.strip_prefix(&base).unwrap_or(&base);
    let rel: String = rel.to_string_lossy().replace('\\', "/");
    let rel = if rel.is_empty() {
        String::new()
    } else {
        format!("{rel}/")
    };

    vlog_none(&format!(
        "GET /api/explore -> {}",
        if rel.is_empty() { "." } else { &rel }
    ));
    Json(json!({
        "t": "ok",
        "root": base.to_string_lossy(),
        "path": rel,
        "dirs": dirs,
        "files": files,
    }))
}

/// Log simple hors application (routes hors `AppServer`).
fn vlog_none(msg: &str) {
    eprintln!("\x1b[90m  [api]\x1b[0m {msg}");
}
