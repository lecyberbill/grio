//! # Theme Studio & Design Tokens — Phase 12 Example
//!
//! Demonstrates:
//! - Curated design presets: `Theme::tokyo_night()`, `Theme::nord()`, `Theme::cyberpunk()`, `Theme::catppuccin_mocha()`, `Theme::corporate()`
//! - Design token customization (custom primary color `#7aa2f7`, border radius `10px`, typography `Inter`)
//! - Multi-tab dashboard styling & real-time responsiveness

use grio::*;

fn main() -> Result<()> {
    App::new("Theme Studio & Design Tokens")
        .subtitle("Showcasing Curated Visual Presets, Typography Tokens, and Glassmorphism Layouts")
        .theme(Theme::tokyo_night())
        .max_width(1280)
        .tabs(|t| {
            t
                // Tab 1: Design Tokens & Palette Preview
                .tab("🎨 Color Palettes & Tokens", |b| {
                    b.row(|r| {
                        r.item(
                            Metric::new("m_primary")
                                .label("Primary Accent")
                                .value("#7aa2f7")
                                .unit("Tokyo Night")
                                .delta("Active")
                                .delta_color("pos"),
                        );
                        r.item(
                            Metric::new("m_radius")
                                .label("Border Radius")
                                .value("10")
                                .unit("px")
                                .delta("Modern")
                                .delta_color("pos"),
                        );
                        r.item(
                            Metric::new("m_font")
                                .label("Typography")
                                .value("Inter")
                                .unit("Google Font")
                                .delta("Clean")
                                .delta_color("neutral"),
                        );
                    });
                    b.panel("Available Built-in Theme Presets", |p| {
                        p.item(
                            HighlightedText::new("theme_descriptions")
                                .label("Built-in Rust Presets")
                                .segments(&[
                                    ("Theme::tokyo_night() ", Some("PRESET")),
                                    ("-> Neon blue & purple palette for developers.\n", None),
                                    ("Theme::nord() ", Some("PRESET")),
                                    ("-> Arctic, calm bluish tone.\n", None),
                                    ("Theme::cyberpunk() ", Some("PRESET")),
                                    ("-> High-contrast dark theme with vivid rose/cyan.\n", None),
                                    ("Theme::catppuccin_mocha() ", Some("PRESET")),
                                    ("-> Soothing pastel dark palette.\n", None),
                                    ("Theme::corporate() ", Some("PRESET")),
                                    ("-> Clean, high-contrast light mode for enterprise.", None),
                                ]),
                        );
                    });
                })
                // Tab 2: Interactive Controls Preview
                .tab("🎛️ Interactive Controls", |b| {
                    b.row(|r| {
                        r.item(Text::new("t_name").label("Project Name").value("grio UI Studio"));
                        r.item(Slider::new("s_intensity").label("Accent Intensity (%)").min(0.0).max(100.0).value(75.0));
                    });
                    b.row(|r| {
                        r.item(Dropdown::new("dd_preset").label("Select Palette").options(&[
                            "Tokyo Night", "Nord Polar", "Cyberpunk", "Catppuccin Mocha", "Corporate Light"
                        ]));
                        r.item(Checkbox::new("chk_glass").label("Enable Glassmorphism Surfaces").value(true));
                    });
                    b.row(|r| {
                        r.item(Button::new("btn_apply").label("Apply Theme Token").primary());
                        r.item(Button::new("btn_reset").label("Reset Default").variant("secondary"));
                    });
                    b.item(Output::new("out_preview").label("Applied Token Snapshot"));
                })
        })
        .on_click("btn_apply", |ctx| {
            let name: String = ctx.get("t_name").unwrap_or_default();
            let preset = ctx.get_str("dd_preset").unwrap_or("Tokyo Night").to_string();
            let intensity = ctx.get::<f64>("s_intensity").unwrap_or(75.0);

            let summary = format!(
                "✓ Project: {name}\n✓ Active Palette: {preset}\n✓ Accent Intensity: {intensity}%\n✓ CSS Variables: Auto-injected at runtime",
            );
            ctx.set("out_preview", summary);
            ctx.alert(AlertLevel::Success, format!("Theme token applied: {preset}"));
            Ok(())
        })
        .launch("127.0.0.1:7860")
}
