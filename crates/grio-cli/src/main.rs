use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_help();
        return;
    }

    match args[1].as_str() {
        "new" => {
            if args.len() < 3 {
                eprintln!("Usage: grio new <project-name> [--template <greet|chatbot|vision>]");
                std::process::exit(1);
            }
            let name = &args[2];
            let template = if args.len() >= 5 && args[3] == "--template" {
                &args[4]
            } else {
                "greet"
            };
            create_project(name, template);
        }
        "version" | "--version" | "-V" => {
            println!("grio CLI v0.1.0 — declarative Rust AI app engine");
        }
        "help" | "--help" | "-h" => {
            print_help();
        }
        cmd => {
            eprintln!("Unknown command: `{cmd}`");
            print_help();
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!(r#"grio CLI — Fast, declarative AI app generator in Rust

USAGE:
    grio <COMMAND> [OPTIONS]

COMMANDS:
    new <name> [--template <name>]  Create a new grio app project
                                    Templates: greet (default), chatbot, vision
    version                         Display CLI version
    help                            Show this help message

EXAMPLES:
    grio new my-chat-app --template chatbot
    cd my-chat-app
    cargo run
"#);
}

fn create_project(name: &str, template: &str) {
    let path = Path::new(name);
    if path.exists() {
        eprintln!("Error: Directory `{name}` already exists.");
        std::process::exit(1);
    }

    fs::create_dir_all(path.join("src")).expect("failed to create src directory");

    let cargo_toml = format!(r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"

[dependencies]
grio = "0.1.0"
"#);

    let main_rs = match template {
        "chatbot" => r#"use grio::*;

fn main() -> grio::Result<()> {
    App::new("Chatbot AI")
        .subtitle("Built with grio")
        .item(
            Chatbot::new("chat")
                .label("Local AI Assistant")
                .message("assistant", "Hello! How can I assist you today?")
        )
        .row(|r| {
            r.item(Text::new("prompt").placeholder("Type your question..."));
            r.item(Button::new("send").label("Send").primary());
        })
        .on_click("send", |ctx| {
            let prompt: String = ctx.get("prompt").unwrap_or_default();
            if prompt.trim().is_empty() { return Ok(()); }
            let mut hist: Vec<ChatMessage> = ctx.get("chat").unwrap_or_default();
            hist.push(ChatMessage::user(&prompt));
            hist.push(ChatMessage::assistant(format!("Echo response to: {prompt}")));
            ctx.set("chat", hist);
            ctx.set("prompt", "");
            Ok(())
        })
        .launch("127.0.0.1:7860")
}
"#,
        "vision" => r#"use grio::*;

fn main() -> grio::Result<()> {
    App::new("Vision Model Demo")
        .row(|r| {
            r.item(Image::new("photo").label("Upload Image"));
            r.item(Output::new("classification").label("Model Prediction"));
        })
        .on_submit(|ctx| {
            ctx.set("classification", "Object detected: Cat (98.4%)");
            Ok(())
        })
        .launch("127.0.0.1:7860")
}
"#,
        _ => r#"use grio::*;

fn main() -> grio::Result<()> {
    App::new("My grio App")
        .item(Text::new("name").label("Your Name").value("World"))
        .item(Slider::new("intensity").label("Exclamation Level").min(1.0).max(5.0).value(2.0))
        .item(Output::new("greeting").label("Greeting"))
        .on_submit(|ctx| {
            let name: String = ctx.get("name")?;
            let intensity: f64 = ctx.get("intensity")?;
            ctx.set("greeting", format!("Hello {name} {}", "!".repeat(intensity as usize)));
            Ok(())
        })
        .launch("127.0.0.1:7860")
}
"#,
    };

    fs::write(path.join("Cargo.toml"), cargo_toml).expect("failed to write Cargo.toml");
    fs::write(path.join("src/main.rs"), main_rs).expect("failed to write src/main.rs");

    println!("✓ Project `{name}` created successfully with template `{template}`!");
    println!("  cd {name}");
    println!("  cargo run");
}
