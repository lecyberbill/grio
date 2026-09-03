//! Phase 3 — Média : upload d'image (analyse via `media::inspect`),
//! streaming micro/caméra (stats `StreamInfo`), événements play/pause/stop.
//!
//! Lancement : `cargo run -p grio --example media`

fn main() -> grio::Result<()> {
    use grio::*;

    App::new("Média · grio")

        // Analyse image + stats caméra (bouton « Analyser » et bouton Run
        // généré par `on_submit` partagent la même fonction).
        .on_click("analyze", analyze)
        .on_submit(analyze)

        // À chaque fragment de micro live : lecture des stats serveur + mise
        // à jour de la sortie. Flux déclaré sur le handler `on_stream`.
        .on_stream("mic_in", |ctx| {
            if let Ok(s) = ctx.get::<StreamInfo>("mic_in") {
                ctx.set("stats_out", format!("micro : {} · {} · {} fragments", s.mime, s.kb(), s.chunks));
                if s.chunks % 5 == 0 {
                    ctx.alert(AlertLevel::Info, format!("micro live : {} ({})", s.kb(), s.mime));
                }
            }
            Ok(())
        })
        .flow(&["mic_in"], &["stats_out"])

        .on_play("cam_view", |ctx| {
            ctx.alert(AlertLevel::Info, "lecture démarrée (play)");
            Ok(())
        })
        .on_pause("cam_view", |_| Ok(()))
        .on_stop("cam_view", |ctx| {
            ctx.alert(AlertLevel::Warn, "flux arrêté (stop)");
            Ok(())
        })

        .subtitle("Composants image / audio / vidéo : upload, streaming live et événements transports.")
        .item(Markdown::new("intro").text(
            "# Média\n\nUpload une **image** (le serveur renvoie type, taille et dimensions), teste le **micro** ou la **caméra live**, et observe les statistiques de flux ainsi que les événements `play` / `pause` / `stop`.",
        ))
        .row(|r| {
            r.item(Image::new("img_in").label("Image (upload)").interactive(true));
            r.item(Output::new("facts_out").label("Analyse serveur"));
        })
        .row(|r| {
            r.item(Audio::new("mic_in").label("Micro live").interactive(true).live(true));
            r.item(Video::new("cam_in").label("Caméra live").output().live(true));
        })
        .row(|r| {
            r.item(Output::new("stats_out").label("Stats micro (streaming)"));
            r.item(Output::new("cam_stats").label("Stats caméra"));
        })
        .item(Video::new("cam_view").label("Lecteur vidéo (play / pause / stop)"))
        .item(Button::new("analyze").label("Analyser"))
        .panel("Comment ça marche", |p| {
            p.item(Markdown::new("how").text(
                "- **image** : la data URL part dans un `change` ; le serveur la lit via `ctx.get_str` puis `media::inspect` (type, octets, dimensions PNG/GIF/JPEG).\n- **micro / caméra** : `MediaRecorder` produit des fragments poussés par WebSocket `{ t:stream }` ; le serveur cumule `StreamInfo { mime, bytes, chunks }`, consultable par `ctx.get::<StreamInfo>`.\n- **lecteur** : les boutons **play / pause / stop** émettent les événements média → `App::on_play` / `on_pause` / `on_stop`.",
            ));
        })

        .launch("0.0.0.0:7860")
}

/// Analyse partagée par le bouton « Analyser » et par la soumission Run.
///
/// 1) Image uploadée : des données brutes (data URL) on extrait type, taille
///    et dimensions côté serveur via `media::inspect`.
/// 2) Stats streaming caméra (fragments reçus via `{t:stream}`).
fn analyze(ctx: &mut grio::Context) -> grio::Result<()> {
    use grio::*;

    let img = ctx
        .get_str("img_in")
        .filter(|s| !s.is_empty())
        .map(str::to_string);
    match img {
        Some(raw) => {
            ctx.set(
                "facts_out",
                format!("image reçue : data URL de {} octets", raw.len()),
            );
            if let Some(info) = grio::media::inspect(&raw) {
                let dims = match (info.width, info.height) {
                    (Some(w), Some(h)) => format!("{w}x{h}"),
                    _ => "inconnues".to_string(),
                };
                ctx.append(
                    "facts_out",
                    format!(
                        "\n  -> {} · {} · {dims} · {} octets",
                        info.kind(),
                        info.mime,
                        info.size_bytes
                    ),
                );
                ctx.alert(AlertLevel::Success, format!("image {}", info.kind()));
            } else {
                ctx.append("facts_out", "\n  -> format non reconnu");
            }
        }
        None => ctx.set("facts_out", "aucune image pour l'instant"),
    }

    let cam = ctx.get::<Option<StreamInfo>>("cam_in").ok().flatten();
    match cam {
        Some(s) => ctx.set(
            "cam_stats",
            format!("caméra : {} · {} · {} fragments", s.mime, s.kb(), s.chunks),
        ),
        None => ctx.set("cam_stats", "caméra : aucun flux actif"),
    }
    Ok(())
}
