# grio Roadmap

> Living document maintained throughout development. Each delivered feature
> checks its box, updates doc comments (`///`), documentation (`README.md`, `COMPONENTS.md`),
> and this roadmap. Inspired by the Gradio *Blocks and Event Listeners* guide
> as well as *Queuing, Streaming Outputs, Streaming Inputs, Alerts, and Progress Bars*.

## Current Status (Foundation)

Already implemented:
- [x] Engine: `axum` server + UI rendering (CSS3 + Vanilla JS, zero frontend dependencies, `MG.register`)
- [x] Core Components: `Text`, `Slider`, `Output`, `Markdown`, `Button`
- [x] Containers: `Row`, `Column`, `Panel`, `Grid`, `Tabs`, `Accordion`
- [x] Event Model: `on_submit`, `on_change(id)`, `on_click(id)`, `on("event", [ids], fn)` + internal bus `ctx.emit`
- [x] Declarative Roles: `Role::Input/Output` → auto REST API (`/api/predict`, `/api/schema`, `/docs`, `/api/openapi.json`)
- [x] Verbose Console: (`[http]`, `[ws]`, `[api]`, `[run]`), quiet mode toggleable (`.quiet()`)
- [x] Documentation: `#![warn(missing_docs)]` + `README.md` + [COMPONENTS.md](COMPONENTS.md)

Legend: **[P0]** High priority · **[P1]** Desirable · **[P2]** Under evaluation

---

## Phase 1 — Advanced Interaction (Blocks Inspiration) · [P0]

> **Delivered.** Design decision: the **flow** is not declared inline at function call
> like Gradio (`inputs=`/`outputs=`), but via builder chaining **`.flow(inputs, outputs)`**
> applied to the handler — achieving the same strict scoping for reads and writes.
> Chaining `.then/.success/.failure` also attaches directly to the handler.
> All criteria verified over WebSocket integration tests (`examples/blocks.rs`).

### 1.1 Per-Component Events with Declared Flow · ✅
`.flow(["a", "b"], ["cmp_out"])` declares readable inputs and writable outputs
for a handler; any unauthorized access is **rejected on read** (`get` → error)
and **ignored on write** (`set`/`set_prop`/`append`/`progress`). Multiple independent
flows coexist seamlessly without `on_submit`.
- **Files**: `app.rs` (`HandlerDef.inputs/outputs`, `App::flow`), `context.rs` (`set_flow`, guards)
- **Accepted when**: Gradio's "a > b / b > a" replicated in `examples/blocks.rs` with scoped handlers.

### 1.2 Event Chaining `.then()` / `.success()` / `.failure()` · ✅
Each primary handler supports an execution chain: `then` (always),
`success` (on success), `failure` (on error — a successful `failure` handler
recovers state and triggers subsequent `success` links).
- **Files**: `app.rs` (`Sibling`, `RunCond`, `handler.chain`, `run_handler`)
- **Accepted when**: Chatbot in `examples/blocks.rs` — `on_submit(user_fn).then(bot_fn)`
  renders streamed tokens after user submission.

### 1.3 Event Data Exposed to Context · ✅
`ctx.event() -> Option<&WireEvent>` exposes target (`c`), action (`e`),
data (`d`), and snapshot (`v`). Registered in `Context` by `server::run_event`.
- **Files**: `context.rs` (`Context.event`, accessor), `server.rs` (wire clone)
- **Accepted when**: Handler displays triggering metadata in `examples/blocks.rs`.

### 1.4 `gr.skip()` / Universal Property Patching (`gr.update`) · ✅
- `ctx.skip("id")` / `ctx.unskip`: subsequent writes to `id` are discarded.
- `ctx.set_prop("id", "visible"|"label", v)` dynamically mutates component properties.
- **Files**: `context.rs` (`skip`/`unskip`/`skipped`), `app.js`, `styles.css`.

### 1.5 Explicit Interactivity · ✅
`.interactive(bool)` on input components: rendered as `disabled`, greyed out,
yet remains included in the input snapshot.
- **Files**: `components.rs`, `app.js` (disabled state), `styles.css` (.mg-disabled).

### 1.6 Page Mount Event (`load`) · ✅
`App::on_load(fn)`: triggered by the client upon WebSocket connection
(`{t:'event', c:'', e:'load'}`), routed through the standard handler pipeline.
- **Files**: `events.rs` (`EventName::Load`), `app.rs` (`on_load`), `app.js` (emit), `server.rs`.

### 1.7 Multi-Trigger Handlers `gr.on` · ✅
`App::on("click"|"change", [ids...], fn)` binds the same shared function
(`Arc<HandlerFn>`) to multiple components; `ctx.event()` distinguishes the caller.
- **Files**: `app.rs` (`App::on`, `HandlerFn = Arc<...>`).

### 1.8 Advanced Layouts: Tabs and Accordions · ✅
- `Tabs::new(id).tab(label, builder)`: Client-side interactive tab panels.
- `Accordion::new(id).section(label, builder)`: Native `<details>/<summary>` collapsible sections.
- **Files**: `components.rs` (`Tabs`, `Accordion`), `server.rs`, `app.js`, `styles.css`.

---

## Phase 2 — Real-Time Engine (Queuing, Streaming, Alerts, Progress) · [P0]

> **Delivered.** Handlers run on `spawn_blocking` within a serialized queue,
> preventing thread pool exhaustion while keeping handler code ergonomic and synchronous.
> Cancellation is handled at enqueue time via `Arc<AtomicBool>` and `ctx.cancelled()`.
> Full real-time streaming, deduplication, alert toasts, and progress bar updates tested.

### 2.1 Async Handlers & Push Dispatching · ✅
`ctx.set`, `ctx.append`, `ctx.progress`, and `ctx.alert` push immediate updates
over the WebSocket broadcast channel while long tasks run.
- **Files**: `server.rs` (`dispatcher`, `run_event`), `context.rs` (`push` sender).

### 2.2 Queuing & Cancellation · ✅
Serialized `tokio::mpsc` queue ensures stable FIFO ordering without race conditions.
Re-triggering the same component event marks prior tasks as cancelled.
- **Files**: `server.rs` (`Job`, `dispatcher`, `enqueue`), `context.rs` (`cancelled`).

### 2.3 Streaming Outputs (LLM Token-by-Token) · ✅
Component streaming via **`ctx.append(id, chunk)`**: chunks are pushed exclusively
in real-time and deduplicated from the final response payload.
- **Files**: `context.rs` (`append`), `assets/app.js` (`apply`), `examples/greet.rs`.

### 2.4 User Alerts / Toasts · ✅
`ctx.alert(AlertLevel, msg)` dispatches `{t:"alert", level, msg}` rendered as styled toasts
(`info`, `success`, `warn`, `error`).
- **Files**: `context.rs` (`AlertLevel`, `alert`), `app.js`, `styles.css` (.mg-toast-*).

### 2.5 Dynamic Progress Bars · ✅
`Progress` component + **`ctx.progress(id, pct, label)`**: animated progress indicator
with stage message and completion highlight at 100%.
- **Files**: `components.rs` (`Progress`), `context.rs`, `app.js`, `styles.css`.

---

## Phase 3 — Media & Streaming Inputs · [P1]

> **Delivered.** Media transport uses Base64 **Data URLs** for zero-dependency portability.
> Server-side analysis via `media::inspect` extracts MIME types, byte sizes, and image dimensions.
> Live audio/video streaming utilizes `{t:"stream", c, p:{mime, b64}}` accumulating `StreamInfo`.

### 3.1 Media Components (Upload / Display) · ✅
`Image`, `Audio`, `Video`: support `.input()` (upload) and `.output()` (player/viewer),
with drag & drop and reactive `change` events.
- **Files**: `components.rs`, `app.js`, `styles.css`, `server.rs`.

### 3.2 Streaming Inputs (Microphone & Camera) · ✅
`getUserMedia` + `MediaRecorder` pipeline streams chunks directly through WebSocket to `App::on_stream`.
- **Files**: `media.rs` (`StreamInfo`, `decode`), `server.rs` (`handle_stream`), `app.js`.

### 3.3 Component Transport Lifecycle Events · ✅
`EventName::Play|Pause|Stop|Stream` wired to media player controls and server lifecycle hooks.
- **Files**: `events.rs`, `app.rs`, `server.rs`, `app.js`.

---

## Phase 4 — Core Parametric Widgets · [P0]

> **Delivered.** Ten rich interactive widgets written in Vanilla JS and CSS3 without npm dependencies.
> Includes built-in SVG vector charting (`Plot`), syntax-highlighted code editor (`Code`),
> inpainting canvas (`ImageEditor`), and sandboxed server-side file explorer (`Explorer`).

### 4.1 Configurable Components · ✅
`Checkbox`, `Dropdown` (single, multiple, searchable), `DatePicker`, `TimePicker`,
`Dataframe` (editable spreadsheet), `Plot` (SVG charts), `Gallery`, `SortableList` (HTML5 DnD),
`Code` (syntax highlighted), and `Explorer`.
- **Files**: `components.rs`, `app.js`, `styles.css`, `server.rs`, `lib.rs`.

### 4.2 Sandboxed Server File Explorer (`/api/explore`) · ✅
Secure directory browsing bounded by root canonicalization with glob pattern filtering.
- **Files**: `server.rs` (`explore`), `app.js`.

### 4.3 Client-Side Inpainting Image Editor (`ImageEditor`) · ✅
Canvas-based retouching: brush, eraser, shapes, crop, rotation, filters, undo/redo,
and 1–4 RGBA annotation layers generating mask outputs for AI inpainting.
- **Files**: `components.rs`, `app.js`, `styles.css`.

### 4.4 Modular Layout System (`Layout` / `WithLayout`) · ✅
Universal responsive layout styling: `width`, `height`, `scale` (`flex-grow`), and `min_width`.
- **Files**: `components.rs` (`WithLayout`), `app.rs` (`RowBuilder`), `app.js`.

---

## Phase 5 — Production · ✅

- [x] **Sessions isolées** : valeurs d'état et flux de messages temps réel routés par session client (`sess_id`).
- [x] **OpenAPI complet** : spécification OpenAPI 3.0.3 auto-générée sur `GET /api/openapi.json` + documentation Swagger UI sur `GET /docs`.
- [x] **Auth simple** : clé d'API paramétrable via `App::api_key(...)` (vérification `X-API-Key` ou `Bearer <token>`).
- [x] **CORS optionnel** + configuration fluide (`.cors(bool)`, `.docs(bool)`, `.isolate_sessions(bool)`).
- [x] **CLI `grio`** : outil en ligne de commande et générateur de projets `crates/grio-cli` (`grio new <nom> --template <chatbot|vision|greet>`).
- [x] **Tests d'intégration** : suite de tests automatisés `crates/grio/tests/api_predict.rs` validant le pipeline complet, l'OpenAPI, la doc et l'authentification.

## Phase 5 — Production & Developer Tooling · ✅

- [x] **Isolated Sessions**: Independent state and event streams per client session (`sess_id`).
- [x] **Full OpenAPI 3.0.3**: Auto-generated specification on `GET /api/openapi.json` + Swagger UI on `GET /docs`.
- [x] **Authentication**: API key protection via `App::api_key(...)` (`X-API-Key` or `Bearer <token>`).
- [x] **CORS & Network Config**: `.cors(bool)`, `.docs(bool)`, `.isolate_sessions(bool)`.
- [x] **`grio` Developer CLI**: Project generator in `crates/grio-cli` (`grio new <name> --template <chatbot|vision|greet>`).
- [x] **Integration Test Suite**: Automated end-to-end testing in `crates/grio/tests/api_predict.rs`.

---

## Phase 6 — AI Ecosystem, Multimodal Studio & Observability · [P0]

- [x] **Responsive Grid (`Grid`)**: Native CSS grid container with responsive column wrapping.
- [x] **Conversational Chatbot (`Chatbot`)**: Markdown formatting, code block highlighting, token streaming.
- [x] **English Reference Guide**: Complete widget documentation in [COMPONENTS.md](COMPONENTS.md).
- [x] **Integrated Theming Engine**: Light, dark, and auto modes with custom color accents (`Theme`).
- [x] **Observability Cards (`Metric`)**: Real-time KPI indicators with delta values and semantic colors.
- [x] **Multimodal AI Studio (`prompt_to_image`)**: Quantized Candle LLM (Qwen 2.5 7B GGUF) + SDXL diffusion studio.
- [x] **Multilingual i18n & API Code Modal**: On-the-fly language switching (🇬🇧, 🇫🇷, 🇪🇸, 🇩🇪) and client snippet generator (Python, JS, cURL, MCP Tool).

---

## Phase 7 — Files & Utility Widgets · ✅

> **Delivered.** Six Gradio-style widgets written end-to-end in English
> (frontend still zero-dependency: CSS3 + vanilla JS, no build step):
> `Number` (numeric field with bounds and a ± stepper, `f64` input),
> `Label` (result badge with semantic color), `Json` (live-validated JSON
> editor / `.output()` viewer), `Timer` (periodic clock that emits `change`
> each tick, so `on_change("id")` runs on a schedule), `File` (multi-upload
> with drag & drop, MIME filter, size cap, upload progress bar and removable
> list), `DownloadButton` (server-triggered download from a data URL or a
> `{b64, mime}` object pushed via `ctx.set`).
> Verified: build + docs zero warnings, `node --check` green, the `forms`
> demo renders the new panel and all WS scenarios pass.

### 7.1 Components (`Number`, `Label`, `Json`, `Timer`, `File`, `DownloadButton`) · ✅
- All six are fully configurable through builders and readable from
  handlers: `f64` (`Number`), `String` (`Label`), `serde_json::Value`
  (`Json`), elapsed seconds in `ctx.event().d` (`Timer`), and
  `Vec<serde_json::Value>` of `{name, size, mime, data_url}` (`File`).
- **Files**: `components.rs` (6 structs + rustdoc), `app.js` (6 registers),
  `styles.css` (Phase 7 block), `lib.rs` (re-exports), `server.rs`
  (no change — generic rendering), `forms.rs` (demo + handlers)
- **Accepted when**: `examples/forms.rs` — panel « Phase 7 — Files &
  Utilities »; the `Timer` ticks every 3 s and updates a `Label` through
  `on_change("clock")`; « Generate CSV » pushes `{ b64, mime }` into the
  `DownloadButton`; `on_submit` reads `numcpu`, `jsondoc` and `docs`
  (verified over WebSocket and in the served HTML).

---

## Phase 8 — Advanced Controls & AI Modalities · 🚀

> **Delivered.** Specialized components expanding grio's multimodal AI capabilities,
> maintaining zero npm dependencies with pure Rust, modern CSS3, and Vanilla ES6.

### 8.1 Batch 1: Controls & Selection (`Radio`, `SliderRange`, `ColorPicker`) · ✅
- `Radio`: Mutually exclusive selector supporting segmented pill buttons or classic radio styles.
- `SliderRange`: Dual-thumb slider for selecting bounded intervals `[min, max]`.
- `ColorPicker`: Native color palette selector with hex code entry and clickable swatches.
- **Files**: `components.rs`, `lib.rs`, `app.js`, `styles.css`, `COMPONENTS.md`, `tests/api_predict.rs`, `examples/forms.rs`.
- **Accepted when**: `cargo check --all-targets` & `cargo test` pass with zero warnings; `forms.rs` displays the "Phase 8 (Batch 1)" panel with full API tests and snapshots.

### 8.2 Built-in Showcase & CLI Runner (`App::showcase()`, `grio showcase`) · ✅
- `App::showcase()`: Pre-configured 5-tab application featuring all 30+ components with reactive event handlers.
- `grio showcase [--port <7860>]`: Standalone CLI command to launch the full gallery in 1 line.
- Minimal example in `examples/showcase.rs`.
- **Files**: `showcase.rs`, `lib.rs`, `grio-cli/src/main.rs`, `examples/showcase.rs`, `tests/api_predict.rs`.
- **Accepted when**: `test_showcase_boot_and_components` boots the showcase and validates the DOM for all widgets, the OpenAPI documentation, and `/api/predict`.

### 8.3 Batch 2: AI Vision & Media Annotation (`AnnotatedImage`, `ImageComparison`, `AudioRecorder`) · ✅
- `AnnotatedImage`: Normalized bounding box overlay (YOLO/SAM labels, confidence scores, accent colors).
- `ImageComparison`: Interactive before/after sliding curtain comparison (mouse and touch draggable).
- `AudioRecorder`: Direct microphone recorder with pulsing REC indicator, timer, and ASR/Whisper export.
- **Files**: `components.rs`, `lib.rs`, `app.js`, `styles.css`, `showcase.rs`, `COMPONENTS.md`, `tests/api_predict.rs`.
- **Accepted when**: `cargo check --all-targets` & `cargo test` pass; `test_lot2_vision_and_audio_integration` validates DOM rendering and `/api/predict`.

### 8.4 Batch 3: Progress Visual Variants (`Progress` : `bar`, `circle`, `pie`) · ✅
- Extended `Progress` with 3 visual variants: `.bar()` (default), `.circle()` (SVG arc ring), and `.pie()` (conic-gradient sector).
- Full real-time support via `ctx.progress(id, pct, label)`.
- **Files**: `components.rs`, `app.js`, `styles.css`, `showcase.rs`, `COMPONENTS.md`, `tests/api_predict.rs`.
- **Accepted when**: `test_progress_variants_bar_circle_pie` validates HTML output for all 3 variants and API responsiveness.

### 8.5 Batch 4: Specialized AI Evaluation (`HighlightedText`, `CodeDiff`, `Model3D`) · ✅
- `HighlightedText`: NLP/NER entity span visualizer with category chips and automatic color legend.
- `CodeDiff`: AI code refactoring comparative diff viewer with line numbers and `+`/`-` line additions/deletions.
- `Model3D`: Lightweight WebGL 3D mesh viewer supporting Wavefront OBJ files with orbit rotation and mouse zoom.
- **Files**: `components.rs`, `lib.rs`, `app.js`, `styles.css`, `showcase.rs`, `COMPONENTS.md`, `tests/api_predict.rs`.
- **Accepted when**: `cargo check --all-targets` & `cargo test` pass with 100% success; `test_lot4_specialized_components_integration` validates the DOM and `/api/predict`.

### 8.6 Batch 5: Robust `Html` Component & `window.grio` JavaScript Bridge · ✅
- `Html`: Embeds custom HTML, CSS, and scoped JavaScript without memory leaks or event drops.
- **Automatic Event Delegation**: Inner elements bearing `data-grio-action`, `data-grio-change`, or `data-grio-input` automatically dispatch directly to the server.
- **`window.grio` API Bridge**: Documented client object (`emit`, `on`, `get`, `snapshot`, `toast`) enabling custom scripts to communicate seamlessly with Rust handlers.
- **Files**: `app.js`, `components.rs`, `lib.rs`, `styles.css`, `showcase.rs`, `COMPONENTS.md`, `tests/api_predict.rs`.
- **Accepted when**: `test_html_custom_component_robustness` validates delegation and the WebSocket event lifecycle.

### 8.7 Batch 6: Geospatial OpenStreetMap (`Map`) · ✅
- `Map`: Interactive OpenStreetMap tile renderer, panning/zooming, SVG pins with popups, and radius circles.
- **Bi-directional event loop**: Clicking the map or pins emits geographic coordinates `{lat, lon, marker_id}` to Rust handlers; dynamic mutations via `ctx.set`.
- **Files**: `components.rs`, `lib.rs`, `app.js`, `styles.css`, `showcase.rs`, `COMPONENTS.md`, `tests/api_predict.rs`.
- **Accepted when**: `cargo check --all-targets` & `cargo test` pass; `test_map_openstreetmap_component` validates DOM rendering and parameters.

---

## Phase 9 — Enterprise Layouts, Big Data & Advanced Modalities · [P0]

> **Under Development / Planned.** Structured into coherent functional batches to build 
> all necessary primitives before assembling the Flagship Enterprise Showcase.

### 📦 Lot 1 — Layout, Navigation & Responsive Structure · ✅
- **9.1 Sliding Drawers (`Drawer`)**: Slide-in panels (`left`, `right`, `bottom` bottom-sheet, `top`), blurred backdrop overlay, click/`Esc` dismissal, bi-directional `ctx.set` open/close binding.
- **9.2 Declarative Multi-Page Routing (`MultiPage`)**: Declarative `app.page(route, title, builder)`, client-side SPA routing (`HTML5 History API`), auto-generated responsive sidebar & mobile drawer.
- **9.3 Zero-Config Responsive Engine**: Standardized breakpoints (`< 640px`, `640–1024px`, `> 1024px`), 44px touch targets, horizontally scrollable tabs, and dense widget fullscreen toggles.
- **Files**: `components.rs`, `app.rs`, `server.rs`, `app.js`, `styles.css`, `COMPONENTS.md`, `examples/multi_page_drawer.rs`, `tests/api_predict.rs`.

### 📦 Lot 2 — Rich Editing, Dynamic Slots & Big Data Grid · ✅
- **9.4 Markdown & Rich-Text Micro-Editor (`RichText`)**: Minimalist formatting toolbar (bold, italic, headings, lists, code, links), keyboard shortcuts (`Ctrl+B`, `Ctrl+I`, `Ctrl+K`), clean Markdown generation and preview mode.
- **9.5 Interactive Data Editor (`DataEditor`)**: Typed columns (bool checkboxes, text, numbers, dropdowns), in-place cell editing, row/col addition & deletion, spreadsheet TSV/CSV copy-paste support.
- **9.6 Dynamic Component Slots (`DynamicContainer` / Reactive Slots)**: Server-driven component injection/replacement at runtime (`ctx.append_component`, `ctx.replace_children`, `ctx.clear_container`).
- **9.7 Universal Visibility (`visible: bool`)**: Universal dynamic show/hide property mutation with zero-flicker CSS flow toggling (`ctx.set_visible`).
- **Files**: `components/forms.rs`, `components/data.rs`, `components/layout.rs`, `app.rs`, `context.rs`, `server.rs`, `assets/js/forms.js`, `assets/js/data.js`, `assets/js/core.js`, `styles.css`, `COMPONENTS.md`, `examples/rich_data_slots.rs`, `tests/api_predict.rs`.

### 📦 Lot 3 — Documents & Visual Workflows (RAG & Node Pipelines) · ✅
- **9.8 Document & PDF Viewer (`Pdf`)**: In-app document viewer, page navigation, zoom, and dynamic RAG/OCR citation highlights (`.highlight(...)`).
- **9.9 Node-Based Workflow Editor (`NodeGraph` à la ComfyUI)**: Draggable nodes, typed input/output sockets, bezier curve connectors, node status indicators (`idle`, `running`, `success`, `error`), and asynchronous topology change events.
- **Files**: `components/media.rs`, `components/special.rs`, `assets/js/media.js`, `assets/js/special.js`, `styles.css`, `COMPONENTS.md`, `showcase.rs`, `tests/api_predict.rs`.

---

## Flagship Showcase Application & Release · 🎯 [P0] · ✅

### Enterprise IT Service Desk & AI Support Copilot (`examples/it_desk.rs`) · ✅
A complete, production-grade enterprise demonstration app showcasing real-world multimodal AI and data workflows in pure Rust:
- **💬 Conversational AI Copilot (`Chatbot`)**: Real-time token streaming diagnosing user IT issues, connected to local **LM Studio** (`http://localhost:1234/v1`), with seamless local fallback.
- **📦 Service Catalog (`DataEditor`)**: Interactive service catalog with typed columns, active SLA rules, category filtering, and direct spreadsheet copy-paste.
- **⚙️ Sliding Inspector & User Diagnostics (`Drawer`)**: Slide-in side panel for requester telemetry, network diagnostics, and emergency procedures.
- **📝 Rich Incident Reporting (`RichText`, `File`)**: Formatted incident descriptions with Markdown toolbar and screenshot/log file uploads.
- **📋 Reactive Ticket Management (`DynamicContainer`)**: Live hot-slot ticket injection upon submission with status badges and realtime updates.
- **📊 User Observability & Incident Analytics (`Metric`, `Plot`, `Progress`)**: KPI header metrics + monthly quota progress + 6-month historical activity chart.
- **📑 Knowledge Base Citations (`Pdf`)**: Integrated document viewer with highlighted RAG runbooks.
- **Files**: `examples/it_desk.rs`, `Cargo.toml`, `assets/js/media.js`, `assets/styles.css`.

---

## Phase 10 — High-Throughput Big Data, Virtual Grid & Data Streams · ✅

> **Delivered.** Sub-millisecond manipulation and high-speed streaming of massive datasets 
> (100k+ rows) with 60 FPS DOM virtualization windowing, instant client-side search, and native CSV export.

### 📦 Lot 1 — DOM Virtualization Engine (`DataEditor` & `Dataframe`) · ✅
- **10.1 High-Performance Virtual Scroll**: Dynamic viewport windowing rendering only ~25 visible rows across 100k+ rows at 60 fps with zero DOM overhead.
- **10.2 Chunked / Streaming Table Ingestion**: Real-time batch ingestion via `ctx.append_rows(id, batch)`.
- **10.3 Quick Search & Column Filtering**: Client-side Instant Full-Text Filter (< 3ms) and native CSV download button.
- **Files**: `assets/js/data.js`, `crates/grio/src/components/data.rs`, `crates/grio/src/context.rs`, `assets/styles.css`.

### 📦 Lot 2 — Big Data Showcase (`examples/snowflake_stream.rs`) · ✅
- **10.4 Snowflake / DuckDB Analytical Stream Demo**: Real-time streaming analytics dashboard rendering hundreds of thousands of transactions with burst buttons (500 rows / 5,000 rows).
- **Files**: `examples/snowflake_stream.rs`, `tests/api_predict.rs`.

---

## Phase 11 — Native LLM Connectors & AI Agent Hub Gateway · ✅

> **Delivered.** Universal multi-provider LLM connector module (`grio::ai`), live HTTP SSE token streaming, and real-time observability metrics.

### 📦 Lot 1 — Native LLM Connectors (`grio::ai`) · ✅
- **11.1 Multi-Engine Connector Module**: Preset configurations for **LM Studio** (`Llm::lm_studio()`), **Ollama** (`Llm::ollama()`), and **OpenAI / vLLM** (`Llm::openai(key)`).
- **11.2 Standard Payload Generation & SSE Parsing**: Zero-copy JSON payload construction and line-by-line SSE stream decoding.
- **Files**: `crates/grio/src/ai.rs`, `crates/grio/src/lib.rs`.

### 📦 Lot 2 — AI Agent Hub Gateway (`examples/agent_hub.rs`) · ✅
- **11.3 Multi-Engine Gateway Demo**: Live token streaming connected to local LM Studio (`localhost:1234`) with instant fallback diagnostics.
- **11.4 Live LLM Telemetry**: Real-time calculation of generation speed (`tok/s`), Time to First Token latency (`TTFT in ms`), and total tokens processed.
- **11.5 Model Context Tools**: Interactive MCP tool calling registry (`fetch_db_schema`, `search_vector_rag`, `execute_code`).
- **Files**: `examples/agent_hub.rs`, `Cargo.toml`.

---

## Phase 12 — Curated Themes, Design Tokens & Live Hot-Swapping · ✅

> **Delivered.** Modern curated design presets and real-time CSS theme hot-swapping over WebSockets without page reload.

### 📦 Lot 1 — Curated Theme Presets & Design Tokens · ✅
- **12.1 Built-in Theme Presets**: `Theme::tokyo_night()`, `Theme::nord()`, `Theme::cyberpunk()`, `Theme::catppuccin_mocha()`, and `Theme::corporate()`.
- **12.2 Design Tokens**: Typography font injection, border radius tokens, and glassmorphism styling.
- **12.3 Live Theme Hot-Swapping (`ctx.set_theme`)**: Dynamic CSS variable hot-swapping across connected browser clients in real-time.
- **Files**: `crates/grio/src/app.rs`, `crates/grio/src/context.rs`, `crates/grio/src/assets/js/core.js`, `crates/grio/examples/theme_studio.rs`.

---

## Phase 13 — Standalone Desktop Mode, CLI Scaffolder & Dockerization · ✅

> **Delivered.** Native frameless desktop app launcher, enhanced CLI project generator, and lightweight multi-stage Docker container.

### 📦 Lot 1 — Desktop Standalone Mode (`App::launch_desktop`) · ✅
- **13.1 Frameless Desktop Window**: `app.launch_desktop(addr)` launches the server and opens a dedicated native standalone application window without browser URL bars.
- **13.2 Cross-Platform Window Management**: Direct Windows Edge/Chrome app mode, macOS `open`, and Linux `xdg-open` handlers.
- **Files**: `crates/grio/src/app.rs`, `crates/grio/examples/desktop_app.rs`.

### 📦 Lot 2 — CLI Project Scaffolder & Containerization (`grio-cli`) · ✅
- **13.3 CLI Scaffolding Templates**: `grio new <name> --template <agent|bigdata|chatbot|vision|greet>`.
- **13.4 Lightweight Docker Generator**: `grio docker <name>` generating an ultra-compact ~15MB Alpine multi-stage Docker image with zero Node/NPM dependencies.
- **Files**: `crates/grio-cli/src/main.rs`.

---

# 🔮 Future Developments & Vision (v0.2.0+)

## Phase 14 — Binary Zero-Copy Data Pipelines (Apache Arrow & WebGL Accelerators) · ✅

> **Delivered.** Direct zero-copy binary streaming (`ctx.append_f32_points`, `ctx.append_binary`), WebGL2 hardware-accelerated time-series engine rendering 1,000,000+ points at 60 FPS, and OLAP multidimensional dynamic pivot tables (`PivotTable`).

### 📦 Lot 1 — Zero-Copy Binary WebSocket Protocol & Hardware WebGL Engine · ✅
- **14.1 Zero-Copy Binary Streaming (`ctx.append_f32_points`, `ctx.append_binary`)**: Direct transmission of typed `f32` buffers and binary frames over WebSockets without JSON serialization bottlenecks.
- **14.2 WebGL GPU Time-Series Plotting (`WebGlPlot`)**: Hardware-accelerated WebGL2 rendering engine featuring vertex shaders, dynamic VBO buffers, live FPS telemetry, auto-scaling and neon cyberpunk palettes.
- **14.3 Interactive Pivot Tables & OLAP Slicers (`PivotTable`)**: Multidimensional cube slicing with real-time in-browser dynamic aggregations (Sum, Mean, Count, Min, Max) across rows and columns.
- **Files**: `crates/grio/src/components/data.rs`, `crates/grio/src/context.rs`, `crates/grio/src/assets/js/core.js`, `crates/grio/src/assets/js/data.js`, `crates/grio/src/assets/styles.css`, `crates/grio/examples/bigdata_accelerator.rs`, `crates/grio/tests/api_predict.rs`.

---

## Phase 15 — Enterprise Security, Multi-Tenancy & Model Context Protocol (MCP) · ⏳ [In Progress]

### 📦 Lot 1 — Model Context Protocol (MCP) Server Endpoint (`/mcp/v1`) · ✅
- **15.1 Official Model Context Protocol (MCP) Server Endpoint**: Built-in `/mcp/v1` and `/mcp/tools` server allowing Claude Desktop, Cursor, and Windsurf to automatically discover and invoke `grio` tools, inspect input schemas, and execute pipelines via standard JSON-RPC 2.0.
- **Files**: `crates/grio/src/mcp.rs`, `crates/grio/src/app.rs`, `crates/grio/src/server.rs`, `crates/grio/src/lib.rs`, `crates/grio/examples/mcp_agent_server.rs`, `crates/grio/tests/api_predict.rs`.

### 📦 Lot 2 — Enterprise Auth & Desktop Packaging · ⏳ [In Progress]
- **15.2 Enterprise Auth & OIDC / OAuth2**: Turnkey SSO integration (GitHub, Google, Keycloak, Okta) with role-based component access control (RBAC).
- **15.3 Sandboxed WebAssembly Plugin Engine (`WasmPlugin`) · ✅**: Safe execution of third-party user plugins without security risks, strict memory & fuel sandboxing (`SandboxLimits`), dynamic capability negotiation, universal extensible ABI, and example demonstrators.
  - **Files**: `crates/grio/src/wasm.rs`, `crates/grio/src/context.rs`, `crates/grio/src/app.rs`, `crates/grio/src/server.rs`, `crates/grio/src/lib.rs`, `crates/grio/examples/wasm_plugins.rs`, `PLUGINS.md`, `crates/grio/tests/api_predict.rs`.
- **15.4 Native Tauri v2 Desktop Bundler**: Single-command creation of signed `.msi`, `.dmg`, and `.deb` desktop installers.

---

## Roadmap Conventions

1. One task = one checked box + an entry in `README.md`.
2. Every item lists affected files before starting.
3. Criteria *Accepted when* serve as the acceptance tests.
4. Always ensure `cargo check -p grio --all-targets` and `cargo test --all-targets` have zero errors and zero warnings.