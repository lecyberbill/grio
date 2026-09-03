# `grio` Component Reference Manual

> **grio** is a lightweight, declarative web framework for Rust that lets you build interactive AI demos, dashboards, and evaluation interfaces in pure Rust with **zero frontend tooling** and automatic REST API generation.

---

## Table of Contents

- [Layout & Containers](#layout--containers)
  - [Multi-Page Apps & Navigation](#multi-page-apps--navigation)
  - [Drawer / Offcanvas (Horizontal & Vertical Drawers)](#drawer--offcanvas-horizontal--vertical-drawers)
  - [App::row / Row](#approw--row)
  - [App::column / Column](#appcolumn--column)
  - [App::grid / Grid](#appgrid--grid)
  - [App::panel / Panel](#apppanel--panel)
  - [Tabs](#tabs)
  - [Accordion](#accordion)
- [Input Components](#input-components)
  - [Text](#text)
  - [RichText (Markdown & Rich Text Micro-Editor)](#richtext-markdown--rich-text-micro-editor)
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
- [Media Components (Vision, Audio & Documents)](#media-components-vision-audio--documents)
  - [Image & Webcam](#image--webcam)
  - [ImageEditor (Inpainting Masks)](#imageeditor-inpainting-masks)
  - [Pdf (Document Viewer & RAG Citations)](#pdf-document-viewer--rag-citations)
  - [Audio & Microphone](#audio--microphone)
  - [Video](#video)
- [Buttons & Actions](#buttons--actions)
  - [Button](#button)

---

## Layout & Containers

### Multi-Page Apps & Navigation
Define multi-page applications with declarative routing, automatic responsive sidebar navigation, and deep-linking support via HTML5 History API.

```rust
let mut app = App::new("AI Platform");

// Home Page / Dashboard
app.page("/", "📊 Overview", |p| {
    p.icon("chart-pie");
    p.item(Metric::new("tps").label("Throughput").value("64 tok/s"));
    p.item(Plot::new("benchmarks"));
});

// Chatbot Page
app.page("/chat", "💬 LLM Assistant", |p| {
    p.icon("chat-bubble");
    p.item(Chatbot::new("bot"));
});

// Diffusion Studio Page
app.page("/diffusion", "🎨 Image Studio", |p| {
    p.icon("paint-brush");
    p.item(ImageEditor::new("canvas"));
});
```

- **Features**:
  - Auto-generated collapsible navigation sidebar with responsive mobile drawer.
  - Client-side routing with URL deep-linking (`/`, `/chat`, `/diffusion`) without full page reloads.
  - Shared session state across pages.

---

### `Drawer` / Offcanvas (Horizontal & Vertical Sliding Drawers)
Sliding container supporting 4 directional placements: horizontal (`"left"`, `"right"`) and vertical (`"bottom"` / bottom-sheet, `"top"`). Ideal for property inspectors, advanced settings panels, mobile bottom drawers, and live console logs.

```rust
// 1. Declarative Drawer Builder
let drawer = Drawer::new("settings_drawer")
    .title("⚙️ Advanced Parameters")
    .placement("right") // "left" | "right" | "bottom" | "top"
    .size(380)          // Width in px (or height if bottom/top)
    .open(false)        // Closed by default
    .backdrop(true)     // Dimmed overlay background, clickable to dismiss
    .content(|d| {
        d.item(Slider::new("temp").label("Temperature").min(0.0).max(1.0).value(0.7));
        d.item(Slider::new("top_p").label("Top-P").min(0.0).max(1.0).value(0.9));
        d.item(Checkbox::new("stream").label("Enable Token Streaming").value(true));
    });

app.item(drawer);

// 2. Open / Close from event handlers
app.on_click("open_settings_btn", |ctx| {
    ctx.set("settings_drawer", true); // Opens the drawer
    Ok(())
});
```

- **Builder Methods**:
  - `.placement("left" | "right" | "bottom" | "top")`: Sliding edge origin.
  - `.size(u32)`: Drawer width (px) for horizontal placements or height (px) for vertical placements.
  - `.open(bool)`: Initial open state.
  - `.backdrop(bool)`: Displays a dimmed background backdrop (click or `Esc` closes drawer).
- **Dynamic Control**:
  - `ctx.set("settings_drawer", true / false)` opens or closes the drawer in real time.

---

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

### `RichText` (Markdown & Rich Text Micro-Editor)
Lightweight formatting text editor supporting Markdown/WYSIWYG styling: bold (`**B**`), italic (`*I*`), headings (`H`), bullet/numbered lists, inline/block code (`</>`), and hyperlinks. Perfect for incident descriptions, notes, and formatted prompts.

```rust
RichText::new("ticket_desc")
    .label("Issue Description & Steps to Reproduce")
    .placeholder("Describe the problem with formatting, steps, and error messages...")
    .toolbar(&["bold", "italic", "heading", "list", "code", "link"])
    .min_height(160)
    .value("**Error:** Unable to connect to `vpn.corp.local`.\n\n*Steps:*\n1. Start client\n2. Enter credentials")
```

- **Builder Methods**:
  - `.toolbar(&["bold", "italic", "heading", "list", "code", "link"])`: Select visible action tools.
  - `.min_height(u32)`: Minimum editor height in pixels.
  - `.placeholder(str)`: Hint text shown when empty.
- **Keyboard Shortcuts**: `Ctrl+B` (Bold), `Ctrl+I` (Italic), `Ctrl+K` (Link).
- **Data format**: Returns clean Markdown/plain-text `String` via `ctx.get::<String>("ticket_desc")?`.

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

### `Radio` (Radio Group & Pills)
Mutually exclusive selector with either segmented pill button appearance or traditional radio circles.

```rust
Radio::new("arch")
    .label("Architecture")
    .choices(&["transformer", "mamba", "diffusion"])
    .value("mamba")
    .style("pills") // "pills" or "radio"
    .direction("horizontal") // "horizontal" or "vertical"
```

- **Data format**: `String` via `ctx.get::<String>("arch")?` or `ctx.get_str("arch")?`.

---

### `SliderRange` (Interval / Dual-Thumb Slider)
Dual-thumb slider allowing selection of bounded ranges `[min_val, max_val]`.

```rust
SliderRange::new("confidence")
    .label("Confidence Range")
    .min(0.0)
    .max(1.0)
    .step(0.01)
    .value(0.20, 0.80)
    .unit("%")
```

- **Data format**: `(f64, f64)` or `[f64; 2]` via `ctx.get::<(f64, f64)>("confidence")?`.

---

### `ColorPicker`
Color selector featuring native palette picker, hex code input, and quick swatches.

```rust
ColorPicker::new("accent")
    .label("Highlight Color")
    .value("#6366f1")
    .presets(&["#6366f1", "#10b981", "#ef4444", "#f59e0b", "#000000", "#ffffff"])
```

- **Data format**: `String` via `ctx.get::<String>("accent")?`.

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

### `DataEditor` (Big Data, Typed Columns & Spreadsheet Editing)
Advanced interactive spreadsheet grid with typed columns (checkboxes, numbers, text, URLs), clipboard TSV/CSV copy-paste, in-place cell editing with visual status badges, and dynamic row/column additions.

```rust
DataEditor::new("data_grid")
    .label("Dataset Inspector & Annotator")
    .column_bool("active", "Active")
    .column_text("name", "Name").editable(true)
    .column_number("age", "Age").min(0.0).max(120.0)
    .column_text("homepage", "Homepage").as_link(true)
    .allow_add_rows(true)
    .allow_delete_rows(true)
    .data(my_records)
```

- **Big Data & Polars/Arrow Support**: Optimized for streaming and virtual scrolling of large tabular datasets directly from Rust.

---

### `DynamicContainer` (Reactive Slots / Runtime Component Injection)
Dynamic slot container whose children components can be added, replaced, or removed at runtime from server handlers without pre-declaring static graphs.

```rust
// Declared in layout
app.dynamic_container("dynamic_filters_slot");

// Mutated in server handler
app.on_click("add_filter_btn", |ctx| {
    let new_widget = Slider::new(format!("filter_{}", uuid)).label("Dynamic Threshold");
    ctx.append_component("dynamic_filters_slot", new_widget);
    Ok(())
});
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
### `Progress` (Progress Bar, Circular Ring, or Pie Chart)
Real-time progress indicator driven by `ctx.progress(id, fraction, label)`.
Supports 3 modern display variants:
- **`bar`** (default): Horizontal progress bar with animated gradient and completion glow.
- **`circle`** : Circular SVG ring gauge with centered percentage and status label.
- **`pie`** : Conic-gradient pie chart with percentage badge.

```rust
// 1. Classic Horizontal Bar
Progress::new("dl_bar")
    .label("Downloading model weights")
    .bar()

// 2. Circular SVG Ring Gauge
Progress::new("epoch_circle")
    .label("Epoch Progress")
    .circle()
    .size(96) // Diameter in pixels

// 3. Conic Pie Chart
Progress::new("vram_pie")
    .label("VRAM Allocation")
    .pie()
    .size(80)
```

- **Updating Progress in Handlers**:
```rust
ctx.progress("epoch_circle", 0.85, "Epoch 85/100 · Loss: 0.12");
```

---

## Utility Components (Phase 7)

### `Number`
Numeric field with min/max/step bounds and a ± stepper.

```rust
Number::new("epochs")
    .label("Training Epochs")
    .value(3.0)
    .min(0.0)
    .max(16.0)
    .step(1.0)
```

- **Data format**: `f64` via `ctx.get::<f64>("epochs")?`.

---

### `Label`
A Gradio-style output label with a semantic color badge.

```rust
Label::new("accuracy")
    .label("Accuracy")
    .value("97.4%")
    .variant("success")   // normal | success | warning | danger | off
```

- Update on the server with `ctx.set("accuracy", "98.1%")`.

---

### `Json`
Live-validated JSON **editor** (input, default) or pretty **viewer** (`.output()`); a valid edit emits a `change` carrying the parsed object.

```rust
Json::new("params").label("Parameters").value(json!({ "model": "qwen", "top_k": 40 }))
```

- **Data format**: `serde_json::Value` via `ctx.get::<serde_json::Value>("params")?`.

---

### `Timer`
Periodic clock (`gr.Timer` equivalent): emits a `change` every `interval` seconds.
The elapsed seconds are exposed in `ctx.event().d`, so `on_change("id")` runs on a schedule (refreshes, polling).

```rust
Timer::new("clock").label("Heartbeat").interval(2.0)
```

```rust
.on_change("clock", |ctx| {
    let t = ctx.event().and_then(|e| e.d.clone()).and_then(|d| d.as_f64()).unwrap_or(0.0);
    ctx.set("heartbeat", format!("{t:.1} s"));
    Ok(())
})
```

---

### `File`
Multi-file upload with drag & drop, MIME type filter, size limit, upload progress bar and a removable list.

```rust
File::new("attachments")
    .label("Attachments")
    .types(&["image/*", "application/pdf"])
    .max_size(4 * 1024 * 1024)
    .multiple(true)
```

- **Data format**: array of `{ name, size, mime, data_url }` objects via
  `ctx.get::<Vec<serde_json::Value>>("attachments")?`.

---

### `DownloadButton`
Server-triggered download: push a data URL (or a `{ b64, mime }` object) with `ctx.set` to activate the button; a click downloads it under the configured filename.

```rust
DownloadButton::new("report").label("Download CSV").filename("report.csv")
```

```rust
.on_click("generate", |ctx| {
    let csv = "a,b,c\n1,2,3\n".to_string();
    ctx.set("report", json!({ "b64": base64_encode(csv), "mime": "text/csv" }));
    Ok(())
})
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

### `Pdf` (Document Viewer & RAG Citations)
Interactive PDF document viewer and uploader supporting page navigation, zoom/pan controls, text selection, and dynamic bounding-box or span highlights for RAG citations and OCR inspections.

```rust
Pdf::new("contract_doc")
    .label("Source Document (RAG Analysis)")
    .value("https://example.com/document.pdf") // or Base64 data URL
    .page(2)
    .height(650)
    // Add citation highlight on page 2: [norm_x, norm_y, norm_w, norm_h, hex_color]
    .highlight(2, 0.12, 0.35, 0.76, 0.10, "#fbbf24")
    .interactive(true)
```

- **Builder Methods**:
  - `.page(usize)`: Set the initially displayed page (1-indexed).
  - `.highlight(page, x, y, w, h, color)`: Adds a normalized bounding box highlight over cited text.
  - `.height(u32)`: Viewport height in pixels.
- **Dynamic Updates**:
  - `ctx.set("contract_doc", json!({ "page": 4, "highlights": [...] }))`: Change page and jump to cited paragraph dynamically.

---

### `AnnotatedImage` (Object Detection & Bounding Boxes)
Displays an image with vector bounding boxes, class labels, and confidence tags (e.g. YOLO, SAM, RT-DETR).

```rust
AnnotatedImage::new("detection")
    .label("YOLOv11 Detections")
    .image("https://example.com/photo.jpg")
    .box_norm(0.12, 0.28, 0.72, 0.72, "person", Some(0.96), "#6366f1")
    .box_norm(0.65, 0.35, 0.95, 0.65, "clothing", Some(0.88), "#10b981")
```

- Update server-side via `ctx.set("detection", json!({ "image": "...", "boxes": [...] }))`.

---

### `ImageComparison` (Before / After Slider)
Interactive side-by-side comparison with a draggable curtain divider (ideal for Super-Resolution, Denoising, Colorization, and Upscaling).

```rust
ImageComparison::new("sr_eval")
    .label("Super-Resolution 4x Comparison")
    .before("https://example.com/lowres.jpg", "Original (Low-Res)")
    .after("https://example.com/highres.jpg", "Upscaled (ESRGAN)")
    .position(50.0)
```

---

### `AudioRecorder`
Dedicated direct microphone recorder with REC button, live pulsing animation, timer, and automatic export for Whisper / Speech-to-Text pipelines.

```rust
AudioRecorder::new("voice_prompt")
    .label("Record Audio Prompt")
    .max_duration(30.0)
```

- **Data format**: Data URL `String` (`data:audio/webm;base64,...`) readable via `ctx.get::<String>("voice_prompt")?`.

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

### `HighlightedText` (NLP / Named Entity Recognition)
Renders rich text with color-coded entity spans, label badges, and an automatic color legend bar.

```rust
HighlightedText::new("ner_output")
    .label("Named Entity Recognition (NER)")
    .segments(&[
        ("Google ", Some("ORG")),
        ("was founded by ", None),
        ("Larry Page ", Some("PER")),
        ("and ", None),
        ("Sergey Brin ", Some("PER")),
        ("at ", None),
        ("Stanford University", Some("LOC")),
    ])
    .color_map(&[("ORG", "#6366f1"), ("PER", "#10b981"), ("LOC", "#f59e0b")])
    .show_legend(true)
```

- **Update from server**: `ctx.set("ner_output", json!([["Paris", "LOC"], [" is nice.", null]]))`.

---

### `CodeDiff` (AI Code Refactoring & Diff Viewer)
Comparative code diff viewer with line additions (`+` green), deletions (`-` red), and line numbering.

```rust
CodeDiff::new("diff_view")
    .label("Refactoring Diff")
    .old_code("fn compute(x: f64) -> f64 {\n    x * 2.0\n}")
    .new_code("fn compute(x: f64) -> f64 {\n    x.mul_add(2.0, 1.0)\n}")
    .language("rust")
    .split_view(false)
```

---

### `Model3D` (3D Mesh & Generative 3D Viewer)
Lightweight 3D mesh viewer supporting Wavefront OBJ files with interactive orbit rotation and zoom controls (zero external dependencies).

```rust
Model3D::new("viewer_3d")
    .label("3D Generated Mesh")
    .value("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3") // raw OBJ string or data URL
    .clear_color("#1e293b")
    .interactive(true)
```

---

### `Html` (Custom HTML, CSS & Scoped JavaScript)
Embeds raw HTML, inline CSS, and executable `<script>` blocks with **robust event delegation** and a dedicated client bridge `window.grio`.

```rust
Html::new("my_custom_widget")
    .label("Custom Metric Dashboard")
    .value(r#"
        <div class="my-box">
            <button data-grio-action="click" data-grio-payload='{"action":"refresh"}'>Refresh</button>
            <input type="text" data-grio-change placeholder="Type something...">
        </div>
        <script>
            // Scoped execution: 'element' is the container, 'grio' is window.grio
            grio.on("my_custom_widget", (patch) => {
                console.log("Updated from Rust server:", patch);
            });
        </script>
    "#)
```

#### The `window.grio` JavaScript Bridge:
- `window.grio.emit(componentId, eventName, data)` : Dispatches an event directly to the Rust WebSocket loop.
- `window.grio.on(componentId, callback)` : Subscribes to real-time state updates sent by `ctx.set(id, ...)`.
- `window.grio.get(componentId)` : Reads the current value of any input component on the page.
- `window.grio.toast(message, level)` : Triggers native toaster alerts.

---

### `Map` (Interactive Geographic Map & OpenStreetMap)
Interactive geospatial map powered by OpenStreetMap tiles, supporting pan/zoom, custom SVG markers, radius circles, and click events with zero npm dependencies.

```rust
Map::new("fleet_map")
    .label("AI Fleet Tracking & Node Locations")
    .center(48.8566, 2.3522) // [latitude, longitude]
    .zoom(12) // Initial zoom level (1 to 19)
    // Add markers with custom popups and colors
    .marker(48.8584, 2.2945, "Compute Node A (Eiffel Tower)", Some("#6366f1"))
    .marker(48.8606, 2.3376, "Edge Node B (Louvre)", Some("#10b981"))
    // Add geographical radius circles (in meters)
    .circle(48.8566, 2.3522, 1500.0, Some("#f59e0b"))
    .height(420)
    .interactive(true)
```

- **Builder Methods**:
  - `.center(lat: f64, lon: f64)` : Sets the initial geographic center coordinates.
  - `.zoom(level: u8)` : Sets the initial zoom factor (`1` to `19`, default `12`).
  - `.marker(lat, lon, label, color)` : Adds an interactive pin with a tooltip and hex color.
  - `.marker_with_id(id, lat, lon, label, color)` : Adds a pin with an explicit ID for event identification.
  - `.circle(lat, lon, radius_meters, color)` : Adds a radius circle.
  - `.height(px: u32)` : Sets the viewport height in pixels.
- **Event Handling**:
  - Listen to clicks on the map or markers with `app.on_click("fleet_map", |ctx| { ... })`.
  - `ctx.event().unwrap().d` contains `{ "lat": 48.85..., "lon": 2.35..., "marker_id": "..." }`.
- **Dynamic Updates**:
  - Mutate map markers and position in real time using `ctx.set("fleet_map", json!({ "center": [48.85, 2.35], "markers": [...] }))`.

### `NodeGraph` (ComfyUI / Node-RED Interactive Workflow Editor)
Interactive node-based visual workflow editor (inspired by ComfyUI / Blender Nodes). Supports draggable nodes, input/output sockets, bezier spline connections, custom node parameters (sliders, dropdowns, inputs inside nodes), execution status indicators, and serialized graph export to Rust.

```rust
NodeGraph::new("pipeline_graph")
    .label("AI Diffusion & Pipeline Workflow")
    .height(600)
    // Define a Node prototype or instance
    .node("clip_loader", "CLIP Text Encode", |n| {
        n.category("loaders");
        n.input_socket("clip", "CLIP");
        n.output_socket("conditioning", "CONDITIONING");
        n.widget(Text::new("prompt").label("Prompt").value("A cinematic photo of a cyberpunk city"));
    })
    .node("sampler", "KSampler", |n| {
        n.category("sampling");
        n.input_socket("model", "MODEL");
        n.input_socket("positive", "CONDITIONING");
        n.input_socket("latent_image", "LATENT");
        n.output_socket("latent", "LATENT");
        n.widget(Slider::new("steps").label("Steps").min(1.0).max(100.0).value(25.0));
        n.widget(Slider::new("cfg").label("CFG Scale").min(1.0).max(20.0).value(7.5));
    })
    // Pre-connect sockets
    .connect("clip_loader", "conditioning", "sampler", "positive")
    .interactive(true)
```

- **Graph Execution & Events**:
  - `app.on_change("pipeline_graph", |ctx| { ... })`: Emits graph topology changes (nodes added, moved, connected, deleted).
  - Handlers can read the entire DAG topology via `ctx.get::<NodeGraphTopology>("pipeline_graph")?` to execute pipelines asynchronously in Rust.
  - Server can push node execution states (`ctx.set_node_status("pipeline_graph", "sampler", NodeState::Running)`).

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
