// [WFGY] Zone: SAFE | λ: 0.2 | Fallbacks: 0 | Action: Interactive WASM plugin sandbox example in English
//! # Example: WebAssembly Plugin Engine (`WasmPlugin`)
//!
//! This example demonstrates how to:
//! 1. Declare and load sandboxed third-party plugins with `WasmPlugin`.
//! 2. Execute dynamic plugin methods (`ctx.call_wasm`) to enrich the application
//!    (NLP text moderation, custom data transformations, anomaly scoring).
//! 3. Enforce strict sandboxing with memory limits (`SandboxLimits`) and an extensible ABI.
//!
//! To run this example:
//! ```bash
//! cargo run --example wasm_plugins
//! ```

use grio::*;
use serde_json::json;

fn main() -> Result<()> {
    // 1. Declare a sandboxed NLP & text moderation plugin
    let nlp_plugin = WasmPlugin::new("sentiment_and_moderator")
        .limits(SandboxLimits {
            max_memory_pages: 128, // 8 MB
            max_fuel: 5_000_000,
            timeout_ms: 2000,
        })
        .register_method("analyze", |input_bytes| {
            let input: serde_json::Value = serde_json::from_slice(input_bytes)?;
            let text = input.get("text").and_then(|v| v.as_str()).unwrap_or("");
            
            // Sandboxed processing logic
            let bad_words = ["spam", "scam", "phishing", "malware", "hate"];
            let mut flagged = false;
            for w in &bad_words {
                if text.to_lowercase().contains(w) {
                    flagged = true;
                    break;
                }
            }

            let word_count = text.split_whitespace().count();
            let positive_words = ["super", "great", "excellent", "fast", "love", "rust"];
            let positive_count = positive_words.iter().filter(|w| text.to_lowercase().contains(*w)).count();
            let sentiment = if positive_count > 0 { "Positive 🟢" } else if flagged { "Toxic / Flagged 🔴" } else { "Neutral ⚪" };

            let output = json!({
                "flagged": flagged,
                "sentiment": sentiment,
                "word_count": word_count,
                "processed_by": "wasm_nlp_sandbox_v1",
                "clean_text": if flagged { "[CONTENT MASKED BY SANDBOX]" } else { text }
            });
            Ok(serde_json::to_vec(&output)?)
        })
        .register_method("extract_keywords", |input_bytes| {
            let input: serde_json::Value = serde_json::from_slice(input_bytes)?;
            let text = input.get("text").and_then(|v| v.as_str()).unwrap_or("");
            let words: Vec<&str> = text.split_whitespace().filter(|w| w.len() > 4).collect();
            let out = json!({ "keywords": words });
            Ok(serde_json::to_vec(&out)?)
        });

    // 2. Declare a sandboxed financial scoring plugin
    let math_plugin = WasmPlugin::new("crypto_quant_scorer")
        .register_method("score_transaction", |input_bytes| {
            let input: serde_json::Value = serde_json::from_slice(input_bytes)?;
            let amount = input.get("amount").and_then(|v| v.as_f64()).unwrap_or(0.0);
            let risk_factor = input.get("risk_factor").and_then(|v| v.as_f64()).unwrap_or(1.0);
            
            let anomaly_score = ((amount * 0.00042 * risk_factor).min(100.0) * 100.0).round() / 100.0;
            let status = if anomaly_score > 75.0 { "SUSPICIOUS (Alert)" } else { "VALIDATED (Compliant)" };

            let out = json!({
                "anomaly_score": anomaly_score,
                "status": status,
                "computed_in_wasm": true
            });
            Ok(serde_json::to_vec(&out)?)
        });

    // 3. Assemble the grio interface with WASM plugins
    App::new("🛡️ grio WebAssembly Sandbox Plugin Studio")
        .subtitle("Safe, sandboxed execution of third-party plugins without host server recompilation")
        .theme(Theme::tokyo_night())
        .wasm_plugin("nlp", nlp_plugin)
        .wasm_plugin("quant", math_plugin)
        .tabs(|t| {
            t.tab("💬 NLP & Moderation Plugin", |tab| {
                tab.panel("User Input", |p| {
                    p.item(
                        Text::new("prompt_text")
                            .label("Text to evaluate inside the WASM Sandbox")
                            .value("Rust and grio deliver blazing fast performance for machine learning workflows!")
                    );
                    p.item(Button::new("btn_run_nlp").label("⚡ Execute in WASM Sandbox").primary());
                });
                tab.panel("WASM Sandbox Results", |p| {
                    p.item(Json::new("nlp_result").label("JSON Output from WASM Plugin"));
                    p.item(Label::new("sentiment_badge").label("Sentiment Score"));
                });
            })
            .tab("📊 Quant & Calculation Plugin", |tab| {
                tab.panel("Transaction Parameters", |p| {
                    p.item(Number::new("tx_amount").label("Transaction Amount ($)").value(15420.0));
                    p.item(Slider::new("tx_risk").label("Risk Multiplier").min(0.5).max(5.0).step(0.1).value(1.2));
                    p.item(Button::new("btn_run_quant").label("📈 Run WASM Scoring").primary());
                });
                tab.panel("WASM Risk Evaluation", |p| {
                    p.item(Json::new("quant_result").label("Plugin Report"));
                    p.item(Label::new("risk_badge").label("Status"));
                });
            })
        })
        // Reactive event handlers invoking WASM plugins
        .on_click("btn_run_nlp", |ctx| {
            let text: String = ctx.get("prompt_text").unwrap_or_default();
            
            // Direct invocation inside the WASM sandbox
            let res = ctx.call_wasm("nlp", "analyze", &json!({ "text": text }))?;
            let sentiment = res.get("sentiment").and_then(|s| s.as_str()).unwrap_or("Unknown").to_string();
            
            ctx.set("nlp_result", res);
            ctx.set("sentiment_badge", sentiment);
            ctx.alert(AlertLevel::Success, "WASM NLP plugin executed in sandbox successfully!");
            Ok(())
        })
        .on_click("btn_run_quant", |ctx| {
            let amount: f64 = ctx.get("tx_amount").unwrap_or(0.0);
            let risk: f64 = ctx.get("tx_risk").unwrap_or(1.0);

            // Invoke quant plugin
            let res = ctx.call_wasm("quant", "score_transaction", &json!({ "amount": amount, "risk_factor": risk }))?;
            let status = res.get("status").and_then(|s| s.as_str()).unwrap_or("N/A").to_string();

            ctx.set("quant_result", res);
            ctx.set("risk_badge", status);
            ctx.alert(AlertLevel::Info, "WASM execution completed without host overhead.");
            Ok(())
        })
        .launch("127.0.0.1:7860")
}
