use grio::*;
use std::fs;

fn main() -> Result<()> {
    let app = App::showcase();
    let html = app.render_html_bundle();

    // Add a banner with links to GitHub and Crates.io
    let banner = r#"
<div style="background: linear-gradient(135deg, #4f46e5 0%, #7c3aed 100%); color: white; padding: 12px 20px; text-align: center; font-family: system-ui, sans-serif; font-size: 14px; font-weight: 500; display: flex; justify-content: center; align-items: center; gap: 16px; flex-wrap: wrap;">
  <span>🦀 <strong>grio</strong> — Declarative Pure Rust UI Engine for AI & Big Data</span>
  <a href="https://github.com/lecyberbill/grio" target="_blank" style="background: rgba(255,255,255,0.2); color: white; padding: 4px 12px; border-radius: 6px; text-decoration: none; font-weight: 600;">⭐ GitHub</a>
  <a href="https://crates.io/crates/grio" target="_blank" style="background: rgba(255,255,255,0.2); color: white; padding: 4px 12px; border-radius: 6px; text-decoration: none; font-weight: 600;">📦 Crates.io</a>
  <code style="background: rgba(0,0,0,0.3); padding: 3px 8px; border-radius: 4px; font-size: 12px;">cargo run --example showcase</code>
</div>
"#;

    let final_html = html.replace("<body", &format!("{banner}\n<body"));

    fs::create_dir_all("deploy/hf-space-static").expect("create dir");
    fs::write("deploy/hf-space-static/index.html", final_html).expect("write html");
    println!("✓ Generated deploy/hf-space-static/index.html successfully!");
    Ok(())
}
