//! # RichText, DataEditor & Dynamic Slots — Phase 9 Lot 2 Example
//!
//! Demonstrates:
//! 1. `RichText`: Markdown incident reporting with formatting toolbar (Bold, Italic, Headers, Code, Links).
//! 2. `DataEditor`: Interactive typed data grid with active boolean checkboxes, dropdowns, and TSV/CSV clipboard paste.
//! 3. `DynamicContainer`: Real-time reactive component injection via `ctx.append_component` and `ctx.replace_children`.
//! 4. `ctx.set_visible`: Dynamic visibility toggling for widgets and containers.

use grio::*;
use serde_json::json;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = App::new("IT Support & Incident Management");

    // Row 1: RichText Micro-Editor & Controls
    app = app.item(
        Row::new("r_editor")
            .item(
                RichText::new("ticket_desc")
                    .label("Detailed Incident Description (Markdown)")
                    .placeholder("Describe steps to reproduce the issue...")
                    .value("### VPN Connection Outage\n\n- **User:** John Doe\n- **Error Message:** `TLS handshake failed`\n- **Impact:** Critical for remote engineering team.")
                    .lines(7),
            )
            .item(
                Panel::new("p_actions")
                    .label("Actions & Controls")
                    .item(Text::new("requester").label("Requester Email").value("john.doe@enterprise.com"))
                    .item(Dropdown::new("category").label("Category").options(&["Networking & VPN", "Identity & Access", "Hardware", "Software"]))
                    .item(Button::new("btn_toggle_panel").label("👁 Toggle Catalog Visibility"))
                    .item(Button::new("btn_add_slot").label("➕ Inject Dynamic Component"))
                    .item(Button::new("btn_clear_slot").label("🗑 Clear Slot").variant("secondary")),
            ),
    );

    // Row 2: DataEditor Grid (Service Catalog & SLA)
    let initial_data = vec![
        vec![
            json!("SRV-01"),
            json!("Password Reset"),
            json!(true),
            json!(1),
            json!("P1 - Critical"),
        ],
        vec![
            json!("SRV-02"),
            json!("Remote VPN Access"),
            json!(true),
            json!(4),
            json!("P2 - High"),
        ],
        vec![
            json!("SRV-03"),
            json!("Security Badge Request"),
            json!(false),
            json!(24),
            json!("P3 - Normal"),
        ],
        vec![
            json!("SRV-04"),
            json!("New Workstation Provisioning"),
            json!(true),
            json!(48),
            json!("P4 - Low"),
        ],
    ];

    app = app.item(
        Row::new("r_catalog")
            .item(
                DataEditor::new("services_grid")
                    .label("IT Service Catalog & SLA Rules (Double-click to edit, active checkboxes, Ctrl+V supported)")
                    .column("id", "Ref.", ColumnType::Text)
                    .column("name", "Service Name", ColumnType::Text)
                    .column("active", "Active", ColumnType::Boolean)
                    .column("sla", "Max SLA (hrs)", ColumnType::Number)
                    .column("priority", "Priority", ColumnType::Dropdown(vec![
                        "P1 - Critical".into(),
                        "P2 - High".into(),
                        "P3 - Normal".into(),
                        "P4 - Low".into(),
                    ]))
                    .data(initial_data)
                    .allow_add(true)
                    .allow_delete(true)
                    .allow_paste(true)
                    .max_height(280),
            ),
    );

    // Row 3: DynamicContainer (Hot-Slot Injection Area)
    app = app.item(
        Row::new("r_slots").item(
            Panel::new("p_slot_zone")
                .label("Dynamic Container Area (Live Components Injected at Runtime)")
                .item(
                    DynamicContainer::new("dynamic_slot").item(
                        Output::new("slot_initial")
                            .label("Initial Slot Content")
                            .value("Waiting for dynamic component injection..."),
                    ),
                ),
        ),
    );

    // Row 4: Submit Button & Summary Output
    app = app.item(
        Row::new("r_submit")
            .item(Button::new("btn_submit_ticket").label("Submit Ticket"))
            .item(Output::new("ticket_summary").label("Submission Snapshot")),
    );

    // STATE & REACTIVE HANDLERS
    let catalog_visible = Arc::new(AtomicBool::new(true));
    let cat_vis_clone = catalog_visible.clone();
    let slot_counter = Arc::new(AtomicUsize::new(1));
    let slot_cnt_clone = slot_counter.clone();

    // Handler: Toggle visibility
    app = app.on_click("btn_toggle_panel", move |ctx| {
        let current = cat_vis_clone.fetch_xor(true, Ordering::SeqCst);
        let new_state = !current;
        ctx.set_visible("r_catalog", new_state);
        ctx.alert(
            AlertLevel::Info,
            if new_state {
                "Service catalog visible"
            } else {
                "Service catalog hidden"
            },
        );
        Ok(())
    });

    // Handler: Inject dynamic component into slot
    app = app.on_click("btn_add_slot", move |ctx| {
        let idx = slot_cnt_clone.fetch_add(1, Ordering::SeqCst);
        let widget_id = format!("dyn_metric_{idx}");
        let new_widget = Metric::new(widget_id)
            .label(format!("Dynamic Metric #{idx}"))
            .value(format!("{} ms", idx * 12))
            .delta("+2.4%")
            .unit("P99 Latency");

        ctx.append_component("dynamic_slot", new_widget);
        ctx.alert(
            AlertLevel::Success,
            format!("Injected dynamic widget #{idx}!"),
        );
        Ok(())
    });

    // Handler: Clear slot container
    app = app.on_click("btn_clear_slot", |ctx| {
        ctx.clear_container("dynamic_slot");
        ctx.alert(AlertLevel::Warn, "Dynamic slot container cleared.");
        Ok(())
    });

    // Handler: Submit ticket
    app = app.on_click("btn_submit_ticket", |ctx| {
        let desc: String = ctx.get("ticket_desc").unwrap_or_default();
        let user = ctx.get_str("requester").unwrap_or("Unknown");
        let cat = ctx.get_str("category").unwrap_or("General");

        let summary = format!(
            "=== TICKET CREATED ===\nRequester: {user}\nCategory: {cat}\nDescription length: {} characters\nTimestamp: OK",
            desc.len()
        );
        ctx.set("ticket_summary", summary);
        ctx.alert(AlertLevel::Success, "Incident ticket recorded successfully!");
        Ok(())
    });

    println!("🚀 Launching Rich Data & Slots Demo on http://localhost:7860 ...");
    app.serve("127.0.0.1:7860").await
}
