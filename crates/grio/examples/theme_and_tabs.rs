use grio::*;

fn main() -> grio::Result<()> {
    App::new("AI Workbench · Themes & Tabs")
        .subtitle("Demonstration of Dark Mode, Multi-Tab Workspaces and Real-Time AI Metrics")
        .theme(Theme::dark().primary("#6366f1").radius("12px"))
        .tabs(|t| {
            t
                // Tab 1: LLM Chatbot
                .tab("💬 LLM Assistant", |b| {
                    b.item(
                        Chatbot::new("chat")
                            .label("Llama-3-8B-Instruct")
                            .height(360)
                            .message("assistant", "Hello! I am your local AI assistant running with grio. How can I help you today?")
                    );
                    b.row(|r| {
                        r.item(Text::new("prompt").placeholder("Ask anything...").value("What are the advantages of Rust for AI?"));
                        r.item(Button::new("send").label("Send").primary());
                    });
                })
                // Tab 2: Performance Metrics & Benchmarks
                .tab("📊 Model Metrics", |b| {
                    b.row(|r| {
                        r.item(Metric::new("tps").label("Throughput").value("54.2").unit("tok/s").delta("+14.8%").delta_color("normal"));
                        r.item(Metric::new("ttft").label("Time To First Token").value("128").unit("ms").delta("-22ms").delta_color("normal"));
                        r.item(Metric::new("vram").label("VRAM Usage").value("4.2").unit("GB").delta("+0.1 GB").delta_color("inverse"));
                    });
                    b.item(
                        Plot::new("benchmarks")
                            .label("Inference Speed across Context Lengths")
                            .variant("bar")
                            .title("Tokens per Second (Higher is Better)")
                            .xlabel("Context Size")
                            .ylabel("tok/s")
                    );
                    b.item(Button::new("refresh_bench").label("Run Benchmark"));
                })
                // Tab 3: Model Parameters & Configuration
                .tab("⚙️ Settings", |b| {
                    b.row(|r| {
                        r.item(Dropdown::new("quant").label("Quantization").options(&["Q4_K_M", "Q8_0", "FP16"]).value("Q4_K_M"));
                        r.item(Slider::new("temp").label("Temperature").min(0.0).max(2.0).step(0.05).value(0.7));
                        r.item(Slider::new("top_p").label("Top P").min(0.0).max(1.0).step(0.05).value(0.9));
                    });
                    b.item(Output::new("config_saved").label("Status"));
                    b.item(Button::new("save_cfg").label("Save Configuration").primary());
                })
        })
        .on_click("send", |ctx| {
            let prompt: String = ctx.get("prompt").unwrap_or_default();
            if prompt.trim().is_empty() { return Ok(()); }
            let mut hist: Vec<ChatMessage> = ctx.get("chat").unwrap_or_default();
            hist.push(ChatMessage::user(&prompt));
            hist.push(ChatMessage::assistant(format!("Here is why Rust is great for AI:\n- **Zero-cost abstractions** & native execution speed.\n- **Memory safety without garbage collection** (predictable latencies for streaming tokens).\n- **High concurrency** with Tokio & Rayon for tensor workloads.")));
            ctx.set("chat", hist);
            ctx.set("prompt", "");
            ctx.alert(AlertLevel::Success, "Answer generated!");
            Ok(())
        })
        .on_click("refresh_bench", |ctx| {
            ctx.set("tps", serde_json::json!({ "value": "58.6", "delta": "+22.4%" }));
            ctx.set("ttft", serde_json::json!({ "value": "112", "delta": "-38ms" }));
            ctx.set("benchmarks", serde_json::json!({
                "labels": ["512 tokens", "1024 tokens", "2048 tokens", "4096 tokens"],
                "series": [
                    { "name": "Llama-3-8B Q4", "data": [62.4, 58.6, 51.2, 44.8] },
                    { "name": "Mistral-7B Q4", "data": [59.1, 55.4, 49.0, 42.1] }
                ]
            }));
            ctx.alert(AlertLevel::Info, "Benchmarks updated successfully");
            Ok(())
        })
        .on_click("save_cfg", |ctx| {
            let quant: String = ctx.get("quant").unwrap_or_default();
            let temp: f64 = ctx.get("temp").unwrap_or(0.7);
            ctx.set("config_saved", format!("✓ Configuration applied: Quantization={quant}, Temp={temp}"));
            ctx.alert(AlertLevel::Success, "Settings saved");
            Ok(())
        })
        .launch("127.0.0.1:7860")
}
