use grio::*;
use std::time::Duration;

#[tokio::test]
async fn test_api_schema_and_predict_flow() {
    let app = App::new("Test App")
        .item(Text::new("a").label("A").value("Hello"))
        .item(Slider::new("b").label("B").min(1.0).max(10.0).value(3.0))
        .item(Output::new("c").label("C"))
        .api_key("secret123")
        .on_submit(|ctx| {
            let a: String = ctx.get("a")?;
            let b: f64 = ctx.get("b")?;
            ctx.set("c", format!("{a} x{b}"));
            Ok(())
        });

    let port = 17865;
    let addr = format!("127.0.0.1:{port}");
    let addr_clone = addr.clone();

    tokio::spawn(async move {
        let _ = app.serve(addr_clone).await;
    });

    // Laisser le serveur démarrer
    tokio::time::sleep(Duration::from_millis(300)).await;

    // 1. Test GET /api/schema
    let schema = http_get(&format!("http://127.0.0.1:{port}/api/schema")).await;
    assert!(schema.contains("predict"));
    assert!(schema.contains("openapi"));

    // 2. Test GET /api/openapi.json
    let openapi = http_get(&format!("http://127.0.0.1:{port}/api/openapi.json")).await;
    assert!(openapi.contains("openapi"));
    assert!(openapi.contains("3.0.3"));

    // 3. Test GET /docs
    let docs = http_get(&format!("http://127.0.0.1:{port}/docs")).await;
    assert!(docs.contains("swagger-ui"));

    // 4. Test POST /api/predict sans clé (doit échouer 401)
    let unauth = http_post(
        &format!("http://127.0.0.1:{port}/api/predict"),
        r#"{"data":["Ada",4]}"#,
        None,
    )
    .await;
    assert!(unauth.contains("401 Unauthorized") || unauth.contains("Invalid or missing API key"));

    // 5. Test POST /api/predict avec clé (doit réussir 200)
    let auth_ok = http_post(
        &format!("http://127.0.0.1:{port}/api/predict"),
        r#"{"data":["Ada",4]}"#,
        Some("secret123"),
    )
    .await;
    assert!(auth_ok.contains("Ada x4"));
}

#[tokio::test]
async fn test_lot1_controls_api_predict() {
    let app = App::new("Test Lot 1")
        .item(
            Radio::new("arch")
                .choices(&["mamba", "transformer"])
                .value("mamba"),
        )
        .item(
            SliderRange::new("range")
                .min(0.0)
                .max(100.0)
                .value(10.0, 90.0)
                .unit("%"),
        )
        .item(ColorPicker::new("color").value("#10b981"))
        .item(Output::new("out"))
        .item(Output::new("event_echo"))
        .on_change("arch", |ctx| {
            let choice: String = ctx.get("arch")?;
            ctx.set("event_echo", format!("changed_to_{choice}"));
            Ok(())
        })
        .on_change("range", |ctx| {
            let (lo, hi): (f64, f64) = ctx.get("range")?;
            ctx.set("event_echo", format!("range_{lo}_{hi}"));
            Ok(())
        })
        .on_change("color", |ctx| {
            let col: String = ctx.get("color")?;
            ctx.set("event_echo", format!("color_{col}"));
            Ok(())
        })
        .on_submit(|ctx| {
            let arch: String = ctx.get("arch")?;
            let range: (f64, f64) = ctx.get("range")?;
            let color: String = ctx.get("color")?;
            ctx.set(
                "out",
                format!("{arch} [{:.0}-{:.0}] {color}", range.0, range.1),
            );
            Ok(())
        });

    let port = 17866;
    let addr = format!("127.0.0.1:{port}");
    let addr_clone = addr.clone();

    tokio::spawn(async move {
        let _ = app.serve(addr_clone).await;
    });

    tokio::time::sleep(Duration::from_millis(300)).await;

    // 1. Validation du rendu HTML servi (data-kind radio, sliderrange, colorpicker)
    let index_html = http_get(&format!("http://127.0.0.1:{port}/")).await;
    assert!(
        index_html.contains(r#"data-kind="radio""#),
        "Radio widget must be in HTML"
    );
    assert!(
        index_html.contains(r#"data-kind="sliderrange""#),
        "SliderRange widget must be in HTML"
    );
    assert!(
        index_html.contains(r#"data-kind="colorpicker""#),
        "ColorPicker widget must be in HTML"
    );

    // 2. Validation du schéma d'API OpenAPI
    let openapi = http_get(&format!("http://127.0.0.1:{port}/api/openapi.json")).await;
    assert!(openapi.contains("arch"), "Schema must include arch");
    assert!(openapi.contains("range"), "Schema must include range");
    assert!(openapi.contains("color"), "Schema must include color");

    // 3. Validation de prédiction /api/predict complète
    let resp = http_post(
        &format!("http://127.0.0.1:{port}/api/predict"),
        r##"{"data":["transformer",[20,80],"#ef4444"]}"##,
        None,
    )
    .await;
    assert!(
        resp.contains("transformer [20-80] #ef4444"),
        "Prediction response must match payload: {resp}"
    );
}

#[tokio::test]
async fn test_showcase_boot_and_components() {
    let app = App::showcase();
    let port = 17867;
    let addr = format!("127.0.0.1:{port}");
    let addr_clone = addr.clone();

    tokio::spawn(async move {
        let _ = app.serve(addr_clone).await;
    });

    tokio::time::sleep(Duration::from_millis(350)).await;

    // 1. Validation de la page d'accueil du showcase
    let html = http_get(&format!("http://127.0.0.1:{port}/")).await;
    assert!(html.contains("Showcase · grio"), "Title must be present");
    assert!(
        html.contains(r#"data-kind="chatbot""#),
        "Chatbot must be mounted"
    );
    assert!(
        html.contains(r#"data-kind="dataframe""#),
        "Dataframe must be mounted"
    );
    assert!(
        html.contains(r#"data-kind="metric""#),
        "Metric must be mounted"
    );
    assert!(
        html.contains(r#"data-kind="imageeditor""#),
        "ImageEditor must be mounted"
    );
    assert!(
        html.contains(r#"data-kind="sliderrange""#),
        "SliderRange must be mounted"
    );
    assert!(
        html.contains(r#"data-kind="colorpicker""#),
        "ColorPicker must be mounted"
    );
    assert!(
        html.contains(r#"data-kind="radio""#),
        "Radio must be mounted"
    );
    assert!(html.contains(r#"data-kind="plot""#), "Plot must be mounted");
    assert!(
        html.contains(r#"data-kind="annotatedimage""#),
        "AnnotatedImage must be mounted"
    );
    assert!(
        html.contains(r#"data-kind="imagecomparison""#),
        "ImageComparison must be mounted"
    );
    assert!(
        html.contains(r#"data-kind="audiorecorder""#),
        "AudioRecorder must be mounted"
    );
    assert!(
        html.contains(r#"data-kind="list""#),
        "SortableList (kind 'list') must be mounted"
    );
    assert!(
        html.contains(r#"data-kind="explorer""#),
        "Explorer must be mounted"
    );
    assert!(html.contains(r#"data-kind="file""#), "File must be mounted");
    assert!(
        html.contains(r#"data-kind="accordion""#),
        "Accordion must be mounted"
    );
    assert!(
        html.contains(r#"data-kind="progress""#),
        "Progress must be mounted"
    );
    assert!(
        html.contains(r#"data-kind="highlightedtext""#),
        "HighlightedText must be mounted"
    );
    assert!(
        html.contains(r#"data-kind="codediff""#),
        "CodeDiff must be mounted"
    );
    assert!(
        html.contains(r#"data-kind="model3d""#),
        "Model3D must be mounted"
    );
    assert!(html.contains(r#"data-kind="html""#), "Html must be mounted");

    // 2. Validation de la spécification OpenAPI
    let openapi = http_get(&format!("http://127.0.0.1:{port}/api/openapi.json")).await;
    assert!(
        openapi.contains("sc_text"),
        "OpenAPI must include showcase inputs"
    );
    assert!(
        openapi.contains("sc_slider"),
        "OpenAPI must include sc_slider"
    );
    assert!(
        openapi.contains("sc_color"),
        "OpenAPI must include sc_color"
    );

    // 3. Validation de prédiction /api/predict sur le showcase
    let payload = serde_json::json!({
        "inputs": {
            "sc_text": "My AI Test",
            "sc_richtext": "### Title\n- Item",
            "sc_dataeditor": { "columns": [], "data": [] },
            "sc_nodegraph": { "nodes": [], "edges": [] },
            "d_notes": "ok",
            "num_items": 7,
            "sc_slider": 0.85,
            "sc_range": [15, 85],
            "sc_radio_pills": "mamba",
            "sc_radio_classic": "Q4_K_M",
            "sc_dropdown": "qwen",
            "sc_check": true,
            "sc_date": "2026-09-03",
            "sc_time": "12:00",
            "sc_color": "#10b981",
            "sc_editor": {
                "image": "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=",
                "mask": ""
            },
            "sc_recorder": "data:audio/webm;base64,GkX=",
            "sc_df": [],
            "sc_json": { "model": "qwen" },
            "sc_sortable": ["p1", "p2", "p3", "p4"],
            "sc_file": [],
            "sc_explorer": ""
        }
    });

    let resp = http_post(
        &format!("http://127.0.0.1:{port}/api/predict"),
        &payload.to_string(),
        None,
    )
    .await;
    assert!(
        resp.contains("SHOWCASE SUBMISSION RESULT"),
        "Submit handler must process inputs: {resp}"
    );
    assert!(
        resp.contains("My AI Test"),
        "Result must contain submitted text"
    );
    assert!(
        resp.contains("SortableList order"),
        "Result must contain SortableList snapshot"
    );
}

#[tokio::test]
async fn test_lot2_vision_and_audio_integration() {
    let app = App::new("Test Lot 2 Vision")
        .item(AnnotatedImage::new("annotated")
            .image("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=")
            .box_norm(0.1, 0.2, 0.8, 0.9, "dog", Some(0.95), "#10b981"))
        .item(ImageComparison::new("comp")
            .before("data:image/png;base64,before", "Original")
            .after("data:image/png;base64,after", "Upscaled")
            .position(45.0))
        .item(AudioRecorder::new("rec").label("Direct Mic Recording"))
        .item(Output::new("out"))
        .on_submit(|ctx| {
            let audio: String = ctx.get("rec")?;
            ctx.set("out", format!("recorded_audio_len_{}", audio.len()));
            Ok(())
        });

    let port = 17868;
    let addr = format!("127.0.0.1:{port}");
    let addr_clone = addr.clone();

    tokio::spawn(async move {
        let _ = app.serve(addr_clone).await;
    });

    tokio::time::sleep(Duration::from_millis(300)).await;

    // 1. DOM validation
    let html = http_get(&format!("http://127.0.0.1:{port}/")).await;
    assert!(
        html.contains(r#"data-kind="annotatedimage""#),
        "AnnotatedImage in DOM"
    );
    assert!(
        html.contains(r#"data-kind="imagecomparison""#),
        "ImageComparison in DOM"
    );
    assert!(
        html.contains(r#"data-kind="audiorecorder""#),
        "AudioRecorder in DOM"
    );
    assert!(html.contains("dog"), "Class label present in initial props");

    // 2. Predict validation
    let resp = http_post(
        &format!("http://127.0.0.1:{port}/api/predict"),
        r##"{"data":["data:audio/webm;base64,GkXfo59ChoEBQveBAULygQ8="]}"##,
        None,
    )
    .await;
    assert!(
        resp.contains("recorded_audio_len_47"),
        "Prediction should reflect audio input: {resp}"
    );
}

#[tokio::test]
async fn test_progress_variants_bar_circle_pie() {
    let app = App::new("Test Progress Variants")
        .item(Progress::new("p_bar").label("Download Bar").bar())
        .item(
            Progress::new("p_circle")
                .label("Epoch Circle")
                .circle()
                .size(96),
        )
        .item(Progress::new("p_pie").label("Quota Pie").pie().size(80))
        .item(Output::new("out"))
        .on_submit(|ctx| {
            ctx.progress("p_bar", 0.65, "Téléchargement 65%");
            ctx.progress("p_circle", 0.90, "Époque 9/10");
            ctx.progress("p_pie", 0.40, "Quota 40%");
            ctx.set("out", "progress_updated");
            Ok(())
        });

    let port = 17869;
    let addr = format!("127.0.0.1:{port}");
    let addr_clone = addr.clone();

    tokio::spawn(async move {
        let _ = app.serve(addr_clone).await;
    });

    tokio::time::sleep(Duration::from_millis(300)).await;

    // 1. DOM validation
    let html = http_get(&format!("http://127.0.0.1:{port}/")).await;
    assert!(html.contains(r#"data-kind="progress""#), "Progress in DOM");
    assert!(html.contains("p_bar"), "p_bar in DOM");
    assert!(html.contains("p_circle"), "p_circle in DOM");
    assert!(html.contains("p_pie"), "p_pie in DOM");
    assert!(html.contains("Download Bar"), "Label bar present");
    assert!(html.contains("Epoch Circle"), "Label circle present");
    assert!(html.contains("Quota Pie"), "Label pie present");

    // 2. Predict execution
    let resp = http_post(
        &format!("http://127.0.0.1:{port}/api/predict"),
        r##"{"data":[]}"##,
        None,
    )
    .await;
    assert!(
        resp.contains("progress_updated"),
        "Predict output must be received: {resp}"
    );
}

#[tokio::test]
async fn test_lot4_specialized_components_integration() {
    let app = App::new("Test Lot 4 Specialized")
        .item(HighlightedText::new("ht").segments(&[
            ("Mistral ", Some("MODEL")),
            ("est hébergé en ", None),
            ("Europe", Some("LOC")),
        ]))
        .item(
            CodeDiff::new("diff")
                .old_code("let a = 1;")
                .new_code("let a = 2; // updated"),
        )
        .item(Model3D::new("mesh").value("v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3"))
        .item(Output::new("out"))
        .on_submit(|ctx| {
            ctx.set("out", "lot4_verified");
            Ok(())
        });

    let port = 17870;
    let addr = format!("127.0.0.1:{port}");
    let addr_clone = addr.clone();

    tokio::spawn(async move {
        let _ = app.serve(addr_clone).await;
    });

    tokio::time::sleep(Duration::from_millis(300)).await;

    // 1. DOM validation
    let html = http_get(&format!("http://127.0.0.1:{port}/")).await;
    assert!(
        html.contains(r#"data-kind="highlightedtext""#),
        "HighlightedText mounted"
    );
    assert!(html.contains(r#"data-kind="codediff""#), "CodeDiff mounted");
    assert!(html.contains(r#"data-kind="model3d""#), "Model3D mounted");
    assert!(html.contains("MODEL"), "Tag MODEL present in DOM");
    assert!(html.contains("Europe"), "Text Europe present in DOM");

    // 2. Predict execution
    let resp = http_post(
        &format!("http://127.0.0.1:{port}/api/predict"),
        r##"{"data":[]}"##,
        None,
    )
    .await;
    assert!(
        resp.contains("lot4_verified"),
        "Prediction output should be received: {resp}"
    );
}

#[tokio::test]
async fn test_html_custom_component_robustness() {
    let app = App::new("Test HTML Robustness")
        .item(Html::new("custom_ui")
            .label("Custom Dynamic Widget")
            .value(r#"
                <div class="custom-card">
                    <h3>Custom Interactive HTML</h3>
                    <button data-grio-action="click" data-grio-payload='{"btn":"calc"}'>Action Button</button>
                    <input type="text" data-grio-change name="user_note" value="Initial text">
                </div>
            "#))
        .item(Output::new("out"))
        .on_click("custom_ui", |ctx| {
            ctx.set("out", "custom_html_click_received");
            Ok(())
        })
        .on_change("custom_ui", |ctx| {
            ctx.set("out", "custom_html_change_received");
            Ok(())
        });

    let port = 17871;
    let addr = format!("127.0.0.1:{port}");
    let addr_clone = addr.clone();

    tokio::spawn(async move {
        let _ = app.serve(addr_clone).await;
    });

    tokio::time::sleep(Duration::from_millis(300)).await;

    // 1. Validation du montage DOM du composant HTML
    let html = http_get(&format!("http://127.0.0.1:{port}/")).await;
    assert!(
        html.contains(r#"data-kind="html""#),
        "Html component mounted in DOM"
    );
    assert!(
        html.contains("custom-card"),
        "Custom card class present in DOM"
    );
    assert!(
        html.contains("data-grio-action"),
        "Event delegation attribute present in DOM"
    );

    // 2. Validation d'appel predict simulant une interaction
    let resp = http_post(
        &format!("http://127.0.0.1:{port}/api/predict"),
        r##"{"data":[]}"##,
        None,
    )
    .await;
    assert!(resp.contains("true"), "API Predict response valid");
}

#[tokio::test]
async fn test_map_openstreetmap_component() {
    let app = App::new("Test Map OpenStreetMap")
        .item(
            Map::new("geo_map")
                .label("Fleet Map")
                .center(48.8566, 2.3522)
                .zoom(14)
                .marker(48.8584, 2.2945, "Eiffel Tower", Some("#6366f1"))
                .marker(48.8606, 2.3376, "Louvre", Some("#10b981"))
                .circle(48.8566, 2.3522, 1000.0, Some("#f59e0b"))
                .height(380),
        )
        .item(Output::new("out"))
        .on_click("geo_map", |ctx| {
            ctx.set("out", "map_clicked_ok");
            Ok(())
        });

    let port = 17873;
    let addr = format!("127.0.0.1:{port}");
    let addr_clone = addr.clone();

    tokio::spawn(async move {
        let _ = app.serve(addr_clone).await;
    });

    tokio::time::sleep(Duration::from_millis(300)).await;

    let html = http_get(&format!("http://127.0.0.1:{port}/")).await;
    assert!(html.contains(r#"data-kind="map""#), "Map in DOM");
    assert!(html.contains("Fleet Map"), "Map label in DOM");
    assert!(
        html.contains("Eiffel Tower"),
        "Marker label embedded in props"
    );
    assert!(html.contains("Louvre"), "Second marker label embedded");
    assert!(html.contains("1000"), "Circle radius in props");
}

#[tokio::test]
async fn test_phase9_lot1_drawer_and_multipage() {
    let mut app = App::new("Test Multi-Page & Drawer App").quiet();

    // Page 1
    app = app.page_with_icon("/", "Home View", "🏠", |p| {
        p.item(Text::new("greeting").value("Hello World"));
        p.item(Button::new("btn_drawer").label("Open Drawer"));
    });

    // Page 2
    app = app.page_with_icon("/settings", "Settings View", "⚙️", |p| {
        p.item(Slider::new("slider_val").min(0.0).max(10.0).value(5.0));
    });

    // Drawer
    let drawer = Drawer::new("test_drawer")
        .title("Side Drawer Title")
        .placement("right")
        .size(340)
        .content(|d| {
            d.item(Text::new("drawer_text").value("Inside Drawer"));
        });
    app = app.item(drawer);

    let port = 17874;
    let addr = format!("127.0.0.1:{port}");
    let addr_clone = addr.clone();

    tokio::spawn(async move {
        let _ = app.serve(addr_clone).await;
    });

    tokio::time::sleep(Duration::from_millis(300)).await;

    // Test Root Route
    let html_root = http_get(&format!("http://127.0.0.1:{port}/")).await;
    assert!(html_root.contains("HTTP/1.1 200 OK"), "200 on /");
    assert!(html_root.contains("mg-sidebar"), "Sidebar present");
    assert!(html_root.contains("Home View"), "Page 1 in sidebar");
    assert!(html_root.contains("Settings View"), "Page 2 in sidebar");
    assert!(html_root.contains("mg-page-view"), "Page views in DOM");
    assert!(html_root.contains("data-kind=\"drawer\""), "Drawer in DOM");
    assert!(
        html_root.contains("Side Drawer Title"),
        "Drawer title in DOM"
    );
    assert!(
        html_root.contains("mg-drawer-right"),
        "Drawer right placement"
    );

    // Test Deep-linked Route /settings
    let html_settings = http_get(&format!("http://127.0.0.1:{port}/settings")).await;
    assert!(
        html_settings.contains("HTTP/1.1 200 OK"),
        "200 on /settings route"
    );
    assert!(
        html_settings.contains("slider_val"),
        "Settings slider rendered"
    );
}

#[tokio::test]
async fn test_phase9_lot2_richtext_dataeditor_and_slots() {
    use serde_json::json;

    let app = App::new("Test Phase 9 Lot 2")
        .item(
            RichText::new("ticket_md")
                .label("Détails de l'incident")
                .value("**Erreur critique :** base de données inaccessible.")
                .lines(8),
        )
        .item(
            DataEditor::new("grid")
                .label("Catalogue Services")
                .column("id", "Réf.", ColumnType::Text)
                .column("active", "Actif", ColumnType::Boolean)
                .column("sla", "SLA", ColumnType::Number)
                .data(vec![
                    vec![json!("SRV-1"), json!(true), json!(2)],
                    vec![json!("SRV-2"), json!(false), json!(24)],
                ]),
        )
        .item(
            DynamicContainer::new("slot_zone").item(Output::new("slot_item").value("Slot initial")),
        )
        .item(Output::new("out"))
        .on_submit(|ctx| {
            let md: String = ctx.get("ticket_md").unwrap_or_default();
            ctx.set(
                "out",
                format!("verified: {}", md.contains("Erreur critique")),
            );
            Ok(())
        });

    let port = 17885;
    let addr = format!("127.0.0.1:{port}");
    let addr_clone = addr.clone();

    tokio::spawn(async move {
        let _ = app.serve(addr_clone).await;
    });

    tokio::time::sleep(Duration::from_millis(300)).await;

    // 1. Validation du rendu DOM
    let html = http_get(&format!("http://127.0.0.1:{port}/")).await;
    assert!(
        html.contains(r#"data-kind="richtext""#),
        "RichText mounted in DOM"
    );
    assert!(
        html.contains(r#"data-kind="dataeditor""#),
        "DataEditor mounted in DOM"
    );
    assert!(
        html.contains(r#"data-kind="dynamic_container""#),
        "DynamicContainer mounted in DOM"
    );
    assert!(
        html.contains(r#"data-id="ticket_md""#),
        "RichText ticket_md id in DOM"
    );
    assert!(
        html.contains(r#"data-id="grid""#),
        "DataEditor grid id in DOM"
    );
    assert!(
        html.contains(r#"data-id="slot_zone""#),
        "DynamicContainer slot_zone in DOM"
    );

    // 2. Validation de l'API predict avec RichText et DataEditor
    let req_body = json!({
        "inputs": {
            "ticket_md": "**Erreur critique :** serveur en panne",
            "grid": {
                "columns": [
                    {"id": "id", "label": "Réf.", "type": "text"},
                    {"id": "active", "label": "Actif", "type": "boolean"},
                    {"id": "sla", "label": "SLA", "type": "number"}
                ],
                "data": [
                    ["SRV-1", true, 2]
                ]
            }
        }
    });

    let resp = http_post(
        &format!("http://127.0.0.1:{port}/api/predict"),
        &req_body.to_string(),
        None,
    )
    .await;

    assert!(
        resp.contains("verified: true"),
        "Predict output should contain verification: {resp}"
    );
}

#[tokio::test]
async fn test_phase9_lot3_pdf_and_nodegraph() {
    let app = App::new("Test Phase 9 Lot 3")
        .item(
            Pdf::new("pdf_viewer")
                .label("Document Analysis")
                .src("https://example.com/test.pdf")
                .page(2)
                .highlight(2, 0.1, 0.2, 0.5, 0.1, "Extracted Block", "#10b981"),
        )
        .item(
            NodeGraph::new("dag_pipeline")
                .label("DAG Orchestrator")
                .node(GraphNode::new("n1", "Input", "input").output("out", "Text"))
                .node(
                    GraphNode::new("n2", "LLM", "llm")
                        .input("in", "Text")
                        .output("out", "Text"),
                )
                .edge("n1", "out", "n2", "in")
                .height(400),
        )
        .item(Output::new("out"))
        .on_submit(|ctx| {
            ctx.set("out", "lot3_verified");
            Ok(())
        });

    let port = 17890;
    let addr = format!("127.0.0.1:{port}");
    let addr_clone = addr.clone();

    tokio::spawn(async move {
        let _ = app.serve(addr_clone).await;
    });

    tokio::time::sleep(Duration::from_millis(300)).await;

    // 1. Validation du rendu DOM
    let html = http_get(&format!("http://127.0.0.1:{port}/")).await;
    assert!(html.contains(r#"data-kind="pdf""#), "Pdf mounted in DOM");
    assert!(
        html.contains(r#"data-kind="nodegraph""#),
        "NodeGraph mounted in DOM"
    );
    assert!(html.contains(r#"data-id="pdf_viewer""#), "Pdf id in DOM");
    assert!(
        html.contains(r#"data-id="dag_pipeline""#),
        "NodeGraph id in DOM"
    );

    // 2. Validation de l'API predict
    let resp = http_post(
        &format!("http://127.0.0.1:{port}/api/predict"),
        r#"{"inputs":{"dag_pipeline":{"nodes":[],"edges":[]}}}"#,
        None,
    )
    .await;

    assert!(
        resp.contains("lot3_verified"),
        "Predict output should contain verification: {resp}"
    );
}

async fn http_get(url: &str) -> String {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    let url = url.strip_prefix("http://").unwrap();
    let (host, path) = url.split_once('/').unwrap_or((url, ""));
    let path = format!("/{path}");

    let mut stream = TcpStream::connect(host).await.unwrap();
    let req = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    stream.write_all(req.as_bytes()).await.unwrap();

    let mut resp = String::new();
    stream.read_to_string(&mut resp).await.unwrap();
    resp
}

async fn http_post(url: &str, body: &str, api_key: Option<&str>) -> String {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    let url = url.strip_prefix("http://").unwrap();
    let (host, path) = url.split_once('/').unwrap_or((url, ""));
    let path = format!("/{path}");

    let mut stream = TcpStream::connect(host).await.unwrap();
    let auth_header = if let Some(key) = api_key {
        format!("X-API-Key: {key}\r\n")
    } else {
        String::new()
    };

    let req = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{auth_header}Connection: close\r\n\r\n{body}",
        body.len()
    );
    stream.write_all(req.as_bytes()).await.unwrap();

    let mut resp = String::new();
    stream.read_to_string(&mut resp).await.unwrap();
    resp
}

#[tokio::test]
async fn test_phase14_webgl_plot_and_pivot_table() {
    let app = App::new("Phase 14 Test")
        .item(
            WebGlPlot::new("gpu_scope")
                .title("GPU Waveform")
                .height(400)
                .max_points(50_000)
                .series("Test Series", "#00f0ff", &[1.0, 2.0, 3.5]),
        )
        .item(
            PivotTable::new("sales_cube")
                .label("Sales Pivot")
                .headers(&["Country", "Category", "Amount"])
                .data(vec![
                    vec![
                        serde_json::json!("FR"),
                        serde_json::json!("Tech"),
                        serde_json::json!(100),
                    ],
                    vec![
                        serde_json::json!("FR"),
                        serde_json::json!("Tech"),
                        serde_json::json!(150),
                    ],
                    vec![
                        serde_json::json!("US"),
                        serde_json::json!("Food"),
                        serde_json::json!(80),
                    ],
                ])
                .rows(&["Country"])
                .cols(&["Category"])
                .value_field("Amount")
                .aggregator(PivotAggregator::Sum),
        )
        .on_submit(|ctx| {
            let pts = vec![0.1f32, 0.5f32, 0.9f32];
            ctx.append_f32_points("gpu_scope", &pts);
            ctx.append_series_points("gpu_scope", 1, &pts);
            Ok(())
        });

    let port = 17882;
    let addr = format!("127.0.0.1:{port}");
    let addr_clone = addr.clone();

    tokio::spawn(async move {
        let _ = app.serve(addr_clone).await;
    });

    tokio::time::sleep(Duration::from_millis(300)).await;

    // Test du schéma OpenAPI et composants
    let schema = http_get(&format!("http://127.0.0.1:{port}/api/schema")).await;
    assert!(schema.contains("gpu_scope"));
    assert!(schema.contains("webgl_plot"));
    assert!(schema.contains("sales_cube"));
    assert!(schema.contains("pivot_table"));
}

#[tokio::test]
async fn test_phase15_mcp_server_protocol() {
    let app = App::new("MCP Server Test").mcp(true).mcp_tool(
        "calculate_area",
        "Calculate geometric area",
        serde_json::json!({
            "type": "object",
            "properties": {
                "width": { "type": "number" },
                "height": { "type": "number" }
            },
            "required": ["width", "height"]
        }),
        |args| {
            let w = args.get("width").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let h = args.get("height").and_then(|v| v.as_f64()).unwrap_or(0.0);
            Ok(serde_json::json!({ "area": w * h }))
        },
    );

    let port = 17883;
    let addr = format!("127.0.0.1:{port}");
    let addr_clone = addr.clone();

    tokio::spawn(async move {
        let _ = app.serve(addr_clone).await;
    });

    tokio::time::sleep(Duration::from_millis(300)).await;

    // 1. Handshake MCP : initialize
    let init_resp = http_post(
        &format!("http://127.0.0.1:{port}/mcp/v1"),
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
        None,
    )
    .await;
    assert!(init_resp.contains("2024-11-05"));
    assert!(init_resp.contains("serverInfo"));

    // 2. Découverte d'outils : tools/list
    let list_resp = http_post(
        &format!("http://127.0.0.1:{port}/mcp/v1"),
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list"}"#,
        None,
    )
    .await;
    assert!(list_resp.contains("calculate_area"));
    assert!(list_resp.contains("Calculate geometric area"));

    // 3. Exécution d'un outil : tools/call
    let call_resp = http_post(
        &format!("http://127.0.0.1:{port}/mcp/v1"),
        r#"{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"calculate_area","arguments":{"width":12.0,"height":5.0}}}"#,
        None,
    )
    .await;
    assert!(call_resp.contains("area"));
    assert!(call_resp.contains("60"));

    // 4. Découverte REST directe : GET /mcp/tools
    let direct_tools = http_get(&format!("http://127.0.0.1:{port}/mcp/tools")).await;
    assert!(direct_tools.contains("calculate_area"));
}

#[tokio::test]
async fn test_phase15_wasm_plugin_sandbox() {
    let mock_plugin = WasmPlugin::new("custom_wasm_filter")
        .limits(SandboxLimits {
            max_memory_pages: 64,
            max_fuel: 1_000_000,
            timeout_ms: 1000,
        })
        .register_method("transform", |bytes| {
            let input: serde_json::Value = serde_json::from_slice(bytes)?;
            let msg = input.get("msg").and_then(|v| v.as_str()).unwrap_or("");
            let out = serde_json::json!({
                "transformed": format!("[WASM_SECURE: {msg}]"),
                "bytes_len": msg.len()
            });
            Ok(serde_json::to_vec(&out)?)
        })
        .register_method("future_unforeseen_method", |_bytes| {
            // Démonstration d'une méthode non prévue au départ
            Ok(serde_json::to_vec(&serde_json::json!({ "dynamic_feature": 42 }))?)
        });

    let app = App::new("WASM Test")
        .wasm_plugin("filter", mock_plugin)
        .item(Text::new("raw_in").label("Input").value("test payload"))
        .item(Output::new("processed_out").label("Output"))
        .on_submit(|ctx| {
            let raw: String = ctx.get("raw_in")?;
            let res = ctx.call_wasm("filter", "transform", &serde_json::json!({ "msg": raw }))?;
            let out_str = res["transformed"].as_str().unwrap_or("").to_string();
            ctx.set("processed_out", out_str);
            Ok(())
        });

    let port = 17884;
    let addr = format!("127.0.0.1:{port}");
    let addr_clone = addr.clone();

    tokio::spawn(async move {
        let _ = app.serve(addr_clone).await;
    });

    tokio::time::sleep(Duration::from_millis(300)).await;

    // Test predict déclenchant le handler appelant le plugin WASM
    let resp = http_post(
        &format!("http://127.0.0.1:{port}/api/predict"),
        r#"{"data":["hello from client"]}"#,
        None,
    )
    .await;

    assert!(resp.contains("[WASM_SECURE: hello from client]"));
}

#[tokio::test]
async fn test_phase15_enterprise_auth_and_rbac() {
    let test_user = UserProfile::new("usr_1", "charlie")
        .roles(&["analyst"])
        .email("charlie@corp.local");

    let app = App::new("Auth Test")
        .auth(
            AuthConfig::enabled()
                .with_mock_users(vec![test_user.clone()])
        )
        .item(Text::new("req_in").label("Input").value("dataset_a"))
        .item(Output::new("audit_out").label("Audit"))
        .on_submit(|ctx| {
            let user_repr = match ctx.user() {
                Some(u) => format!("user:{}|role:{}", u.username, u.roles.join(",")),
                None => "anonymous".to_string(),
            };
            ctx.set("audit_out", user_repr);
            Ok(())
        });

    let port = 17886;
    let addr = format!("127.0.0.1:{port}");
    let addr_clone = addr.clone();

    tokio::spawn(async move {
        let _ = app.serve(addr_clone).await;
    });

    tokio::time::sleep(Duration::from_millis(400)).await;

    // 1. Test /auth/login page exists
    let login_page = http_get(&format!("http://127.0.0.1:{port}/auth/login")).await;
    assert!(login_page.contains("Sign in as") || login_page.contains("Enterprise Single Sign-On"));
    assert!(login_page.contains("charlie"));

    // 2. Test /auth/user sans session (anonyme)
    let anon_user = http_get(&format!("http://127.0.0.1:{port}/auth/user")).await;
    assert!(anon_user.contains(r#""authenticated":false"#));

    // 3. Test execution avec handler sans session
    let resp_anon = http_post(
        &format!("http://127.0.0.1:{port}/api/predict"),
        r#"{"data":["dataset_a"]}"#,
        None,
    )
    .await;
    assert!(resp_anon.contains("anonymous"));
}

#[tokio::test]
async fn test_phase15_chromatix_visual_passkey_auth() {
    let master_key = "lab_secret_key_42";
    let auth_cfg = AuthConfig::enabled().with_chromatix_pixel(master_key);
    let auth_mgr = AuthManager::new(auth_cfg);

    let alice = UserProfile::new("alice_phd", "Alice Scientist")
        .roles(&["researcher", "admin"])
        .email("alice@lab.internal");

    // 1. Génération d'un passkey PNG valide
    let badge_png = auth_mgr.create_chromatix_badge(alice.clone(), master_key, 3600);
    assert!(!badge_png.is_empty(), "PNG badge should not be empty");
    assert_eq!(&badge_png[0..8], b"\x89PNG\r\n\x1a\n", "Valid PNG header");

    // 2. Décodage et vérification réussie
    let verified_user = auth_mgr.verify_chromatix_badge(&badge_png, master_key);
    assert!(verified_user.is_ok(), "Verification should succeed with valid key");
    let user = verified_user.unwrap();
    assert_eq!(user.id, "alice_phd");
    assert_eq!(user.username, "Alice Scientist");
    assert!(user.has_role("admin"));

    // 3. Tentative de falsification (mauvaise clé maître)
    let bad_key_verify = auth_mgr.verify_chromatix_badge(&badge_png, "wrong_master_key");
    assert!(bad_key_verify.is_err(), "Verification must fail with wrong key");

    // 4. Tentative de corruption de l'image (altération de bit)
    let mut tampered_png = badge_png.clone();
    let mid = tampered_png.len() / 2;
    tampered_png[mid] ^= 0xFF; // Corruption binaire
    let tampered_verify = auth_mgr.verify_chromatix_badge(&tampered_png, master_key);
    assert!(tampered_verify.is_err(), "Tampered image must be rejected");
}
