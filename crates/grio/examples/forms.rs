//! Phase 4 — Rich widgets: Checkbox, Dropdown (multiple/free-text), Date/Time,
//! editable Dataframe, SVG Plot, Gallery, drag-sort list, highlighted Code and
//! a server-side File Explorer — plus Phase 7 utilities: Number, Label, JSON,
//! Timer, File upload and DownloadButton.
//!
//! Run: `cargo run -p grio --example forms`

fn main() -> grio::Result<()> {
    use grio::*;

    App::new("Widgets · grio")

        .on_click("bar", |ctx| {
            ctx.set("chart", chart(1.0, "bar"));
            ctx.alert(AlertLevel::Success, "bar chart displayed");
            Ok(())
        })
        .on_click("line", |ctx| {
            ctx.set("chart", chart(0.0, "line"));
            ctx.alert(AlertLevel::Info, "line chart displayed");
            Ok(())
        })
        .on_click("shots", |ctx| {
            let idx = ctx.event().and_then(|e| e.d.as_ref().and_then(|d| d.as_u64())).unwrap_or(0);
            ctx.alert(AlertLevel::Info, format!("image #{} selected", idx + 1));
            Ok(())
        })
        .on_change("ex", |ctx| {
            let path = ctx.event().and_then(|e| e.d.as_ref().and_then(|d| d.as_str()).map(String::from));
            ctx.alert(AlertLevel::Info, format!("file selected: {}", path.unwrap_or_default()));
            Ok(())
        })
        .on_change("photo", |ctx| {
            if let Ok(v) = ctx.get::<serde_json::Value>("photo") {
                let n = v.get("layers").and_then(|l| l.as_array()).map(|a| a.len()).unwrap_or(0);
                let mask = v.get("mask").and_then(|m| m.as_str()).unwrap_or("");
                ctx.alert(AlertLevel::Info, format!("edit: {} layer(s), mask {} bytes — ready for inpainting", n, mask.len()));
            }
            Ok(())
        })
        .on_change("clock", |ctx| {
            let t = ctx.event().and_then(|e| e.d.as_ref().and_then(|d| d.as_f64())).unwrap_or(0.0);
            ctx.set("uptime", format!("{:.1} s", t));
            Ok(())
        })
        .on_click("gencsv", |ctx| {
            let n = ctx.get::<f64>("numcpu").unwrap_or(3.0) as usize;
            let mut csv = String::from("color,n\n");
            for i in 0..n {
                csv.push_str(&format!("color_{i},{}\n", (i * 37) % 256));
            }
            use base64::Engine as _;
            let b64 = base64::engine::general_purpose::STANDARD.encode(csv.as_bytes());
            ctx.set("dl", serde_json::json!({ "b64": b64, "mime": "text/csv" }));
            ctx.alert(AlertLevel::Success, format!("CSV generated ({} rows) — ready to download", n));
            Ok(())
        })
        .on_submit(|ctx| {
            let mut s = String::from("=== Input summary ===\n");
            if let Ok(b) = ctx.get::<bool>("deal") { s.push_str(&format!("✓ agreement: {b}\n")); }
            if let Ok(m) = ctx.get::<String>("model") { s.push_str(&format!("model: {m}\n")); }
            if let Ok(tags) = ctx.get::<Vec<String>>("tags") { s.push_str(&format!("tags: {}\n", tags.join(", "))); }
            if let Ok(d) = ctx.get::<String>("due") { s.push_str(&format!("due date: {d}\n")); }
            if let Ok(t) = ctx.get::<String>("alarm") { s.push_str(&format!("alarm: {t}\n")); }
            if let Ok(v) = ctx.get::<serde_json::Value>("df") {
                let rows = v.get("data").and_then(|x| x.as_array()).map(|a| a.len()).unwrap_or(0);
                s.push_str(&format!("table: {rows} row(s)\n"));
            }
            if let Ok(o) = ctx.get::<Vec<String>>("prio") {
                s.push_str(&format!("priorities: {}\n", o.join(" > ")));
            }
            if let Ok(code) = ctx.get::<String>("editor") {
                s.push_str(&format!("edited code: {} characters\n", code.len()));
            }
            if let Ok(g) = ctx.get::<Vec<String>>("shots") {
                s.push_str(&format!("gallery: {} image(s)\n", g.len()));
            }
            if let Ok(p) = ctx.get::<String>("ex") {
                s.push_str(&format!("explorer file: {p}\n"));
            }
            if let Ok(n) = ctx.get::<f64>("numcpu") {
                s.push_str(&format!("CSV rows: {n}\n"));
            }
            if let Ok(doc) = ctx.get::<serde_json::Value>("jsondoc") {
                if !doc.is_null() {
                    let model = doc.get("model").and_then(|x| x.as_str()).unwrap_or("?");
                    s.push_str(&format!("JSON: model {} · {}\n", model, doc));
                }
            }
            if let Ok(files) = ctx.get::<Vec<serde_json::Value>>("docs") {
                s.push_str(&format!("attachments: {} file(s)\n", files.len()));
            }
            ctx.set("summary", s);
            Ok(())
        })

        .subtitle("Ten configurable widgets, in the spirit of Gradio — everything is builder-configurable and readable from the handlers.")
        .item(Markdown::new("intro").text(
            "# Widgets\n\nEach component exposes its options through builder methods, its value in the input snapshot, and the classic `change` / `click` / `submit` events.\n\n**Modular layout**: `WithLayout::new(brick).width/.height/.scale/.min_width` wraps any component; `row` / `column` / `grid` / `panel` groups accept the same settings through their builder.",
        ))

        // --- Inputs ---
        .panel("Inputs", |p| {
            p.grid(2, |g| {
                g.item(Checkbox::new("deal").label("I accept the terms").value(true));
                g.item(Dropdown::new("model")
                    .label("Model")
                    .choices(&[("gpt-4o", "GPT-4o"), ("claude-3.5", "Claude 3.5"), ("mistral", "Mistral")])
                    .value("claude-3.5"));
            });
            p.item(Dropdown::new("tags")
                .label("Tags (multiple, free text)")
                .choices_str(&["rust", "ui", "web"])
                .multiple(true)
                .value_list(&["rust", "web"])
                .allow_custom(true));
            p.grid(2, |g| {
                g.item(DatePicker::new("due").label("Due date").min("2026-01-01").max("2026-12-31").value("2026-08-19"));
                g.item(TimePicker::new("alarm").label("Alarm").value("09:30"));
            });
            p.row(|r| {
                r.item(WithLayout::new(Button::new("bar").label("Bars").secondary()).scale(1));
                r.item(WithLayout::new(Button::new("line").label("Lines").secondary()).scale(1));
            });
        })

        // --- Data & Code ---
        .panel("Data & Code", |p| {
            p.item(WithLayout::new(Dataframe::new("df")
                .label("Cart (editable)")
                .headers(&["Product", "Quantity", "Price"])
                .data(&serde_json::json!([
                    ["Apples", 3, 2.5],
                    ["Milk", 2, 1.1],
                    ["Bread", 1, 1.8],
                ]))
                .interactive(true)
                .addable(true)
                .sortable(true))
                .width(520));
            p.row(|r| {
                r.item(SortableList::new("prio")
                    .label("Priorities (drag & drop)")
                    .items(&[("p1", "Fast"), ("p2", "Complete"), ("p3", "Buffer")]));
                r.item(WithLayout::new(Code::new("editor")
                    .label("Rust editor")
                    .language("rust")
                    .value("fn main() {\n    let msg = \"hello grio\";\n    println!(\"{msg}\");\n}\n")
                    .interactive(true)
                    .lines(true))
                    .height(220));
            });
            p.item(Output::new("summary").label("Server summary"));
        })

        // --- Media ---
        .panel("Media", |p| {
            p.row(|r| {
                r.item(WithLayout::new(Gallery::new("shots").label("Gallery (click = index)").columns(3).interactive(true)).width(340));
                r.item(WithLayout::new(Plot::new("chart")
                    .label("SVG chart")
                    .variant("line")
                    .title("Class attendance")
                    .xlabel("session")
                    .ylabel("students"))
                    .height(300));
            });
        })

        // --- Server files ---
        .panel("Server files", |p| {
            p.min_width(400);
            p.item(Explorer::new("ex")
                .label("Explorer (project root, *.rs)")
                .root(".")
                .pattern("*.rs"));
            p.item(Code::new("pretty")
                .label("Generated code (read-only)")
                .language("rust")
                .value("// e.g. the output of a code generator\nlet out = compile(source);\n")
                .output()
                .lines(true));
        })

        // --- Photo editing ---
        .panel("Photo editing (layers → inpainting mask)", |p| {
            p.min_width(500);
            p.item(ImageEditor::new("photo")
                .label("Brush, eraser, shapes, crop, rotate, filters, undo/redo")
                .layers(2)
                .value(""));
            p.item(Markdown::new("photo_note").text(
                "- **Brush/Eraser/Shapes** draw on the active layer; **zoom** (wheel) and **Pan** (✋) navigate.\n- **Crop** trims, **↻ Rotate** turns, **Filters** adjusts the background.\n- Each gesture sends `{image, layers, mask}` to the server — the **mask** (white on black) marks the areas to repaint (**inpainting**).",
            ));
        })

        // --- Phase 7: Files & Utilities ---
        .panel("Phase 7 — Files & Utilities", |p| {
            p.min_width(460);
            p.row(|r| {
                r.item(Number::new("numcpu").label("Rows to generate").value(3.0).min(0.0).max(16.0).step(1.0));
                r.item(Label::new("uptime").label("Session uptime").value("0.0 s").variant("success"));
            });
            p.item(Json::new("jsondoc")
                .label("JSON parameters (live-validated editor)")
                .value(serde_json::json!({ "model": "qwen", "warmup": 3, "top_k": 40, "tags": ["rust", "grio"] })));
            p.item(Timer::new("clock").label("Timer (tick every 3 s)").interval(3.0));
            p.item(File::new("docs")
                .label("Attachments (images / PDF, max 4 MB)")
                .types(&["image/*", "application/pdf"])
                .max_size(4 * 1024 * 1024)
                .multiple(true));
            p.row(|r| {
                r.item(WithLayout::new(Button::new("gencsv").label("Generate CSV").secondary()).scale(1));
                r.item(DownloadButton::new("dl").label("Download CSV").filename("rapport.csv"));
            });
        })

        .panel("About", |p| {
            p.item(Markdown::new("about").text(
                "- **checkbox/dropdown/date/time/dataframe/list/code** send their value in the `change`, retrieved with `ctx.get`.\n- **plot**: SVG drawn in pure JS, fed by `ctx.set` (where `chart(n)` is a small sinusoid generator).\n- **gallery**: uploaded images are data URLs; a click emits the index in `d`.\n- **explorer**: lists files on the **server** machine via `/api/explore` (bounded root + `*.rs` filter).\n- **imageeditor**: canvas editing (RGBA layers) → white/black **mask** usable for inpainting server-side.",
            ));
        })

        .launch("0.0.0.0:7860")
}

/// Series displayed in the chart (`k` shifts the phase; `variant` is
/// `"line"`, `"bar"` or `"scatter"`).
fn chart(k: f64, variant: &str) -> serde_json::Value {
    use serde_json::json;
    let labels: Vec<String> = (1..=8).map(|i| format!("S{i}")).collect();
    let a: Vec<f64> = (0..8).map(|i| 10.0 + 8.0 * ((i as f64 + k) / 2.0).sin()).collect();
    let b: Vec<f64> = (0..8).map(|i| 6.0 + 5.0 * ((i as f64 + 1.0 + k) / 2.0).cos()).collect();
    json!({
        "variant": variant,
        "labels": labels,
        "series": [
            { "name": "promo A", "data": a, "color": "#6366f1" },
            { "name": "promo B", "data": b, "color": "#f59e0b" },
        ]
    })
}