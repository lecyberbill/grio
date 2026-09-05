// [WFGY] Zone: SAFE | λ: 0.2 | Fallbacks: 0 | Action: Chromatix Visual Passkey Demo
//! Chromatix Visual Passkey Authentication & Badge Generator Showcase.
//!
//! Run with: `cargo run -p grio --example chromatix_passkey`
//!
//! This example demonstrates:
//! 1. Signing and generating a Chromatix Visual Passkey (PNG Badge) with embedded timestamping.
//! 2. Visual Drag-and-Drop login on `/auth/login`.
//! 3. Secure RBAC access control for Lab Admin vs Researcher.

use grio::auth::{AuthConfig, AuthManager, UserProfile};
use grio::components::*;
use grio::{App, Result};

const LAB_MASTER_KEY: &str = "cps_secret_lab_key_super_secure_2026";

#[tokio::main]
async fn main() -> Result<()> {
    let auth_cfg = AuthConfig::enabled().with_chromatix_pixel(LAB_MASTER_KEY);
    let auth_mgr = AuthManager::new(auth_cfg.clone());

    // Generate two sample badges for demonstration
    let alice = UserProfile::new("alice_lead", "Dr. Alice (Lead Scientist)")
        .email("alice@quantum-lab.org")
        .roles(&["researcher", "admin"]);
    let bob = UserProfile::new("bob_tech", "Bob (Lab Technician)")
        .email("bob@quantum-lab.org")
        .roles(&["technician"]);

    let alice_badge_bytes = auth_mgr.create_chromatix_badge(alice, LAB_MASTER_KEY, 3600 * 24 * 7); // 7 days
    let bob_badge_bytes = auth_mgr.create_chromatix_badge(bob, LAB_MASTER_KEY, 3600 * 24); // 24 hours

    let alice_b64 = grio::media::encode(&alice_badge_bytes);
    let bob_b64 = grio::media::encode(&bob_badge_bytes);

    App::new("🧬 Quantum Lab Workstation")
        .subtitle("Chromatix Pixel Standard Visual Passkey Authentication")
        .auth(auth_cfg)
        // Page 1: Badge Distribution Kiosk (Public / Pre-auth)
        .page("/", "🎫 Badge Kiosk", |p| {
            p.icon("ticket");
            p.item(Markdown::new("## 💎 Chromatix Visual Passkey Kiosk\n\n\
                In air-gapped laboratories or offline facilities, access is granted via **signed PNG badges**.\n\
                Download one of the passkey badges below, then head over to **Sign in** and drop your badge!"));

            p.row(|row| {
                row.panel("Dr. Alice Badge (Admin + Lead)", |panel| {
                    panel.item(Markdown::new(
                        "**Roles:** `researcher`, `admin`  \n**Validity:** 7 Days  \n**HMAC Signed:** `cps_secret_lab_key`",
                    ));
                    panel.item(
                        Image::new("alice_badge")
                            .label("Dr. Alice Chromatix Badge (Right click -> Save Image)")
                            .value(format!("data:image/png;base64,{alice_b64}"))
                            .interactive(false),
                    );
                });
                row.panel("Bob Badge (Technician)", |panel| {
                    panel.item(Markdown::new(
                        "**Roles:** `technician`  \n**Validity:** 24 Hours  \n**HMAC Signed:** `cps_secret_lab_key`",
                    ));
                    panel.item(
                        Image::new("bob_badge")
                            .label("Bob Chromatix Badge (Right click -> Save Image)")
                            .value(format!("data:image/png;base64,{bob_b64}"))
                            .interactive(false),
                    );
                });
            });
        })
        // Page 2: Secure Lab Instruments (Requires 'admin' role)
        .page("/instruments", "🔬 Lab Instruments", |p| {
            p.icon("beaker");
            p.require_role("admin"); // RBAC protected!
            
            p.panel("Particle Injector Settings (Admin Only)", |panel| {
                panel.item(Markdown::new("## 🔒 High-Energy Laser & Centrifuge Control"));
                panel.item(Slider::new("laser_freq").label("Laser Pulse Frequency (THz)").value(450.0).min(100.0).max(1000.0));
                panel.item(Button::new("btn_fire").label("💥 Fire Calibration Beam").primary());
                panel.item(Output::new("laser_log").label("Instrument Output Log").value("System idle. Ready for beam calibration."));
            });
        })
        // Page 3: Telemetry & Samples (Requires any authenticated role)
        .page("/telemetry", "📊 Telemetry & Logs", |p| {
            p.icon("chart-bar");
            p.panel("Telemetry Feeds", |panel| {
                panel.item(Metric::new("chamber_press").label("Chamber Pressure").value("1.02 bar"));
                panel.item(Metric::new("cryo_temp").label("Cryo Temp").value("-269.15 °C"));
            });
        })
        .on_click("btn_fire", |ctx| {
            if let Some(user) = ctx.user() {
                ctx.set("laser_log", format!("Beam fired successfully by **{}** (Roles: {:?})!", user.username, user.roles));
            } else {
                ctx.set("laser_log", "Unauthorized beam trigger!");
            }
            Ok(())
        })
        .launch("0.0.0.0:7890")
}
