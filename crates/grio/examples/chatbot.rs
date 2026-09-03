use grio::*;
use std::thread;
use std::time::Duration;

fn main() -> grio::Result<()> {
    App::new("LLM Chatbot · grio demo")
        .subtitle("AI conversational interface with interactive message bubbles and real-time token-by-token streaming.")
        .item(
            Chatbot::new("chat")
                .label("AI Assistant (Llama / Rust)")
                .height(480)
                .message("assistant", "Hello! I am your local AI assistant. How can I help you today?")
        )

        .row(|r| {
            r.item(Text::new("prompt").placeholder("Ask the AI a question... (e.g. Write a Rust function)"));
            r.item(Button::new("send").label("Send").primary());
            r.item(Button::new("clear").label("Clear"));
        })

        // Send user message + stream bot reply
        .on_click("send", |ctx| {
            let prompt: String = ctx.get("prompt").unwrap_or_default();
            if prompt.trim().is_empty() {
                return Ok(());
            }

            // 1. Retrieve existing chat history
            let mut history: Vec<ChatMessage> = ctx.get("chat").unwrap_or_default();
            history.push(ChatMessage::user(&prompt));
            history.push(ChatMessage::assistant("")); // empty assistant bubble ready for streaming
            ctx.set("chat", history);
            ctx.set("prompt", ""); // reset input field

            // 2. Simulate token streaming (similar to Candle or llama.cpp)
            let simulated_reply = format!(
                "Here is the generated answer for **\"{}\"**:\n\n```rust\nfn main() {{\n    println!(\"High-performance AI in pure Rust!\");\n}}\n```\nAll rendered in real time with **zero frontend build dependencies**.",
                prompt
            );

            for chunk in simulated_reply.split_inclusive(' ') {
                thread::sleep(Duration::from_millis(45));
                ctx.append("chat", chunk);
            }

            Ok(())
        })

        // Clear chat history
        .on_click("clear", |ctx| {
            let empty: Vec<ChatMessage> = Vec::new();
            ctx.set("chat", empty);
            ctx.set("prompt", "");
            Ok(())
        })

        .launch("127.0.0.1:7860")
}
