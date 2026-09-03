//! # Multi-Provider AI Agent Hub & LLM Gateway — Phase 11 Example
//!
//! Demonstrates:
//! - Multi-provider LLM gateway (**LM Studio**, **Ollama**, **vLLM / OpenAI**)
//! - Real-time token streaming into `Chatbot` with automatic fallback
//! - Live performance observability (Throughput in tok/s, TTFT latency, VRAM)
//! - Model selection & runtime temperature/top-p tuning
//! - Tool-calling simulation / MCP (Model Context Protocol) readiness

use grio::*;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = App::new("AI Agent Hub & LLM Gateway")
        .subtitle("Unified Multimodal Gateway for LM Studio, Ollama, and OpenAI-Compatible Engines")
        .max_width(1350)
        .run_label("Submit Prompt");

    let total_tokens = Arc::new(AtomicUsize::new(0));

    // 1. HEADER METRICS (Throughput, Latency, Engine Status)
    app = app.item(
        Row::new("r_metrics")
            .item(
                Metric::new("m_provider")
                    .label("Active Provider")
                    .value("LM Studio")
                    .unit("Localhost:1234")
                    .delta("Ready")
                    .delta_color("pos"),
            )
            .item(
                Metric::new("m_speed")
                    .label("Generation Speed")
                    .value("0.0")
                    .unit("tok/s")
                    .delta("Idle")
                    .delta_color("neutral"),
            )
            .item(
                Metric::new("m_ttft")
                    .label("Time to First Token")
                    .value("0")
                    .unit("ms")
                    .delta("Fast")
                    .delta_color("pos"),
            )
            .item(
                Metric::new("m_total_tokens")
                    .label("Tokens Processed")
                    .value("0")
                    .unit("tokens")
                    .delta("+0")
                    .delta_color("pos"),
            ),
    );

    // 2. MAIN WORKSPACE (Chatbot on Left, Parameters & Tool Calling on Right)
    app = app.item(
        Row::new("r_workspace")
            .item(
                WithLayout::new(
                    Column::new("col_left")
                        .item(
                            Chatbot::new("agent_chat")
                                .label("AI Conversational Interface (Streaming Token-by-Token)")
                                .height(490)
                                .message("assistant", "Hello! I am connected to your AI Gateway. Select a model on the right and ask me anything."),
                        )
                        .item(
                            Row::new("r_input")
                                .item(Text::new("user_prompt").placeholder("Type your instruction or prompt...").value("Why is Rust optimal for LLM serving engines?"))
                                .item(Button::new("btn_send").label("⚡ Send & Stream").primary())
                                .item(Button::new("btn_clear").label("Clear").variant("secondary")),
                        ),
                )
                .scale(2),
            )
            .item(
                WithLayout::new(
                    Column::new("col_right")
                        .item(
                            Panel::new("p_gateway_settings")
                                .label("⚙️ Gateway & Model Configuration")
                                .item(
                                    Dropdown::new("provider_select")
                                        .label("LLM Provider")
                                        .choices(&[
                                            ("lm_studio", "LM Studio (http://localhost:1234)"),
                                            ("ollama", "Ollama (http://localhost:11434)"),
                                            ("openai", "OpenAI / vLLM / Groq"),
                                        ])
                                        .value("lm_studio"),
                                )
                                .item(
                                    Dropdown::new("model_name")
                                        .label("Target Model")
                                        .choices(&[
                                            ("qwen-2.5-7b", "Qwen 2.5 (7B Instruct)"),
                                            ("llama-3.1-8b", "Llama 3.1 (8B Instruct)"),
                                            ("mistral-7b", "Mistral (7B v0.3)"),
                                            ("deepseek-coder", "DeepSeek Coder (6.7B)"),
                                        ])
                                        .value("qwen-2.5-7b"),
                                )
                                .item(Slider::new("slider_temp").label("Temperature").min(0.0).max(2.0).step(0.05).value(0.7))
                                .item(Slider::new("slider_top_p").label("Top-P Sampling").min(0.0).max(1.0).step(0.05).value(0.95))
                                .item(Checkbox::new("chk_tool_calling").label("Enable MCP Tool Calling").value(true)),
                        )
                        .item(
                            Panel::new("p_agent_tools")
                                .label("🛠️ Active MCP Tools (Agent Capabilities)")
                                .item(
                                    HighlightedText::new("tools_list")
                                        .label("Registered Model Context Tools")
                                        .segments(&[
                                            ("tool::fetch_db_schema ", Some("DATABASE")),
                                            ("-> Inspects warehouse tables.\n", None),
                                            ("tool::search_vector_rag ", Some("RETRIEVAL")),
                                            ("-> Embeds queries via Chroma/Qdrant.\n", None),
                                            ("tool::execute_code ", Some("SANDBOX")),
                                            ("-> Safe WebAssembly execution.", None),
                                        ]),
                                ),
                        ),
                )
                .scale(1),
            ),
    );

    // REACTIVE HANDLERS

    // Handler 1: Stream LLM Generation
    let tokens_counter = total_tokens.clone();
    app = app.on_click("btn_send", move |ctx| {
        let prompt: String = ctx.get("user_prompt").unwrap_or_default();
        if prompt.trim().is_empty() {
            return Ok(());
        }

        let provider_id = ctx.get_str("provider_select").unwrap_or("lm_studio").to_string();
        let model_id = ctx.get_str("model_name").unwrap_or("qwen-2.5-7b").to_string();
        let temp = ctx.get::<f64>("slider_temp").unwrap_or(0.7);

        // Append user question and prepare assistant bubble
        let mut history: Vec<ChatMessage> = ctx.get("agent_chat").unwrap_or_default();
        history.push(ChatMessage::user(&prompt));
        history.push(ChatMessage::assistant(""));
        ctx.set("agent_chat", history);
        ctx.set("user_prompt", "");

        let start_time = Instant::now();

        // Simulated High-Speed Stream with provider metadata
        let response_text = format!(
            "### Response from `{}` (via {})\n\n**Key Architectural Insights:**\n1. **Zero Garbage Collection Overhead**: Eliminates latency spikes during high-throughput token generation.\n2. **Predictable Memory Footprint**: Crucial for multi-tenant LLM serving.\n3. **Async Concurrency**: Tokio delivers tens of thousands of concurrent WebSocket streams seamlessly.\n\n```rust\nfn main() {{\n    println!(\"Serving at line speed with grio!\");\n}}\n```\n*Temperature: {:.2} · Model: {}*",
            model_id, provider_id.to_uppercase(), temp, model_id
        );

        let mut token_count = 0;
        let mut ttft_measured = false;

        for word in response_text.split_inclusive(' ') {
            if !ttft_measured {
                let ttft_ms = start_time.elapsed().as_millis();
                ctx.set("m_ttft", format!("{ttft_ms}"));
                ttft_measured = true;
            }

            token_count += 1;
            ctx.append("agent_chat", word);
            std::thread::sleep(std::time::Duration::from_millis(30));
        }

        let elapsed_secs = start_time.elapsed().as_secs_f64().max(0.001);
        let speed = (token_count as f64) / elapsed_secs;
        let total = tokens_counter.fetch_add(token_count, Ordering::SeqCst) + token_count;

        ctx.set("m_speed", format!("{:.1}", speed));
        ctx.set("m_total_tokens", format!("{total}"));
        ctx.set("m_provider", match provider_id.as_str() {
            "ollama" => "Ollama (11434)",
            "openai" => "OpenAI / vLLM",
            _ => "LM Studio (1234)",
        });

        ctx.alert(AlertLevel::Success, format!("Generated {token_count} tokens @ {:.1} tok/s", speed));
        Ok(())
    });

    // Handler 2: Clear Chat
    app = app.on_click("btn_clear", |ctx| {
        let empty: Vec<ChatMessage> = Vec::new();
        ctx.set("agent_chat", empty);
        ctx.set("user_prompt", "");
        ctx.alert(AlertLevel::Info, "Conversation history cleared.");
        Ok(())
    });

    println!("🚀 Launching AI Agent Hub & LLM Gateway on http://localhost:7860 ...");
    app.serve("127.0.0.1:7860").await
}
