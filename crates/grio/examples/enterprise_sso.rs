// [WFGY] Zone: SAFE | λ: 0.2 | Fallbacks: 0 | Action: Enterprise SSO and RBAC authentication example
//! # Example: Enterprise SSO Authentication & Role-Based Access Control (RBAC)
//!
//! This example demonstrates how to:
//! 1. Enable optional Enterprise Single Sign-On (`app.auth(...)`) with Mock/Dev users or SSO providers.
//! 2. Protect individual multi-page routes with required roles (`.require_role("admin")`).
//! 3. Inspect user session identity, roles, and permissions inside event handlers (`ctx.user()`, `ctx.has_role("admin")`).
//! 4. Seamlessly switch user profiles and test access control without an external identity server.
//!
//! To run this example:
//! ```bash
//! cargo run --example enterprise_sso
//! ```

use grio::*;

fn main() -> Result<()> {
    // 1. Configure pre-seeded Enterprise users for dev/testing
    let dev_user = UserProfile::new("usr_42", "alice_engineer")
        .email("alice@enterprise.corp")
        .roles(&["engineer", "user"])
        .avatar("https://api.dicebear.com/7.x/identicon/svg?seed=alice");

    let admin_user = UserProfile::new("usr_99", "bob_admin")
        .email("bob.admin@enterprise.corp")
        .roles(&["admin", "security_lead"])
        .avatar("https://api.dicebear.com/7.x/identicon/svg?seed=bob");

    // 2. Build the Multi-Page Application with SSO and RBAC
    App::new("🏢 Enterprise AI Control Plane")
        .subtitle("Role-Based Access Control (RBAC) and Single Sign-On (SSO) Demo")
        .theme(Theme::corporate())
        // Enable Enterprise Auth (Opt-in) with Mock Users
        .auth(
            AuthConfig::enabled()
                .with_mock_users(vec![dev_user, admin_user])
        )
        // Public / Engineer Page
        .page("/", "📊 Model Inference", |p| {
            p.icon("cpu-chip");
            p.panel("AI Pipeline Runner", |panel| {
                panel.item(Text::new("prompt").label("Prompt").value("Summarize Q3 financial highlights"));
                panel.item(Slider::new("temp").label("Temperature").min(0.0).max(1.0).value(0.7));
                panel.item(Button::new("btn_run").label("🚀 Run Model Inference").primary());
                panel.item(Output::new("model_output").label("Inference Result"));
            });
        })
        // Protected Admin / Security Page
        .page("/admin", "🛡️ Security & Cluster Audit", |p| {
            p.icon("lock-closed");
            p.require_role("admin"); // RBAC: Only users with 'admin' role can access!
            
            p.panel("Cluster Telemetry (Admin Only)", |panel| {
                panel.item(Metric::new("gpu_cluster").label("Active H100 Nodes").value("64 / 64"));
                panel.item(Metric::new("vram_usage").label("Cluster VRAM").value("5.12 TB (82%)"));
                panel.item(Button::new("btn_flush_cache").label("⚠️ Flush Global Model Cache").primary());
                panel.item(Output::new("admin_audit_log").label("Security Audit Log"));
            });
        })
        // Handlers with programmatic role validation
        .on_click("btn_run", |ctx| {
            let prompt: String = ctx.get("prompt").unwrap_or_default();
            let user_info = match ctx.user() {
                Some(u) => format!("Executed by {} ({:?})", u.username, u.roles),
                None => "Executed anonymously".to_string(),
            };

            ctx.set("model_output", format!("Synthesizing inference for prompt: \"{prompt}\"\n\n[Audit Stamp: {user_info}]"));
            ctx.alert(AlertLevel::Success, "Inference completed successfully!");
            Ok(())
        })
        .on_click("btn_flush_cache", |ctx| {
            if !ctx.has_role("admin") {
                ctx.alert(AlertLevel::Error, "Access Denied: You do not possess the required 'admin' role!");
                ctx.set("admin_audit_log", "🚨 [ALERT 403] Unauthorized cache flush attempt logged to SIEM.");
                return Ok(());
            }

            let admin = ctx.user().map(|u| u.username.as_str()).unwrap_or("admin");
            ctx.set("admin_audit_log", format!("✅ [200 OK] Cache flushed across all 64 nodes by authorized user: {admin}"));
            ctx.alert(AlertLevel::Warn, "Global cluster cache has been purged.");
            Ok(())
        })
        .launch("127.0.0.1:7860")
}
