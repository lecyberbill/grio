# `grio` Component Reference Manual

> **grio** is a lightweight, declarative web framework for Rust that lets you build interactive AI demos, dashboards, and evaluation interfaces in pure Rust with **zero frontend tooling** and automatic REST API generation.

---

## Table of Contents

- [Layout & Containers](#layout--containers)
  - [App::row / Row](#approw--row)
  - [App::column / Column](#appcolumn--column)
  - [App::grid / Grid](#appgrid--grid)
  - [App::panel / Panel](#apppanel--panel)
  - [Tabs](#tabs)
  - [Accordion](#accordion)
- [Input Components](#input-components)
  - [Text](#text)
  - [Slider](#slider)
  - [Checkbox](#checkbox)
  - [Dropdown](#dropdown)
  - [DatePicker & TimePicker](#datepicker--timepicker)
  - [Dataframe](#dataframe)
  - [Code Editor](#code-editor)
  - [Explorer](#explorer)
- [Output & AI Interaction Components](#output--ai-interaction-components)
  - [Chatbot (LLMs & Agents)](#chatbot-llms--agents)
  - [Output](#output)
  - [Markdown](#markdown)
  - [Plot (SVG Charts)](#plot-svg-charts)
  - [Gallery](#gallery)
  - [Progress Bar](#progress-bar)
- [Media Components (Vision & Audio)](#media-components-vision--audio)
  - [Image & Webcam](#image--webcam)
  - [ImageEditor (Inpainting Masks)](#imageeditor-inpainting-masks)
  - [Audio & Microphone](#audio--microphone)
  - [Video](#video)
- [Buttons & Actions](#buttons--actions)
  - [Button](#button)

---

## Layout & Containers

### `App::row` / `Row`
Horizontal container laying out children side-by-side with wrapping and flex alignment.

```rust
app.row(|r| {
    r.item(Text::new("first_name").label("First Name"));
    r.item(Text::new("last_name").label("Last Name"));
})
```

- **Builder Methods**:
  - `.gap(f64)` : Spacing between items in pixels (default: `16.0`).
  - `.wrap(bool)` : Enable or disable flex wrapping (default: `true`).
  - `.align(str)` : Cross-axis alignment (`"start"`, `"center"`, `"end"`, `"stretch"`).
  - `.justify(str)` : Main-axis justification (`"start"`, `"center"`, `"end"`, `"space-between"`).

---

### `App::column` / `Column`
Vertical container stacking items sequentially.

```rust
app.column(|col| {
    col.item(Markdown::new("title").value("## Configuration"));
    col.item(Slider::new("temp").label("Temperature").min(0.0).max(1.0).value(0.7));
})
```

---

### `App::grid` / `Grid`
Responsive CSS grid container with automatic single-column adaptation on mobile devices.

```rust
app.grid(3, |g| {
    g.gap(20.0);
    g.item(Text::new("c1").label("Col 1"));
    g.item(Text::new("c2").label("Col 2"));
    g.item(Text::new("c3").label("Col 3"));
})
```

- **Builder Methods**:
  - `.columns(usize)` : Number of columns (e.g. `2`, `3`, `4`).
  - `.gap(f64)` : Uniform spacing in pixels.
  - `.gap_x(f64)` / `.gap_y(f64)` : Distinct horizontal and vertical gap in pixels.

---

### `App::panel` / `Panel`
A styled card container featuring a header title and padded content body.

```rust
app.panel("Model Settings", |p| {
    p.item(Slider::new("top_p").label("Top-P").min(0.0).max(1.0).value(0.9));
})
```

---

### `App::tabs` / `Tabs`
Client-side multi-tab container switching between views with nested rows, columns, grids, or panels.

```rust
app.tabs(|t| {
    t.tab("💬 LLM Chatbot", |b| {
        b.item(Chatbot::new("bot"));
        b.row(|r| {
            r.item(Text::new("prompt"));
            r.item(Button::new("send").primary());
        });
    })
    .tab("📊 Benchmarks", |b| {
        b.item(Plot::new("bench"));
    })
    .tab("⚙️ Config", |b| {
        b.item(Slider::new("temp").label("Temperature"));
    })
})
```

- **Builder Methods**:
  - `.selected(usize)` : Set initially active tab (0-indexed).
  - `.tab(label, |b| { ... })` : Add a tab with label and children (`b.item`, `b.row`, `b.column`, `b.grid`, `b.panel`).

---

### `Accordion`
Native collapsible `<details>` and `<summary>` sections.

```rust
let acc = Accordion::new("acc")
    .open(true) // Open first section by default
    .section("Advanced Parameters", |s| {
        s.item(Slider::new("repetition_penalty").min(1.0).max(2.0).value(1.1));
    });
app.item(acc)
```

---

## Input Components

### `Text`
Single-line or multi-line text input.

```rust
Text::new("prompt")
    .label("Prompt")
    .placeholder("Type your prompt here...")
    .lines(4) // Switches to <textarea>
    .value("Default text")
    .interactive(true)
```

- **Data format**: `String` via `ctx.get::<String>("prompt")?` or `ctx.get_str("prompt")`.

---

### `Slider`
Numeric range slider.

```rust
Slider::new("temperature")
    .label("Temperature")
    .min(0.0)
    .max(2.0)
    .step(0.05)
    .value(0.7)
```

- **Data format**: `f64` via `ctx.get::<f64>("temperature")?` or `ctx.get_f64("temperature")?`.

---

### `Checkbox`
Boolean toggle switch / checkbox.

```rust
Checkbox::new("stream")
    .label("Enable Streaming")
    .value(true)
```

- **Data format**: `bool` via `ctx.get::<bool>("stream")?`.

---

### `Dropdown`
Single or multi-selection menu.

```rust
Dropdown::new("model")
    .label("Select LLM Model")
    .options(vec!["Llama-3-8B", "Mistral-7B", "Gemma-2-9B"])
    .value("Llama-3-8B")
```

---

### `Dataframe`
Interactive, editable tabular spreadsheet.

```rust
Dataframe::new("dataset")
    .label("Input Samples")
    .headers(vec!["ID", "Prompt", "Score"])
    .data(vec![
        vec!["1", "Summarize article", "0.95"],
        vec!["2", "Extract keywords", "0.88"],
    ])
    .interactive(true)
```

---

### `Code` Editor
Syntax-highlighted code editor (Rust, Python, JS, HTML, JSON).

```rust
Code::new("script")
    .label("Python Evaluation Code")
    .language("python")
    .value("def evaluate(model, dataset):\n    return model.score(dataset)")
    .lines(true)
```

---

### `Explorer`
Safe server-side filesystem navigator bounded to a designated root folder.

```rust
Explorer::new("model_files")
    .label("Select Checkpoint (.safetensors)")
    .root("./models")
    .pattern("*.safetensors")
```

---

## Output & AI Interaction Components

### `Metric` (AI Observability & Benchmarks)
Prominent card component displaying performance metrics (Throughput, Latency TTFT, Memory VRAM, Accuracy) with color-coded delta variations.

```rust
Metric::new("tps")
    .label("Throughput")
    .value("54.2")
    .unit("tok/s")
    .delta("+14.8%")
    .delta_color("normal") // "normal" (green) | "inverse" (red) | "off" (neutral)
```

- **Dynamic Mutation**:
```rust
// Update with full JSON object
ctx.set("tps", serde_json::json!({
    "value": "58.6",
    "delta": "+22.4%"
}));

// Or simple scalar value
ctx.set("tps", "61.2");
```

---

### `Chatbot` (LLMs & Agents)
Rich conversational bubbles with Markdown rendering, syntax-highlighted code blocks, and real-time token streaming.

```rust
Chatbot::new("chat")
    .label("Local LLM Assistant")
    .height(480)
    .message("assistant", "Hello! How can I assist you today?")
```

- **Real-Time Token Streaming**:
```rust
app.on_click("send", |ctx| {
    let user_msg: String = ctx.get("prompt")?;
    let mut history: Vec<ChatMessage> = ctx.get("chat").unwrap_or_default();
    history.push(ChatMessage::user(&user_msg));
    history.push(ChatMessage::assistant("")); // Empty assistant placeholder
    ctx.set("chat", history);

    // Stream tokens chunk by chunk
    for token in generate_tokens(&user_msg) {
        ctx.append("chat", token);
    }
    Ok(())
})
```

---

### `Plot` (SVG Charts)
Native pure-SVG data visualization supporting line charts, categorical bar charts, and scatter plots.

```rust
Plot::new("eval_metrics")
    .label("Evaluation Metrics")
    .variant("bar") // "bar" | "line" | "scatter"
    .title("Model Accuracy per Task")
    .xlabel("Benchmark Task")
    .ylabel("Accuracy (%)")
```

- **Feeding Data**:
```rust
ctx.set("eval_metrics", serde_json::json!({
    "labels": ["MMLU", "GSM8K", "HumanEval", "MATH"],
    "series": [
        { "name": "Model A", "data": [72.4, 68.1, 55.0, 42.3] },
        { "name": "Model B", "data": [78.2, 74.0, 62.1, 48.9] }
    ]
}));
```

---

### `Gallery`
Grid of previewable images with click-to-select interaction.

```rust
Gallery::new("generated_images")
    .label("Diffusion Outputs")
    .columns(4)
```

---

## Media Components (Vision & Audio)

### `Image` & `ImageEditor`
Image display, drag-and-drop upload, webcam capture, and canvas retouching (brush, crop, masks for inpainting).

```rust
Image::new("photo")
    .label("Input Image")
    .source("webcam") // "upload" | "webcam" | "canvas"
```

- **Inpainting mask generation with `ImageEditor`**:
```rust
ImageEditor::new("editor")
    .label("Inpainting Editor")
    .layers(2) // Outputs { image, layers, mask } on change
```

---

### `Audio` (Speech-to-Text & Text-to-Speech)
Interactive audio player, file uploader, and real-time live microphone streaming via WebSocket.

- **Audio as Output (e.g. Text-to-Speech / TTS generation)**:
```rust
Audio::new("synth_audio")
    .label("Generated Voice")
    .output()
```
Pass a base64 Data URL or audio URI with `ctx.set("synth_audio", "data:audio/wav;base64,...")`.

- **Audio as Input (Upload / STT transcription)**:
```rust
Audio::new("speech_in")
    .label("Upload Audio File")
    .input()
```

- **Live Microphone Streaming (Chunk-by-chunk via WebSocket)**:
```rust
Audio::new("mic")
    .label("Live Microphone")
    .interactive(true)
    .live(true)
```
Listen to incoming audio chunks on the server with `app.on_stream("mic", |ctx| { ... })`.

---

### `Video` (Video Generation & Vision Models)
HTML5 video player, file uploader, and live camera feed streaming.

- **Video as Output (e.g. Text-to-Video generation / video synthesis)**:
```rust
Video::new("generated_video")
    .label("Generated Video Output")
    .output()
```
Pass a video Data URL or URL with `ctx.set("generated_video", "data:video/mp4;base64,...")`.

- **Video as Input (Upload for video analysis / classification)**:
```rust
Video::new("source_video")
    .label("Upload Video")
    .input()
    .interactive(true)
```

- **Live Camera Streaming**:
```rust
Video::new("webcam")
    .label("Live Webcam Feed")
    .output()
    .live(true)
```
Captures webcam frames and streams chunks to the server via WebSocket. Also supports transport lifecycle hooks (`on_play`, `on_pause`, `on_stop`).

---

## Themes & Visual Customization

`grio` features a fully integrated Dark/Light mode theme engine with interactive UI toggling, OS preference detection, and CSS customization.

```rust
// Custom Dark Mode with Indigo accent and rounded corners
App::new("My AI App")
    .theme(
        Theme::dark()
            .primary("#6366f1")
            .radius("12px")
            .font("Inter, sans-serif")
            .toggle(true) // Display dark/light toggle button in header
    )
```

- **Presets**:
  - `Theme::dark()` : Default sleek dark palette.
  - `Theme::light()` : Crisp, high-contrast light palette.
  - `Theme::system()` : Adapts dynamically to OS / browser preference.

---

## Full AI Showcase Examples

Explore complete, production-ready examples demonstrating real-world AI pipelines:

- **[Multimodal Prompt Engineer & SDXL Diffusion Studio](crates/grio/examples/prompt_to_image.rs)**:
  - 💬 **LiquidAI LFM-2.5 1.2B** for prompt crafting and token streaming.
  - 🎨 **Juggernaut-XL v9 / SDXL** diffusion pipeline with step progress, metric observability, and persistent `output_images/` disk gallery.
  - Run with: `cargo run -p grio --example prompt_to_image`

- **[Theme & Multi-Tab AI Benchmarks](crates/grio/examples/theme_and_tabs.rs)**:
  - Dark/Light mode customization, `Tabs` layout, and real-time `Metric` observability cards.
  - Run with: `cargo run -p grio --example theme_and_tabs`

---

## Summary & Quick Tips

1. **State Mutation**: Use `ctx.set(id, value)` to update any component value.
2. **Property Mutation**: Use `ctx.set_prop(id, "visible" | "label" | "disabled", value)` to dynamically alter component properties.
3. **Reactive Streaming**: Use `ctx.append(id, chunk)` for incremental text/chat streaming.
4. **Scoped Handlers**: Use `.flow(&["inputs..."], &["outputs..."])` for strict sandboxing of reads and writes.
