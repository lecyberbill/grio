//! Phase 14 Demo : Zero-Copy Big Data Pipeline, WebGL GPU Accelerators & OLAP Pivot Tables.
//!
//! Lancez avec :
//! ```bash
//! cargo run --example bigdata_accelerator
//! ```

use grio::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<()> {
    // 1. Initialisation de 5 000 points de signaux haute fréquence
    let base_samples: Vec<f32> = (0..5000)
        .map(|i| {
            let t = i as f32 * 0.02;
            (t * 1.5).sin() * 0.7 + (t * 4.0).cos() * 0.25 + (t * 0.1).sin() * 0.4
        })
        .collect();

    // 2. Jeu de données OLAP Big Data Transactions (600 lignes initiales)
    let regions = ["Europe", "North America", "Asia-Pacific", "Latin America"];
    let categories = ["Cloud AI", "Data Platform", "Edge Compute", "Cybersecurity"];
    let quarters = ["Q1", "Q2", "Q3", "Q4"];

    let mut olap_data: Vec<Vec<serde_json::Value>> = Vec::new();
    let mut rng_seed: u64 = 1337;
    let mut pseudo_rand = || {
        rng_seed = rng_seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        (rng_seed >> 33) as f64 / 2147483648.0
    };

    for i in 0..600 {
        let reg = regions[(pseudo_rand() * 4.0) as usize % 4];
        let cat = categories[(pseudo_rand() * 4.0) as usize % 4];
        let qtr = quarters[(pseudo_rand() * 4.0) as usize % 4];
        let rev = (pseudo_rand() * 8500.0 + 1200.0).round();
        let margin = (rev * (0.25 + pseudo_rand() * 0.35)).round();

        olap_data.push(vec![
            serde_json::json!(format!("TX-2026-{:04}", i + 1)),
            serde_json::json!(reg),
            serde_json::json!(cat),
            serde_json::json!(qtr),
            serde_json::json!(rev),
            serde_json::json!(margin),
        ]);
    }

    App::new("Grio Ultra Data Engine — WebGL GPU & OLAP Accelerator")
        .subtitle("Phase 14 · Binary Zero-Copy Streaming, 1M+ Points at 60 FPS & Multidimensional Slicing")
        .theme(Theme::cyberpunk())
        .row(|r| {
            r.item(
                WebGlPlot::new("gpu_stream")
                    .title("⚡ WebGL2 Real-Time GPU Waveform (Binary Stream 60 FPS)")
                    .xlabel("Échantillons (t)")
                    .ylabel("Amplitude (V)")
                    .colors(&["#00f0ff", "#ff007f", "#7000ff"])
                    .height(360)
                    .max_points(300_000)
                    .show_fps(true)
                    .series("Signal Harmonique", "#00f0ff", &base_samples),
            );
        })
        .row(|r| {
            r.item(Button::new("btn_burst").label("🚀 Injecter 50 000 Points Binaire").primary());
            r.item(Button::new("btn_stream").label("▶ Démarrer Flux Haute Fréquence (100 Hz)").secondary());
        })
        .row(|r| {
            r.item(
                PivotTable::new("olap_pivot")
                    .label("📊 Explorateur OLAP & Tableaux Croisés Dynamiques (Cube Multidimensionnel)")
                    .headers(&["ID", "Région", "Catégorie", "Trimestre", "Chiffre d'Affaires (€)", "Marge (€)"])
                    .data(olap_data)
                    .rows(&["Région", "Catégorie"])
                    .cols(&["Trimestre"])
                    .value_field("Chiffre d'Affaires (€)")
                    .aggregator(PivotAggregator::Sum)
                    .height(380),
            );
        })
        .on_click("btn_burst", |ctx| {
            // Génère 50 000 points f32 en un seul coup
            let mut burst = Vec::with_capacity(50_000);
            for i in 0..50_000 {
                let t = i as f32 * 0.01;
                burst.push((t * 2.0).sin() * 0.8 + (t * 12.0).cos() * 0.2);
            }
            ctx.append_f32_points("gpu_stream", &burst);
            ctx.alert(AlertLevel::Success, "50 000 points injectés sans copie via GPU buffer !");
            Ok(())
        })
        .on_click("btn_stream", |ctx| {
            ctx.alert(AlertLevel::Info, "Flux haute fréquence actif (2 000 points/sec streamés en continu)...");
            // Stream continu de 40 trames de 500 points (20 000 points total)
            for batch in 0..40 {
                if ctx.cancelled() {
                    break;
                }
                let mut chunk = Vec::with_capacity(500);
                for i in 0..500 {
                    let t = (batch * 500 + i) as f32 * 0.05;
                    chunk.push((t * 0.8).sin() * 0.9 + (t * 3.5).sin() * 0.3);
                }
                ctx.append_f32_points("gpu_stream", &chunk);
                std::thread::sleep(Duration::from_millis(25));
            }
            Ok(())
        })
        .launch("127.0.0.1:7860")
}
