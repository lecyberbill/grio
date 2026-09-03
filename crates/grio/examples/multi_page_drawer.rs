//! # Multi-Page SPA, Drawers & Responsive Layout — Phase 9 Lot 1 Example
//!
//! Demonstrates:
//! - Multi-page application with declarative routing (`app.page`)
//! - Automatic responsive sidebar navigation (with mobile slide-out drawer)
//! - Sliding side drawers (`Drawer`) on the right and bottom (bottom-sheet)
//! - Reactive handlers dynamically opening and closing drawers
//!
//! Run with:
//! ```bash
//! cargo run -p grio --example multi_page_drawer
//! ```

use grio::*;

#[tokio::main]
async fn main() -> grio::Result<()> {
    let mut app = App::new("Enterprise AI Hub");
    app.subtitle = "Multi-Page Routing & Sliding Drawers Showcase".to_string();

    // -----------------------------------------------------------------------
    // Page 1 : Dashboard & Assistant IA
    // -----------------------------------------------------------------------
    app = app.page_with_icon("/", "Dashboard & Copilot", "📊", |p| {
        p.grid(3, |g| {
            g.item(
                Metric::new("kpi_tps")
                    .label("Throughput")
                    .value("74.2")
                    .unit("tok/s")
                    .delta("+18.4%"),
            );
            g.item(
                Metric::new("kpi_latency")
                    .label("TTFT Latency")
                    .value("142")
                    .unit("ms")
                    .delta("-22.1%"),
            );
            g.item(
                Metric::new("kpi_vram")
                    .label("VRAM Usage")
                    .value("4.8")
                    .unit("GB")
                    .delta_color("off"),
            );
        });

        p.row(|r| {
            r.item(
                Button::new("open_settings")
                    .label("⚙️ Open Settings Drawer (Right)")
                    .primary(),
            );
            r.item(Button::new("open_logs").label("📋 Open System Logs (Bottom Sheet)"));
        });

        p.item(
            Chatbot::new("bot")
                .label("AI Assistant")
                .height(380)
                .message(
                    "assistant",
                    "Hello! Welcome to the new Multi-Page & Responsive interface of **grio**.",
                ),
        );
    });

    // -----------------------------------------------------------------------
    // Page 2 : Analytics & Visualisations
    // -----------------------------------------------------------------------
    app = app.page_with_icon("/analytics", "Analytics & Models", "📈", |p| {
        p.panel("Evaluation Benchmarks", |pan| {
            pan.item(
                Plot::new("benchmarks")
                    .label("Model Performance by Category")
                    .variant("bar")
                    .title("Benchmark Accuracy (%)"),
            );
        });
    });

    // -----------------------------------------------------------------------
    // Page 3 : Configuration Globale
    // -----------------------------------------------------------------------
    app = app.page_with_icon("/config", "System Configuration", "⚙️", |p| {
        p.panel("Environment Settings", |pan| {
            pan.item(
                Text::new("api_endpoint")
                    .label("LLM Endpoint URL")
                    .value("https://api.openai.com/v1"),
            );
            pan.item(
                Slider::new("timeout")
                    .label("Request Timeout (s)")
                    .min(5.0)
                    .max(120.0)
                    .value(30.0),
            );
            pan.item(
                Checkbox::new("auto_backup")
                    .label("Enable Automatic Backups")
                    .value(true),
            );
        });
    });

    // -----------------------------------------------------------------------
    // Tiroir 1 : Paramètres du Modèle (Droite)
    // -----------------------------------------------------------------------
    let settings_drawer = Drawer::new("settings_drawer")
        .title("⚙️ Model Parameters")
        .placement("right")
        .size(380)
        .content(|d| {
            d.item(
                Slider::new("temp")
                    .label("Temperature")
                    .min(0.0)
                    .max(2.0)
                    .value(0.7),
            );
            d.item(
                Slider::new("top_p")
                    .label("Top-P")
                    .min(0.0)
                    .max(1.0)
                    .value(0.95),
            );
            d.item(
                Slider::new("rep_pen")
                    .label("Repetition Penalty")
                    .min(1.0)
                    .max(2.0)
                    .value(1.1),
            );
            d.item(
                Checkbox::new("stream")
                    .label("Enable Token Streaming")
                    .value(true),
            );
            d.item(
                Button::new("save_params_btn")
                    .label("Apply & Close")
                    .primary(),
            );
        });
    app = app.item(settings_drawer);

    // -----------------------------------------------------------------------
    // Tiroir 2 : Logs Système (Bas / Bottom-Sheet)
    // -----------------------------------------------------------------------
    let logs_drawer = Drawer::new("logs_drawer")
        .title("📋 Real-time System Logs")
        .placement("bottom")
        .size(280)
        .content(|d| {
            d.item(Markdown::new("logs_content").value("`[12:00:01]` Model checkpoint loaded into memory\n`[12:00:03]` Tokio runtime pool initialized (4 worker threads)\n`[12:00:05]` WebSocket bus ready for multi-client connections"));
            d.item(Button::new("close_logs_btn").label("Close Console"));
        });
    app = app.item(logs_drawer);

    // -----------------------------------------------------------------------
    // Handlers d'interaction
    // -----------------------------------------------------------------------
    app = app
        .on_click("open_settings", |ctx| {
            ctx.set("settings_drawer", true);
            Ok(())
        })
        .on_click("save_params_btn", |ctx| {
            ctx.set("settings_drawer", false);
            ctx.alert(AlertLevel::Success, "Parameters applied successfully!");
            Ok(())
        })
        .on_click("open_logs", |ctx| {
            ctx.set("logs_drawer", true);
            Ok(())
        })
        .on_click("close_logs_btn", |ctx| {
            ctx.set("logs_drawer", false);
            Ok(())
        });

    app.launch("127.0.0.1:7860")
}
