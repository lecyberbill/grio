//! # Desktop Standalone Mode — Phase 13 Example
//!
//! Demonstrates running a `grio` application as a standalone desktop app
//! with a single command (`app.launch_desktop(...)`).

use grio::*;

fn main() -> Result<()> {
    App::new("grio Desktop Assistant")
        .subtitle("Native Standalone Frameless Window App")
        .theme(Theme::cyberpunk())
        .max_width(900)
        .row(|r| {
            r.item(
                Metric::new("d_mode")
                    .label("Execution Mode")
                    .value("Desktop Native")
                    .unit("Frameless")
                    .delta("Zero NodeJS")
                    .delta_color("pos"),
            );
            r.item(
                Metric::new("d_latency")
                    .label("IPC Ping")
                    .value("< 1.0")
                    .unit("ms")
                    .delta("Pure Rust")
                    .delta_color("pos"),
            );
        })
        .item(
            Panel::new("p_desktop_info")
                .label("💻 Desktop Application Architecture")
                .item(
                    HighlightedText::new("desktop_features")
                        .label("Key Desktop Capabilities")
                        .segments(&[
                            ("app.launch_desktop(addr) ", Some("API")),
                            ("-> Automatically opens a dedicated standalone app window without browser URL bars.\n", None),
                            ("grio docker <name> ", Some("CLI")),
                            ("-> Generates a ~15MB multi-stage Docker container with zero web dependencies.\n", None),
                            ("Cross-Platform ", Some("OS")),
                            ("-> Works seamlessly on Windows, macOS, and Linux.", None),
                        ]),
                ),
        )
        .row(|r| {
            r.item(Text::new("t_input").label("User Command").value("Analyze system telemetry"));
            r.item(Button::new("btn_exec").label("Execute Task").primary());
        })
        .item(Output::new("out_desktop").label("Execution Result"))
        .on_click("btn_exec", |ctx| {
            let cmd: String = ctx.get("t_input").unwrap_or_default();
            ctx.set("out_desktop", format!("✓ Executed `{cmd}` within desktop container in 0.4ms."));
            ctx.alert(AlertLevel::Success, "Desktop action finished!");
            Ok(())
        })
        .launch_desktop("127.0.0.1:7860")
}
