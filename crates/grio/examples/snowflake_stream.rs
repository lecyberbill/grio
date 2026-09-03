//! # Snowflake & Big Data Stream Analytics — Phase 10 Example
//!
//! High-throughput Big Data demonstration:
//! - Virtualized grid (`DataEditor` with 60 FPS Virtual Scroll for 100k+ rows)
//! - Snowflake / DuckDB / Polars real-time analytical stream simulation
//! - Live chunked batch ingestion via WebSocket (`ctx.append_rows`)
//! - Instant client-side full-text search (< 5ms)
//! - One-click native CSV export

use grio::*;
use serde_json::json;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = App::new("Snowflake & Real-Time Big Data Stream")
        .subtitle("High-Throughput Analytical Dashboard — Virtualized 60 FPS Grid for 100k+ Rows")
        .max_width(1400)
        .run_label("Re-execute Analytical Query");

    let counter = Arc::new(AtomicUsize::new(1001));

    // Generate initial dataset of 1,000 transactions
    let mut initial_data = Vec::with_capacity(1000);
    let regions = [
        "EU-West (Paris)",
        "US-East (N. Virginia)",
        "AP-South (Tokyo)",
        "EU-Central (Frankfurt)",
    ];
    let status_choices = ["Success", "Pending", "Flagged"];

    for i in 1..=1000 {
        let region = regions[i % regions.len()];
        let status = status_choices[i % status_choices.len()];
        let amount = ((i * 37) % 5000) as f64 + 14.50;
        let latency = ((i * 13) % 180) + 12;

        initial_data.push(vec![
            json!(format!("TXN-{:06}", i)),
            json!(format!("WAREHOUSE_{}", (i % 4) + 1)),
            json!(region),
            json!(amount),
            json!(latency),
            json!(status == "Success"),
            json!(status),
        ]);
    }

    // 1. KPI & THROUGHPUT METRICS
    app = app.item(
        Row::new("r_kpis")
            .item(
                Metric::new("m_rows")
                    .label("Indexed Transactions")
                    .value("1,000")
                    .unit("rows")
                    .delta("+100/s")
                    .delta_color("pos"),
            )
            .item(
                Metric::new("m_throughput")
                    .label("Snowflake Ingestion Throughput")
                    .value("42.8")
                    .unit("MB/s")
                    .delta("Optimal")
                    .delta_color("pos"),
            )
            .item(
                Metric::new("m_latency")
                    .label("Query P99 Latency")
                    .value("14")
                    .unit("ms")
                    .delta("-4ms")
                    .delta_color("pos"),
            )
            .item(
                Metric::new("m_engine")
                    .label("Analytics Engine")
                    .value("Snowflake")
                    .unit("Arrow IPC")
                    .delta("Connected")
                    .delta_color("neutral"),
            ),
    );

    // 2. STREAMING ACTION CONTROLS
    app = app.item(
        Row::new("r_stream_controls")
            .item(
                Button::new("btn_stream_batch")
                    .label("⚡ Stream 500-Row Batch (Snowflake Ingestion)"),
            )
            .item(
                Button::new("btn_simulate_burst")
                    .label("🚀 Simulate 5,000-Row Bulk Burst")
                    .variant("secondary"),
            ),
    );

    // 3. VIRTUALIZED DATA GRID
    app = app.item(
        Panel::new("p_grid_panel")
            .label("Transaction Analytics Stream (Virtualized 60 FPS Grid with Instant Search & Sorting)")
            .item(
                DataEditor::new("data_grid")
                    .label("Financial Transactions (Live 🔍 Filter, Virtual Scroll & Sorting)")
                    .column("id", "Transaction ID", ColumnType::Text)
                    .column("warehouse", "Snowflake Warehouse", ColumnType::Text)
                    .column("region", "Cloud Region", ColumnType::Text)
                    .column("amount", "Amount ($)", ColumnType::Number)
                    .column("latency", "Latency (ms)", ColumnType::Number)
                    .column("verified", "Verified", ColumnType::Boolean)
                    .column("status", "Status", ColumnType::Dropdown(vec![
                        "Success".into(),
                        "Pending".into(),
                        "Flagged".into(),
                    ]))
                    .data(initial_data)
                    .allow_add(true)
                    .allow_delete(true)
                    .allow_paste(true)
                    .max_height(480),
            ),
    );

    // 4. CHARTS & AUDIT SUMMARY
    app =
        app.item(
            Row::new("r_charts")
                .item(
                    Plot::new("plot_distribution")
                        .label("Volume Distribution by Cloud Region")
                        .data(&serde_json::json!({
                            "variant": "bar",
                            "labels": ["EU-West", "US-East", "AP-South", "EU-Central"],
                            "series": [
                                { "name": "Volume ($k)", "data": [480.0, 720.0, 310.0, 540.0] }
                            ]
                        })),
                )
                .item(Output::new("out_summary").label("Stream Audit Log").value(
                    "🟢 Pipeline active · Continuously ingesting Snowflake warehouse events.",
                )),
        );

    // REACTIVE HANDLERS

    // Handler 1: Ingest 500-row batch
    let cnt_batch = counter.clone();
    app = app.on_click("btn_stream_batch", move |ctx| {
        let mut batch = Vec::with_capacity(500);
        let start = cnt_batch.fetch_add(500, Ordering::SeqCst);

        for i in start..(start + 500) {
            let region = regions[i % regions.len()];
            let status = status_choices[i % status_choices.len()];
            let amount = ((i * 41) % 7000) as f64 + 18.20;
            let latency = ((i * 17) % 150) + 10;

            batch.push(vec![
                json!(format!("TXN-{:06}", i)),
                json!(format!("WAREHOUSE_{}", (i % 4) + 1)),
                json!(region),
                json!(amount),
                json!(latency),
                json!(status == "Success"),
                json!(status),
            ]);
        }

        ctx.append_rows("data_grid", batch);
        ctx.set("m_rows", format!("{}", start + 500));
        ctx.set(
            "out_summary",
            format!(
                "⚡ Ingested 500 transactions successfully (Total: {} rows).",
                start + 500
            ),
        );
        ctx.alert(
            AlertLevel::Success,
            "500 transactions streamed in real time!",
        );
        Ok(())
    });

    // Handler 2: Ingest 5,000-row bulk burst
    let cnt_burst = counter.clone();
    app = app.on_click("btn_simulate_burst", move |ctx| {
        let mut batch = Vec::with_capacity(5000);
        let start = cnt_burst.fetch_add(5000, Ordering::SeqCst);

        for i in start..(start + 5000) {
            let region = regions[i % regions.len()];
            let status = status_choices[i % status_choices.len()];
            let amount = ((i * 43) % 9000) as f64 + 22.00;
            let latency = ((i * 19) % 120) + 8;

            batch.push(vec![
                json!(format!("TXN-{:06}", i)),
                json!(format!("WAREHOUSE_{}", (i % 4) + 1)),
                json!(region),
                json!(amount),
                json!(latency),
                json!(status == "Success"),
                json!(status),
            ]);
        }

        ctx.append_rows("data_grid", batch);
        ctx.set("m_rows", format!("{}", start + 5000));
        ctx.set(
            "out_summary",
            format!(
                "🚀 Bulk burst of 5,000 transactions ingested (Total: {} rows).",
                start + 5000
            ),
        );
        ctx.alert(
            AlertLevel::Success,
            "5,000 transactions rendered with zero DOM lag!",
        );
        Ok(())
    });

    println!("🚀 Launching Snowflake Stream Analytics Demo on http://localhost:7860 ...");
    app.serve("127.0.0.1:7860").await
}
