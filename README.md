# ⚙️ grio — A Blazing Fast, Pure Rust Alternative to Gradio
<img width="1291" height="1170" alt="image" src="https://github.com/user-attachments/assets/cb03e11b-4856-413f-9ca8-facab0de9e4f" />

<img width="1934" height="1189" alt="image" src="https://github.com/user-attachments/assets/ad39a3aa-a124-4523-b253-409518b12347" />

<img width="774" height="487" alt="image" src="https://github.com/user-attachments/assets/963ed96f-20cd-4452-99b7-4bb2bfd7a6a4" />

<img width="555" height="275" alt="image" src="https://github.com/user-attachments/assets/acef02f0-4818-4892-947d-c237dd8c7cc9" />

**grio** is a declarative web framework for Rust AI, ML, and data applications. Define **components** and **event handlers** in pure Rust, and grio instantly serves:
1. **A modern, reactive real-time Web UI** (CSS3 + Vanilla JS, zero frontend dependencies, zero npm build).
2. **A full auto-generated REST API** (`/api/predict`, `/api/schema`, `/docs` Swagger UI, `/api/openapi.json`).
3. **Interactive Client Code Generator** (`⚡ Use via API` modal with ready-to-use Python, JavaScript, cURL, and MCP Tool snippets).
4. **Native i18n & Dark/Light Themes** out-of-the-box.

```
┌───────────────── Rust (Your Application) ─────────────────┐
│ App::new("⚙️ AI Studio")                                  │
│     .item(Text::new("prompt"))                            │
│     .item(Slider::new("steps").min(10).max(50))           │
│     .item(Chatbot::new("chat"))                           │
│     .on_click("generate", |ctx| { ... })                  │
│     .launch("127.0.0.1:7860")                             │
└──────────────────────────┬────────────────────────────────┘
                           ▼
              ┌─────────────────────────────┐
              │           ⚙️ grio           │
              ├─────────────────────────────┤
              │  UI    GET /                │   Page + Components (HTML5/CSS3/ES6)
              │        GET /assets/*        │   styles.css + app.js (Zero node_modules)
              │        WS  /ws              │   Bi-directional Real-Time Event Bus
              │  API   GET  /api/schema     │   Auto-generated Schema Manifest
              │        POST /api/predict    │   Unified REST Execution Pipeline
              │        GET  /docs           │   Interactive Swagger UI
              │        GET  /api/openapi    │   OpenAPI 3.0.3 Specification
              └─────────────────────────────┘
```

> 📖 **Full Component Reference**: See [COMPONENTS.md](COMPONENTS.md) for detailed APIs, parameters, and code examples for all 25+ built-in widgets.

---

## 🚀 Quick Start

Add `grio` to your `Cargo.toml`:

```toml
[dependencies]
grio = "0.1.0"
```

Create `examples/greet.rs`:

```rust
use grio::*;

fn main() -> grio::Result<()> {
    App::new("⚙️ Greet Demo")
        .subtitle("Built with grio · Native Rust Web Framework")
        .item(Text::new("name").label("Your Name").placeholder("World"))
        .item(Slider::new("intensity").label("Excitement Level").min(1.0).max(5.0).value(2.0))
        .item(Output::new("greeting").label("Response"))
        .on_submit(|ctx| {
            let name = ctx.get_str("name").unwrap_or_else(|_| "World".into());
            let intensity = ctx.get_f64("intensity").unwrap_or(1.0) as usize;
            let exclamations = "!".repeat(intensity);
            ctx.set("greeting", format!("Hello, {name} {exclamations}"));
            ctx.alert(AlertLevel::Success, "Greeting generated!");
            Ok(())
        })
        .launch("127.0.0.1:7860")
}
```

Run it with Cargo:

```bash
cargo run -p grio --example greet
# → ⚙️ grio running at http://127.0.0.1:7860
```

---

## 🎨 Built-in Examples

| Example | Command | Highlights |
|---|---|---|
| **Multimodal AI Studio** | `cargo run -p grio --example prompt_to_image` | Autoregressive LLM (Candle Qwen 2.5 7B GGUF) + SDXL Image Diffusion + Live Analytics Dataframe & Plots |
| **Chatbot** | `cargo run -p grio --example chatbot` | Conversational Chatbot widget with token-by-token streaming |
| **Media & Vision** | `cargo run -p grio --example media` | Image, Audio (live mic streaming), Video (live camera streaming) |
| **Rich Forms & Controls** | `cargo run -p grio --example forms` | Sliders, Checkboxes, Dropdowns, Date/TimePickers, Dataframes, Plots, Code Editors |
| **Grid & Containers** | `cargo run -p grio --example grid` | Responsive Grids, Rows, Columns, Panels, and Accordions |
| **Theming & Tabs** | `cargo run -p grio --example theme_and_tabs` | Multi-tab workflows, light/dark themes, brand accent customization |

---

## 📦 Project Architecture

```
d:\Projet\UI
├─ Cargo.toml                 Workspace root
├─ README.md                  Main documentation
├─ COMPONENTS.md              Comprehensive component reference
├─ ROADMAP.md                 Completed phases and feature checklist
└─ crates/
   ├─ grio                    Core framework crate
   │  ├─ src/
   │  │  ├─ lib.rs            Public API exports & crate documentation
   │  │  ├─ app.rs            App builder & event distribution engine
   │  │  ├─ components.rs     25+ Component implementations & traits
   │  │  ├─ context.rs        Handler Context API (get, set, append, alert, progress)
   │  │  ├─ events.rs         WireEvent & EventName model
   │  │  ├─ server.rs         Axum HTTP/WebSocket/REST server & OpenAPI engine
   │  │  └─ assets/           Embedded styles.css (CSS3) & app.js (Vanilla ES6)
   │  └─ examples/            Showcase examples (prompt_to_image, chatbot, greet...)
   └─ grio-cli                Standalone developer CLI tool
```

---

## 🛠️ Declarative Rust API

### Core Widgets

| Component | Kind | Role | Description | Key Builder Methods |
|---|---|---|---|---|
| `Text` | `text` | Input | Single-line or multi-line text input | `.label()`, `.value()`, `.placeholder()`, `.lines()`, `.interactive()` |
| `Slider` | `slider` | Input | Numeric range slider | `.label()`, `.min()`, `.max()`, `.step()`, `.value()`, `.unit()` |
| `Checkbox` | `checkbox` | Input | Boolean toggle checkbox | `.label()`, `.value()`, `.interactive()` |
| `Dropdown` | `dropdown` | Input | Select box (single, multiple, searchable) | `.choices()`, `.multiple()`, `.allow_custom()`, `.value()` |
| `DatePicker` / `TimePicker` | `date` / `time` | Input | Native ISO date (`yyyy-mm-dd`) & time pickers | `.label()`, `.min()`, `.max()`, `.value()` |
| `Chatbot` | `chatbot` | Output | Interactive conversation thread | `.bubble()`, `.placeholder()`, `.clear()` |
| `Output` | `output` | Output | Standard rendered text or JSON output | `.label()`, `.value()` |
| `Markdown` | `markdown` | Output | GitHub-flavored markdown renderer | `.text()`, `.render()` |
| `Metric` | `metric` | Output | Analytics KPI card with value & delta badge | `.label()`, `.value()`, `.delta()`, `.delta_color()`, `.unit()` |
| `Plot` | `plot` | Output | Pure SVG Charts (`line`, `bar`, `scatter`) | `.variant()`, `.title()`, `.xlabel()`, `.ylabel()`, `.colors()` |
| `Dataframe` | `dataframe` | In/Out | Interactive editable & sortable spreadsheet | `.headers()`, `.data()`, `.sortable()`, `.addable()`, `.interactive()` |
| `Gallery` | `gallery` | In/Out | Media visualizer with lightbox popup | `.label()`, `.columns()`, `.upload()`, `.interactive()` |
| `ImageEditor` | `imageeditor` | In/Out | Full canvas editor (draw, mask, crop, filters) | `.brush()`, `.crop()`, `.filters()`, `.layers()` |
| `Code` | `code` | In/Out | Syntax-highlighted code editor | `.language()`, `.lines()`, `.theme()`, `.interactive()` |
| `Explorer` | `explorer` | Input | Server-side directory and file browser | `.root()`, `.pattern()` |

### Containers & Layouts

- **`Row`** (`b.row(|r| ...)`): Horizontal flexbox layout.
- **`Column`** (`b.column(|c| ...)`): Vertical stacking layout.
- **`Panel`** (`b.panel("Title", |p| ...)`): Card container with embossed border and header.
- **`Grid`** (`b.grid(cols, |g| ...)`): Responsive CSS grid layout (`repeat(auto-fit, minmax(...))`).
- **`Tabs`** (`b.tabs(|t| t.tab("Tab 1", ...))`): Client-switched multi-view workflow.
- **`Accordion`** (`b.accordion("Details", |a| ...)`): Collapsible `<details>` drawer.

---

## ⚡ Event Model & Context API

Event handlers are attached directly to the `App` builder and receive a rich `Context`:

```rust
App::new("Demo")
    .on_click("btn_submit", |ctx| {
        // Read typed inputs
        let prompt: String = ctx.get("prompt").unwrap_or_default();
        let temperature: f64 = ctx.get("temp").unwrap_or(0.7);

        // Send instant alert toast
        ctx.alert(AlertLevel::Info, "Processing inference...");

        // Stream real-time tokens to a Chatbot or Output
        for word in ["Generating ", "safe ", "Rust ", "tokens..."] {
            ctx.append("chat", word);
            std::thread::sleep(std::time::Duration::from_millis(50));
        }

        // Update progress bar
        ctx.progress("progress_bar", 1.0, "Done!");

        // Update outputs
        ctx.set("status_metric", json!({ "value": "100%", "delta": "Ready" }));

        Ok(())
    })
```

### Context Methods Summary

- **`ctx.get::<T>("id")` / `ctx.get_str("id")` / `ctx.get_f64("id")`**: Read deserialized input state.
- **`ctx.set("id", value)`**: Update component value across all connected clients and API response.
- **`ctx.append("id", text)`**: Stream incremental text/tokens directly to a component in real-time.
- **`ctx.progress("id", pct, "step")`**: Drive visual progress bars (`0.0` to `1.0`).
- **`ctx.alert(AlertLevel::Success, "msg")`**: Display color-coded toast notifications (`Info`, `Success`, `Warn`, `Error`).
- **`ctx.cancelled()`**: Check if the user cancelled or re-triggered the current job.

---

## 🔌 Automatic REST API & Client Code Generator

Every `grio` application automatically generates a complete REST API:

1. **`POST /api/predict`**: Execute the same pipeline as the UI programmatically.
2. **`GET /api/schema`**: Machine-readable JSON manifest of all inputs, outputs, and widgets.
3. **`GET /docs`**: Interactive Swagger UI documentation.
4. **`GET /api/openapi.json`**: OpenAPI 3.0.3 compliant specification.

### ⚡ Interactive `Use via API` Modal

Every `grio` page features a footer launcher **`⚡ Use via API`** providing copy-paste client code generated in real time:

- **🐍 Python (`requests`)**:
  ```python
  import requests

  res = requests.post("http://127.0.0.1:7860/api/predict", json={
      "inputs": { "prompt": "cyberpunk city", "steps": 25 }
  })
  print(res.json())
  ```
- **🟨 JavaScript (`fetch` / async-await)**:
  ```javascript
  const res = await fetch("http://127.0.0.1:7860/api/predict", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ inputs: { prompt: "cyberpunk city", steps: 25 } })
  });
  console.log(await res.json());
  ```
- **💻 cURL**:
  ```bash
  curl -X POST "http://127.0.0.1:7860/api/predict" \
    -H "Content-Type: application/json" \
    -d '{"inputs": {"prompt": "cyberpunk city", "steps": 25}}'
  ```
- **🤖 MCP Tool (Model Context Protocol)**: Ready-to-use JSON tool definition for LLM agents (Claude Desktop, Cursor, local agentic systems).

---

## 🌐 Internationalization (i18n)

`grio` includes built-in multilingual support. Users can switch languages dynamically from the **⚙️ Settings** modal:
- 🇬🇧 **English**
- 🇫🇷 **Français**
- 🇪🇸 **Español**
- 🇩🇪 **Deutsch**

All default component labels, file drops, empty states, and modal dialogs translate instantly without reloading the page.

---

## 📜 License

Licensed under the [MIT License](LICENSE).
Made with ❤️ for the Rust & AI Community.
