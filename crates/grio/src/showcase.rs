//! Native Showcase application generator showcasing all interactive components.

use crate::app::App;
use crate::components::*;
use crate::context::{AlertLevel, Context};
use crate::Result;

impl App {
    /// Creates a pre-configured showcase application
    /// demonstrating all interactive `grio` components.
    ///
    /// Ready to run in a single line:
    /// ```no_run
    /// grio::App::showcase().launch("127.0.0.1:7860")?;
    /// # Ok::<(), grio::Error>(())
    /// ```
    pub fn showcase() -> Self {
        App::new("Showcase · grio")
            .subtitle("Interactive gallery of all components — zero frontend dependencies")
            .max_width(1280)
            .run_label("Test Submission (Run)")

            // Reactive Handlers
            .on_click("btn_toast_info", |ctx| {
                ctx.alert(AlertLevel::Info, "This is an informational notification.");
                Ok(())
            })
            .on_click("btn_toast_success", |ctx| {
                ctx.alert(AlertLevel::Success, "Operation completed successfully!");
                Ok(())
            })
            .on_click("btn_toast_warn", |ctx| {
                ctx.alert(AlertLevel::Warn, "Warning: Inference threshold reached.");
                Ok(())
            })
            .on_click("btn_toast_error", |ctx| {
                ctx.alert(AlertLevel::Error, "Simulated compute error.");
                Ok(())
            })
            .on_click("btn_bot_stream", |ctx| {
                ctx.append("chat_demo", "Hello! I am the grio assistant. Here is a streamed token-by-token response over native WebSockets.");
                ctx.alert(AlertLevel::Success, "Message generated in Chatbot");
                Ok(())
            })
            .on_click("btn_prog_sim", |ctx| {
                ctx.progress("prog_bar", 0.75, "Loading model weights: 75%");
                ctx.progress("prog_circle", 0.85, "Epoch 85/100");
                ctx.progress("prog_pie", 0.60, "VRAM allocated: 60%");
                ctx.alert(AlertLevel::Info, "Progress gauges updated!");
                Ok(())
            })
            .on_click("sc_map", |ctx| {
                if let Some(ev) = ctx.event() {
                    if let Some(ref d) = ev.d {
                        ctx.alert(AlertLevel::Info, format!("Map clicked at coordinates: {d}"));
                    }
                }
                Ok(())
            })
            .on_click("sc_custom_html", |ctx| {
                ctx.alert(AlertLevel::Success, "'click' action received from custom HTML component (data-grio-action)!");
                Ok(())
            })
            .on_change("sc_custom_html", |ctx| {
                if let Ok(txt) = ctx.get::<String>("sc_custom_html") {
                    ctx.alert(AlertLevel::Info, format!("Input received from HTML component: \"{txt}\""));
                }
                Ok(())
            })
            .on_click("btn_gencsv", |ctx| {
                let n = ctx.get::<f64>("num_items").unwrap_or(5.0) as usize;
                let mut csv = String::from("id,value,status\n");
                for i in 1..=n {
                    csv.push_str(&format!("{i},{},ok\n", i * 42));
                }
                use base64::Engine as _;
                let b64 = base64::engine::general_purpose::STANDARD.encode(csv.as_bytes());
                ctx.set("dl_btn", serde_json::json!({ "b64": b64, "mime": "text/csv" }));
                ctx.alert(AlertLevel::Success, format!("Generated CSV export ({n} rows)!"));
                Ok(())
            })
            .on_change("sc_slider", |ctx| {
                let v = ctx.get::<f64>("sc_slider").unwrap_or(0.0);
                ctx.set("sc_slider_echo", format!("{:.2}", v));
                Ok(())
            })
            .on_change("sc_color", |ctx| {
                let c = ctx.get::<String>("sc_color").unwrap_or_default();
                ctx.set("sc_color_echo", format!("Active color: {c}"));
                Ok(())
            })
            .on_click("btn_inject_slot", |ctx| {
                let new_box = Output::new("slot_dyn_metric")
                    .label("Hot-Injected Component (WebSocket Slot)")
                    .value("✅ Dynamic node mounted in real time with zero page refresh.");
                ctx.append_component("sc_dynamic_slot", new_box);
                ctx.alert(AlertLevel::Success, "Component injected into DynamicContainer!");
                Ok(())
            })
            .on_click("btn_clear_slot", |ctx| {
                ctx.clear_container("sc_dynamic_slot");
                ctx.alert(AlertLevel::Info, "DynamicContainer cleared.");
                Ok(())
            })
            .on_click("btn_toggle_drawer", |ctx| {
                ctx.set_prop("sc_drawer", "open", true);
                ctx.alert(AlertLevel::Info, "Telemetry drawer opened.");
                Ok(())
            })
            .on_click("btn_sc_webgl_burst", |ctx| {
                let mut burst = Vec::with_capacity(20_000);
                for i in 0..20_000 {
                    let t = i as f32 * 0.02;
                    burst.push((t * 2.0).sin() * 0.8 + (t * 10.0).cos() * 0.25);
                }
                ctx.append_f32_points("sc_webgl", &burst);
                ctx.alert(AlertLevel::Success, "⚡ 20,000 points injectés sans copie dans le GPU WebGL2 !");
                Ok(())
            })
            .on_click("btn_sc_webgl_stream", |ctx| {
                ctx.alert(AlertLevel::Info, "Streaming haute fréquence actif (2 000 pts/sec)...");
                for batch in 0..20 {
                    if ctx.cancelled() {
                        break;
                    }
                    let mut chunk = Vec::with_capacity(250);
                    for i in 0..250 {
                        let t = (batch * 250 + i) as f32 * 0.08;
                        chunk.push((t * 0.9).sin() * 0.85 + (t * 4.0).sin() * 0.25);
                    }
                    ctx.append_f32_points("sc_webgl", &chunk);
                    std::thread::sleep(std::time::Duration::from_millis(30));
                }
                Ok(())
            })
            .on_submit(showcase_submit)
            .flow(
                &[
                    "sc_text", "sc_richtext", "num_items", "sc_slider", "sc_range", "sc_radio_pills",
                    "sc_radio_classic", "sc_dropdown", "sc_check", "sc_date", "sc_time",
                    "sc_color", "sc_recorder", "sc_df", "sc_dataeditor", "sc_json", "sc_sortable",
                ],
                &["sc_summary"],
            )

            // Tabs structure
            .tabs(|t| {
                t
                // --- Tab 1 : Forms & Controls ---
                .tab("🎛️ Forms & Editing", |b| {
                    b.row(|r| {
                        r.item(Text::new("sc_text").label("Simple Text Input").value("My AI Model"));
                        r.item(Number::new("num_items").label("Items Count (Stepper)").value(5.0).min(1.0).max(20.0).step(1.0));
                    });
                    b.item(RichText::new("sc_richtext")
                        .label("Markdown Micro-Editor (RichText / Toolbar)")
                        .placeholder("Type your notes in Markdown...")
                        .value("### AI Model Deployment\n- **Status:** Production ready\n- **Inference:** `vLLM` engine enabled\n- **Security:** API Token required")
                        .lines(5));
                    b.row(|r| {
                        r.item(Slider::new("sc_slider").label("Temperature (Slider)").min(0.0).max(1.0).step(0.05).value(0.7));
                        r.item(Label::new("sc_slider_echo").label("Slider Echo").value("0.70").variant("success"));
                    });
                    b.item(SliderRange::new("sc_range")
                        .label("Confidence Range (SliderRange)")
                        .min(0.0).max(100.0).step(1.0)
                        .value(20.0, 80.0)
                        .unit("%"));
                    b.row(|r| {
                        r.item(Radio::new("sc_radio_pills")
                            .label("Architecture (Radio - Pills style)")
                            .choices(&["transformer", "mamba", "diffusion", "hybrid"])
                            .value("mamba"));
                        r.item(Radio::new("sc_radio_classic")
                            .label("Precision (Radio - Classic style)")
                            .style("radio")
                            .choices(&["Q4_K_M", "Q8_0", "F16"])
                            .value("Q4_K_M"));
                    });
                    b.row(|r| {
                        r.item(Dropdown::new("sc_dropdown")
                            .label("Model Selection (Dropdown)")
                            .choices(&[("llama3", "Llama 3 (8B)"), ("mistral", "Mistral (7B)"), ("qwen", "Qwen 2.5 (7B)")])
                            .value("mistral"));
                        r.item(Checkbox::new("sc_check").label("Enable GPU Acceleration").value(true));
                    });
                    b.row(|r| {
                        r.item(DatePicker::new("sc_date").label("Date").value("2026-09-03"));
                        r.item(TimePicker::new("sc_time").label("Time").value("10:15"));
                        r.item(ColorPicker::new("sc_color").label("Accent Color").value("#6366f1"));
                    });
                    b.item(Label::new("sc_color_echo").label("Color Selection").value("Active color: #6366f1"));
                    b.item(SortableList::new("sc_sortable")
                        .label("Inference Pipeline Order (SortableList drag & drop)")
                        .items(&[
                            ("p1", "1. Embeddings & Tokenization"),
                            ("p2", "2. Attention KV-Cache"),
                            ("p3", "3. Multimodal Feed-Forward"),
                            ("p4", "4. Sampling & Quantization"),
                        ]));
                })

                // --- Tab 2 : Media, Vision & Documents ---
                .tab("🖼️ Media & Documents", |b| {
                    b.row(|r| {
                        r.item(Image::new("sc_img").label("Image (upload / preview)").output().value("https://images.unsplash.com/photo-1618005182384-a83a8bd57fbe?w=500"));
                        r.item(Gallery::new("sc_gallery").label("Image Gallery").columns(3).output());
                    });
                    b.item(AnnotatedImage::new("sc_annotated")
                        .label("Vision AI: Object Detection (AnnotatedImage)")
                        .image("https://images.unsplash.com/photo-1534528741775-53994a69daeb?w=500")
                        .box_norm(0.12, 0.28, 0.72, 0.72, "person", Some(0.96), "#6366f1")
                        .box_norm(0.65, 0.35, 0.95, 0.65, "clothing", Some(0.88), "#10b981"));
                    b.item(Pdf::new("sc_pdf")
                        .label("Interactive PDF Document Viewer (with RAG/OCR Highlights)")
                        .src("https://raw.githubusercontent.com/mozilla/pdf.js/master/examples/learning/helloworld.pdf")
                        .page(1)
                        .highlight(1, 0.05, 0.14, 0.90, 0.07, "Extracted Title", "#6366f1"));
                    b.item(ImageComparison::new("sc_comp")
                        .label("Before / After Comparison (ImageComparison)")
                        .before("https://images.unsplash.com/photo-1579783900882-c0d3dad7b119?w=300", "Low Resolution (Original)")
                        .after("https://images.unsplash.com/photo-1579783900882-c0d3dad7b119?w=800", "4x Upscaled (Super-Resolution)")
                        .position(50.0));
                    b.item(ImageEditor::new("sc_editor")
                        .label("Image Editor & Inpainting Mask (ImageEditor)")
                        .value("https://images.unsplash.com/photo-1579783900882-c0d3dad7b119?w=800")
                        .layers(2));
                    b.row(|r| {
                        r.item(Audio::new("sc_audio").label("Audio Player").output());
                        r.item(AudioRecorder::new("sc_recorder").label("Voice Recorder ASR (AudioRecorder)"));
                        r.item(Video::new("sc_video").label("Video Player"));
                    });
                    b.item(Model3D::new("sc_m3d").label("3D Viewer (Model3D WebGL)"));
                })

                // --- Tab 3 : Data, Tables & Code ---
                .tab("📊 Data & GPU BigData", |b| {
                    b.item(WebGlPlot::new("sc_webgl")
                        .title("⚡ WebGL2 GPU-Accelerated Waveform (Binary Stream & 60 FPS)")
                        .xlabel("Échantillons (t)")
                        .ylabel("Amplitude")
                        .colors(&["#00f0ff", "#ff007f"])
                        .height(300)
                        .max_points(150_000)
                        .show_fps(true)
                        .series("Signal Harmonique", "#00f0ff", &[0.0, 0.4, 0.8, 1.0, 0.7, 0.1, -0.6, -0.9, -0.5, 0.2, 0.7, 0.9, 0.3, -0.4, -0.8]));
                    b.row(|r| {
                        r.item(Button::new("btn_sc_webgl_burst").label("🚀 Burst 20 000 Points").primary());
                        r.item(Button::new("btn_sc_webgl_stream").label("▶ Stream Live 2 000 pts/s").secondary());
                    });
                    b.item(PivotTable::new("sc_pivot")
                        .label("📊 OLAP Multidimensional Pivot Table (Interactive Slice & Dice)")
                        .headers(&["Country", "Segment", "Quarter", "Revenue (€)", "Margin (€)"])
                        .data(vec![
                            vec![serde_json::json!("France"), serde_json::json!("Cloud AI"), serde_json::json!("Q1"), serde_json::json!(42000), serde_json::json!(16000)],
                            vec![serde_json::json!("France"), serde_json::json!("Edge"), serde_json::json!("Q1"), serde_json::json!(18000), serde_json::json!(7500)],
                            vec![serde_json::json!("Germany"), serde_json::json!("Cloud AI"), serde_json::json!("Q1"), serde_json::json!(51000), serde_json::json!(22000)],
                            vec![serde_json::json!("Germany"), serde_json::json!("Edge"), serde_json::json!("Q2"), serde_json::json!(29000), serde_json::json!(11000)],
                            vec![serde_json::json!("USA"), serde_json::json!("Cloud AI"), serde_json::json!("Q1"), serde_json::json!(95000), serde_json::json!(41000)],
                            vec![serde_json::json!("USA"), serde_json::json!("Cybersecurity"), serde_json::json!("Q2"), serde_json::json!(64000), serde_json::json!(30000)],
                        ])
                        .rows(&["Country", "Segment"])
                        .cols(&["Quarter"])
                        .value_field("Revenue (€)")
                        .aggregator(PivotAggregator::Sum)
                        .height(300));
                    b.item(DataEditor::new("sc_dataeditor")
                        .label("Interactive Data Grid & Editor (DataEditor: Typed columns, Checkboxes, Ctrl+V TSV/CSV)")
                        .column("id", "ID", ColumnType::Text)
                        .column("service", "IT Service", ColumnType::Text)
                        .column("active", "Active", ColumnType::Boolean)
                        .column("sla_hours", "SLA (hrs)", ColumnType::Number)
                        .column("priority", "Priority", ColumnType::Dropdown(vec![
                            "P1 - Critical".into(),
                            "P2 - High".into(),
                            "P3 - Normal".into(),
                        ]))
                        .data(vec![
                            vec![serde_json::json!("SRV-01"), serde_json::json!("Password Reset"), serde_json::json!(true), serde_json::json!(1), serde_json::json!("P1 - Critical")],
                            vec![serde_json::json!("SRV-02"), serde_json::json!("Remote VPN Access"), serde_json::json!(true), serde_json::json!(4), serde_json::json!("P2 - High")],
                            vec![serde_json::json!("SRV-03"), serde_json::json!("Security Badge"), serde_json::json!(false), serde_json::json!(24), serde_json::json!("P3 - Normal")],
                        ])
                        .allow_add(true)
                        .allow_delete(true)
                        .allow_paste(true)
                        .max_height(260));
                    b.item(HighlightedText::new("sc_ner")
                        .label("Named Entity Recognition (HighlightedText / NER)")
                        .segments(&[
                            ("Google ", Some("ORG")),
                            ("was founded by ", None),
                            ("Larry Page ", Some("PER")),
                            ("and ", None),
                            ("Sergey Brin ", Some("PER")),
                            ("at ", None),
                            ("Stanford University", Some("LOC")),
                            (". Model evaluation score is ", None),
                            ("outstanding", Some("POSITIVE")),
                            (".", None),
                        ]));
                    b.item(CodeDiff::new("sc_diff")
                        .label("AI Code Comparator (CodeDiff)")
                        .old_code("fn compute(x: f64) -> f64 {\n    x * 2.0\n}")
                        .new_code("fn compute(x: f64) -> f64 {\n    // SIMD optimization\n    let res = x.mul_add(2.0, 1.0);\n    res.clamp(0.0, 100.0)\n}"));
                    b.item(Dataframe::new("sc_df")
                        .label("Interactive Data Table (Dataframe)")
                        .headers(&["ID", "Model", "Throughput (tok/s)", "VRAM (GB)"])
                        .data(&serde_json::json!([
                            [1, "Llama-3-8B", 84.5, 5.2],
                            [2, "Mistral-7B", 92.1, 4.8],
                            [3, "Qwen-2.5-7B", 105.4, 4.4]
                        ]))
                        .interactive(false));
                    b.row(|r| {
                        r.item(Code::new("sc_code")
                            .label("Syntax-Highlighted Code Editor")
                            .language("rust")
                            .value("fn evaluate(ctx: &Context) -> grio::Result<()> {\n    println!(\"Running grio evaluation...\");\n    Ok(())\n}")
                            .output()
                            .lines(true));
                        r.item(Json::new("sc_json")
                            .label("Realtime Validated JSON Editor")
                            .value(serde_json::json!({
                                "model": "qwen-2.5",
                                "quant": "Q4_K_M",
                                "temperature": 0.7,
                                "top_p": 0.95
                            })));
                    });
                    b.row(|r| {
                        r.item(File::new("sc_file").label("Multi-File Upload (File)").types(&["image/*", "text/*"]).interactive(true));
                        r.item(Explorer::new("sc_explorer").label("Server Explorer (Explorer)").root(".").pattern("*.rs"));
                    });
                    b.item(Map::new("sc_map")
                        .label("Geospatial Fleet & Infrastructure Analytics (Map / OpenStreetMap)")
                        .center(48.8566, 2.3522)
                        .zoom(12)
                        .marker(48.8584, 2.2945, "AI Compute Node A (Eiffel Tower)", Some("#6366f1"))
                        .marker(48.8606, 2.3376, "Edge Cluster B (Louvre)", Some("#10b981"))
                        .marker(48.8530, 2.3499, "Data Hub C (Notre-Dame)", Some("#ec4899"))
                        .circle(48.8566, 2.3522, 1800.0, Some("#6366f1"))
                        .height(360));
                    b.item(Html::new("sc_custom_html")
                        .label("Custom HTML / JS Component (Robust Events & window.grio Bridge)")
                        .value(r#"
                            <div style="padding: 16px; background: var(--mg-surface-2); border: 1px solid var(--mg-border); border-radius: var(--mg-radius); display: flex; flex-direction: column; gap: 12px;">
                                <div style="display: flex; align-items: center; justify-content: space-between; border-bottom: 1px solid var(--mg-border); padding-bottom: 8px;">
                                    <div style="display: flex; align-items: center; gap: 8px;">
                                        <span style="font-size: 1.2rem;">⚡</span>
                                        <strong>Sandboxed & Reactive Custom HTML / JS</strong>
                                        <span class="mg-badge" style="background: rgba(99,102,241,0.15); color: var(--mg-primary); font-size: 0.75rem; padding: 2px 8px; border-radius: 9999px;">Event Delegation & Bidirectional API</span>
                                    </div>
                                    <span style="font-size: 0.8rem; color: var(--mg-muted);">Seamless integration with the Rust backend</span>
                                </div>
                                <div style="display: flex; flex-wrap: wrap; gap: 12px; align-items: center;">
                                    <button class="mg-btn mg-btn-sm" data-grio-action="click" data-grio-payload='{"action":"pulse", "mode":"turbo"}' style="cursor: pointer;">
                                        🚀 Delegated Action (Click)
                                    </button>
                                    <div style="display: flex; align-items: center; gap: 6px;">
                                        <label style="font-size: 0.85rem; color: var(--mg-muted);">Live note:</label>
                                        <input type="text" data-grio-change class="mg-input" placeholder="Type text..." style="padding: 4px 8px; font-size: 0.85rem; border-radius: 4px; border: 1px solid var(--mg-border); background: var(--mg-bg); color: var(--mg-text);">
                                    </div>
                                    <span id="grio_bridge_badge" style="font-size: 0.8rem; color: #10b981; margin-left: auto;">🟢 window.grio active</span>
                                </div>
                            </div>
                        "#));
                })

                // --- Tab 4 : Visual Workflows & DAG ---
                .tab("🕸️ Visual Workflows (DAG)", |b| {
                    b.item(NodeGraph::new("sc_nodegraph")
                        .label("Multi-Model Pipeline & Workflow DAG Editor (ComfyUI-style)")
                        .node(GraphNode::new("n_prompt", "User Prompt", "input").output("text", "Text").pos(40.0, 50.0).status("success"))
                        .node(GraphNode::new("n_rag", "RAG Vector Search", "tool").input("query", "Text").output("context", "Documents").pos(260.0, 40.0).status("success"))
                        .node(GraphNode::new("n_llm", "Mistral-Large-24B", "llm").input("prompt", "Text").input("context", "Documents").output("response", "Text").pos(480.0, 60.0).status("running"))
                        .node(GraphNode::new("n_out", "Format & Deliver", "output").input("response", "Text").pos(720.0, 70.0).status("idle"))
                        .edge("n_prompt", "text", "n_rag", "query")
                        .edge("n_prompt", "text", "n_llm", "prompt")
                        .edge("n_rag", "context", "n_llm", "context")
                        .edge("n_llm", "response", "n_out", "response")
                        .height(420));
                })

                // --- Tab 5 : AI Chat, Observability & Slots ---
                .tab("🤖 Chatbot & Dynamic Slots", |b| {
                    b.row(|r| {
                        r.item(Metric::new("m_tps").label("Throughput").value("94.2").unit("tok/s").delta("+14.5%"));
                        r.item(Metric::new("m_ttft").label("TTFT").value("142").unit("ms").delta("-18ms").delta_color("pos"));
                        r.item(Metric::new("m_vram").label("GPU VRAM").value("5.2").unit("GB").delta("Stable").delta_color("neutral"));
                    });
                    b.item(Chatbot::new("chat_demo")
                        .label("LLM Chatbot (Markdown & Streaming)")
                        .messages(vec![
                            ChatMessage::user("How fast is grio?"),
                            ChatMessage::assistant("grio is built in **pure Rust** with asynchronous WebSockets and zero heavy JS frameworks: response times are under 2 ms."),
                        ]));
                    b.row(|r| {
                        r.item(Button::new("btn_bot_stream").label("Simulate LLM Stream"));
                        r.item(Button::new("btn_toggle_drawer").label("📂 Open Telemetry Drawer"));
                        r.item(WithLayout::new(Button::new("btn_gencsv").label("Generate CSV Export").secondary()).scale(1));
                        r.item(DownloadButton::new("dl_btn").label("Download CSV").filename("export_grio.csv"));
                    });
                    b.row(|r| {
                        r.item(Button::new("btn_inject_slot").label("➕ Inject Component (Slot)"));
                        r.item(Button::new("btn_clear_slot").label("🗑 Clear Slot").secondary());
                    });
                    b.item(Panel::new("p_slot_panel")
                        .label("Dynamic Container Zone (DynamicContainer)")
                        .item(DynamicContainer::new("sc_dynamic_slot")
                            .item(Output::new("sc_slot_init").label("Initial Slot").value("Waiting for dynamic injection..."))));
                    b.item(Plot::new("sc_plot")
                        .label("Native SVG Chart (zero dependencies)")
                        .data(&serde_json::json!({
                            "variant": "line",
                            "labels": ["P1", "P2", "P3", "P4", "P5", "P6"],
                            "series": [
                                { "name": "Prompt eval", "data": [12.0, 18.0, 24.0, 35.0, 48.0, 52.0] },
                                { "name": "Generation", "data": [80.0, 85.0, 92.0, 95.0, 102.0, 108.0] }
                            ]
                        })));
                })

                // --- Tab 6 : System, Gauges & Documentation ---
                .tab("⚙️ System, Gauges & Docs", |b| {
                    b.row(|r| {
                        r.item(Button::new("btn_toast_info").label("Toast Info"));
                        r.item(Button::new("btn_toast_success").label("Toast Success"));
                        r.item(Button::new("btn_toast_warn").label("Toast Warn").secondary());
                        r.item(Button::new("btn_toast_error").label("Toast Error").secondary());
                        r.item(Button::new("btn_prog_sim").label("Simulate Progress (Gauges)"));
                    });
                    b.row(|r| {
                        r.item(Progress::new("prog_bar").label("Progress (Bar)").bar());
                        r.item(Progress::new("prog_circle").label("Progress (Circle)").circle().size(84));
                        r.item(Progress::new("prog_pie").label("Progress (Pie)").pie().size(84));
                    });
                    b.item(Accordion::new("sc_acc").open(true).section("ℹ️ Architecture & Invariants Guide", |s| {
                        s.item(Markdown::new("sc_md").value("### grio Architecture\n- **100% Rust** backend powered by Tokio & Axum.\n- **Zero npm/node_modules** : Embedded modern CSS3 & Vanilla JS.\n- **Auto-generated REST & OpenAPI 3.0** endpoints for every interface."));
                    }));
                    b.item(Timer::new("sc_timer").label("Periodic Clock / Timer").interval(5.0));
                    b.item(Output::new("sc_summary").label("Form Snapshot (Submit Output)"));
                })
            })

            // Tiroir latéral d'inspection (Drawer)
            .item(Drawer::new("sc_drawer")
                .title("Inspection Système & Télémétrie")
                .placement("right")
                .size(420)
                .open(false)
                .content(|s| {
                    s.item(Metric::new("d_cpu").label("CPU Usage").value("18.4").unit("%"));
                    s.item(Metric::new("d_mem").label("RAM Occupée").value("1.2").unit("GB"));
                    s.item(Text::new("d_notes").label("Notes d'audit").value("Audit validé"));
                }))
    }
}

fn showcase_submit(ctx: &mut Context) -> Result<()> {
    let mut out = String::from("=== SHOWCASE SUBMISSION RESULT ===\n\n");
    if let Ok(t) = ctx.get::<String>("sc_text") {
        out.push_str(&format!("• Text: {t}\n"));
    }
    if let Ok(rt) = ctx.get::<String>("sc_richtext") {
        out.push_str(&format!(
            "• RichText (Markdown length): {} chars\n",
            rt.len()
        ));
    }
    if let Ok(n) = ctx.get::<f64>("num_items") {
        out.push_str(&format!("• Item count: {n}\n"));
    }
    if let Ok(s) = ctx.get::<f64>("sc_slider") {
        out.push_str(&format!("• Slider: {s}\n"));
    }
    if let Ok(r) = ctx.get::<(f64, f64)>("sc_range") {
        out.push_str(&format!("• SliderRange bounds: [{:.1}, {:.1}]\n", r.0, r.1));
    }
    if let Ok(pills) = ctx.get::<String>("sc_radio_pills") {
        out.push_str(&format!("• Architecture (pills): {pills}\n"));
    }
    if let Ok(rad) = ctx.get::<String>("sc_radio_classic") {
        out.push_str(&format!("• Precision (radio): {rad}\n"));
    }
    if let Ok(drop) = ctx.get::<String>("sc_dropdown") {
        out.push_str(&format!("• Dropdown: {drop}\n"));
    }
    if let Ok(chk) = ctx.get::<bool>("sc_check") {
        out.push_str(&format!("• GPU Acceleration: {chk}\n"));
    }
    if let Ok(col) = ctx.get::<String>("sc_color") {
        out.push_str(&format!("• Chosen color: {col}\n"));
    }
    if let Ok(sort) = ctx.get::<Vec<String>>("sc_sortable") {
        out.push_str(&format!("• SortableList order: {:?}\n", sort));
    }
    if let Ok(j) = ctx.get::<serde_json::Value>("sc_json") {
        out.push_str(&format!("• Valid JSON: {}\n", j));
    }

    ctx.set("sc_summary", out);
    ctx.alert(AlertLevel::Success, "Complete submission recorded!");
    Ok(())
}
