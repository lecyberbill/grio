use std::thread;
use std::time::Duration;

use grio::*;

fn main() -> grio::Result<()> {
    App::new("Greet · grio demo")
        .subtitle("A fast pure-Rust Gradio alternative — server, components, events and real-time streaming.")
        .add(
            Markdown::new("intro").text(
                "# Welcome\n\nThis is a **Markdown** view served by the engine.\nEnter a name, select intensity, then click *Run*.",
            ),
        )
        .row(|r| {
            r.item(Text::new("name").label("Name").value("World").placeholder("Your name…"));
            r.item(
                Slider::new("intensity")
                    .label("Intensity")
                    .min(0.0)
                    .max(5.0)
                    .step(1.0)
                    .value(2.0),
            );
        })
        .item(Output::new("greeting").label("Greeting"))
        .panel("Real-time — streaming, progress, alerts", |p| {
            p.item(
                Markdown::new("rt").text(
                    "Click **Generate**: the handler runs in the background pool, tokens are **streamed in real time** and the progress bar updates. Click again to **cancel**.",
                ),
            );
            p.item(Progress::new("pg").label("Generation"));
            p.item(Output::new("log").label("Streaming Output"));
            p.item(Button::new("generate").label("Start Generation"));
        })
        .on_event("reset", |_ctx| {
            println!("[event] reset ← server");
            Ok(())
        })
        .on_submit(|ctx| {
            let name: String = ctx.get("name")?;
            let intensity: f64 = ctx.get("intensity")?;
            let greeting = format!("Hello, {} {}!", name, "!".repeat(intensity as usize));
            ctx.set("greeting", greeting);
            Ok(())
        })
        .on_click("generate", |ctx| {
            // Simulated long task: handler runs on background worker thread
            ctx.set("log", "Starting…\n");
            for i in 1..=10 {
                if ctx.cancelled() {
                    ctx.alert(AlertLevel::Warn, "Generation cancelled");
                    return Ok(());
                }
                ctx.progress("pg", i as f64 / 10.0, format!("step {i}/10"));
                ctx.append("log", format!("token {i}\n"));
                thread::sleep(Duration::from_millis(350));
            }
            ctx.alert(AlertLevel::Success, "Generation completed");
            Ok(())
        })
        .launch("127.0.0.1:7860")
}
