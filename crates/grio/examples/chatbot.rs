use grio::*;
use std::thread;
use std::time::Duration;

fn main() -> grio::Result<()> {
    App::new("Chatbot LLM · grio demo")
        .subtitle("Démonstration d'interface conversationnelle IA avec bulles de messages et streaming temps réel (token par token).")
        .item(
            Chatbot::new("chat")
                .label("Assistant IA (Llama / Rust)")
                .height(480)
                .message("assistant", "Bonjour ! Je suis votre assistant local. Comment puis-je vous aider ?")
        )

        .row(|r| {
            r.item(Text::new("prompt").placeholder("Posez une question à l'IA... (ex: Écris du Rust)"));
            r.item(Button::new("send").label("Envoyer").primary());
            r.item(Button::new("clear").label("Effacer"));
        })

        // Envoi du message utilisateur + streaming de la réponse du bot
        .on_click("send", |ctx| {
            let prompt: String = ctx.get("prompt").unwrap_or_default();
            if prompt.trim().is_empty() {
                return Ok(());
            }

            // 1. On récupère les messages existants ou on initialise
            let mut history: Vec<ChatMessage> = ctx.get("chat").unwrap_or_default();
            history.push(ChatMessage::user(&prompt));
            history.push(ChatMessage::assistant("")); // message vide prêt à recevoir le flux
            ctx.set("chat", history);
            ctx.set("prompt", ""); // reset le champ de saisie

            // 2. Simulation de streaming de tokens (comme Candle ou llama.cpp)
            let simulated_reply = format!(
                "Voici une réponse générée pour votre question **\"{}\"** :\n\n```rust\nfn main() {{\n    println!(\"IA performante en Rust pur !\");\n}}\n```\nTout fonctionne avec **zéro dépendance front**.",
                prompt
            );

            for chunk in simulated_reply.split_inclusive(' ') {
                thread::sleep(Duration::from_millis(45));
                ctx.append("chat", chunk);
            }

            Ok(())
        })

        // Effacer la conversation
        .on_click("clear", |ctx| {
            let empty: Vec<ChatMessage> = Vec::new();
            ctx.set("chat", empty);
            ctx.set("prompt", "");
            Ok(())
        })

        .launch("127.0.0.1:7860")
}
