use grio::*;
use std::fs;

fn main() -> Result<()> {
    let app = App::showcase();
    let html = app.render_html_bundle();

    // 1. Prominent Top Notification & Disclaimer Banner
    let banner = r#"
<div style="background: linear-gradient(135deg, #1e1b4b 0%, #312e81 50%, #4338ca 100%); color: #ffffff; padding: 14px 20px; border-bottom: 2px solid #6366f1; font-family: system-ui, -apple-system, sans-serif; font-size: 13.5px; box-shadow: 0 4px 16px rgba(0,0,0,0.35);">
  <div style="max-width: 1280px; margin: 0 auto; display: flex; align-items: center; justify-content: space-between; gap: 16px; flex-wrap: wrap;">
    <div style="display: flex; align-items: center; gap: 12px;">
      <span style="font-size: 22px;">🦀</span>
      <div>
        <div style="font-weight: 700; font-size: 14.5px; color: #a5b4fc; display: flex; align-items: center; gap: 8px;">
          <span>grio — Declarative Pure Rust UI Engine</span>
          <span style="background: #f59e0b; color: #78350f; font-size: 10.5px; font-weight: 800; padding: 2px 7px; border-radius: 4px; text-transform: uppercase; letter-spacing: 0.05em;">Static Showcase Preview</span>
        </div>
        <div style="color: #cbd5e1; font-size: 12.5px; margin-top: 2px;">
          ⚠️ <strong>Notice:</strong> This web page is a visual UI showcase. Backend Rust event loops & real-time WebSocket streams are disabled in this static preview.
        </div>
      </div>
    </div>
    <div style="display: flex; align-items: center; gap: 10px; flex-wrap: wrap;">
      <a href="https://github.com/lecyberbill/grio" target="_blank" style="background: rgba(255,255,255,0.15); hover:background: rgba(255,255,255,0.25); color: white; padding: 6px 14px; border-radius: 6px; text-decoration: none; font-weight: 600; font-size: 12.5px; border: 1px solid rgba(255,255,255,0.2); display: inline-flex; align-items: center; gap: 6px;">
        ⭐ Star on GitHub
      </a>
      <a href="https://crates.io/crates/grio" target="_blank" style="background: #6366f1; color: white; padding: 6px 14px; border-radius: 6px; text-decoration: none; font-weight: 600; font-size: 12.5px; border: 1px solid #818cf8; display: inline-flex; align-items: center; gap: 6px;">
        📦 Crates.io
      </a>
      <div style="background: rgba(0,0,0,0.4); padding: 5px 10px; border-radius: 5px; font-family: monospace; font-size: 12px; color: #38bdf8; border: 1px solid rgba(56,189,248,0.3);">
        cargo run --example showcase
      </div>
    </div>
  </div>
</div>
"#;

    // 2. Client-side interactive simulator script for static preview (Intercepts clicks on buttons to show feedback toasts)
    let static_enhancement_script = r#"
<script>
(function() {
  // Static preview interactive helper
  document.addEventListener('DOMContentLoaded', () => {
    // Show gentle guidance toast on interactive clicks
    const interactiveButtons = document.querySelectorAll('.mg-btn, .mg-run-btn, button[data-action]');
    interactiveButtons.forEach(btn => {
      btn.addEventListener('click', (e) => {
        if (window.grio && window.grio.toast) {
          window.grio.toast('💡 Static UI Preview: Rust backend is offline. Run `cargo run --example showcase` locally for real-time WebSocket execution!', 'info');
        }
      });
    });
  });
})();
</script>
"#;

    let final_html = html
        .replace("<body", &format!("{banner}\n<body"))
        .replace("</body>", &format!("{static_enhancement_script}\n</body>"));

    fs::create_dir_all("deploy/hf-space-static").expect("create dir");
    fs::write("deploy/hf-space-static/index.html", final_html).expect("write html");
    println!("✓ Generated deploy/hf-space-static/index.html with static disclaimer banner!");
    Ok(())
}
