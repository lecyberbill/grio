//! Phase 4 — Widgets avancés : Checkbox, Dropdown (multiple/saisie libre),
//! Date/Time, Dataframe éditable, Plot (SVG), Gallery, liste triable par
//! glissé-déposé, Code colorisé, File Explorer (côté serveur).
//!
//! Lancement : `cargo run -p grio --example forms`

fn main() -> grio::Result<()> {
    use grio::*;

    App::new("Widgets · grio")

        .on_click("bar", |ctx| {
            ctx.set("chart", chart(1.0, "bar"));
            ctx.alert(AlertLevel::Success, "graphique en barres affiché");
            Ok(())
        })
        .on_click("line", |ctx| {
            ctx.set("chart", chart(0.0, "line"));
            ctx.alert(AlertLevel::Info, "graphique en lignes affiché");
            Ok(())
        })
        .on_click("shots", |ctx| {
            let idx = ctx.event().and_then(|e| e.d.as_ref().and_then(|d| d.as_u64())).unwrap_or(0);
            ctx.alert(AlertLevel::Info, format!("image n°{} sélectionnée", idx + 1));
            Ok(())
        })
        .on_change("ex", |ctx| {
            let path = ctx.event().and_then(|e| e.d.as_ref().and_then(|d| d.as_str()).map(String::from));
            ctx.alert(AlertLevel::Info, format!("fichier sélectionné : {}", path.unwrap_or_default()));
            Ok(())
        })
        .on_change("photo", |ctx| {
            if let Ok(v) = ctx.get::<serde_json::Value>("photo") {
                let n = v.get("layers").and_then(|l| l.as_array()).map(|a| a.len()).unwrap_or(0);
                let mask = v.get("mask").and_then(|m| m.as_str()).unwrap_or("");
                ctx.alert(AlertLevel::Info, format!("retouche : {} calque(s), masque {} octets — prêt pour de l'inpainting", n, mask.len()));
            }
            Ok(())
        })
        .on_submit(|ctx| {
            let mut s = String::from("=== Résumé des entrées ===\n");
            if let Ok(b) = ctx.get::<bool>("deal") { s.push_str(&format!("✓ accord : {b}\n")); }
            if let Ok(m) = ctx.get::<String>("model") { s.push_str(&format!("modèle : {m}\n")); }
            if let Ok(tags) = ctx.get::<Vec<String>>("tags") { s.push_str(&format!("tags : {}\n", tags.join(", "))); }
            if let Ok(d) = ctx.get::<String>("due") { s.push_str(&format!("échéance : {d}\n")); }
            if let Ok(t) = ctx.get::<String>("alarm") { s.push_str(&format!("alarme : {t}\n")); }
            if let Ok(v) = ctx.get::<serde_json::Value>("df") {
                let rows = v.get("data").and_then(|x| x.as_array()).map(|a| a.len()).unwrap_or(0);
                s.push_str(&format!("tableau : {rows} ligne(s)\n"));
            }
            if let Ok(o) = ctx.get::<Vec<String>>("prio") {
                s.push_str(&format!("priorités : {}\n", o.join(" > ")));
            }
            if let Ok(code) = ctx.get::<String>("editor") {
                s.push_str(&format!("code édité : {} caractères\n", code.len()));
            }
            if let Ok(g) = ctx.get::<Vec<String>>("shots") {
                s.push_str(&format!("galerie : {} image(s)\n", g.len()));
            }
            if let Ok(p) = ctx.get::<String>("ex") {
                s.push_str(&format!("fichier explorateur : {p}\n"));
            }
            ctx.set("summary", s);
            Ok(())
        })

        .subtitle("Dix widgets paramétrables, à l'image de Gradio — tout est configurable par builder et lisible depuis les handlers.")
        .item(Markdown::new("intro").text(
            "# Widgets\n\nChaque composant expose ses options via des méthodes builder, sa valeur dans le snapshot d'entrées, et des événements `change` / `click` / `submit` classiques.\n\n**Mise en page (modulaire)** : `WithLayout::new(brique).width/.height/.scale/.min_width` enveloppe n'importe quel composant ; les groupes `row/column/panel` acceptent les mêmes réglages via leur builder (`r.scale(2)`, `p.min_width(…)`…).",
        ))

        .row(|r| {
            r.scale(1);
            r.min_width(360);
            r.item(Checkbox::new("deal").label("J'accepte les conditions").value(true));
            r.item(Dropdown::new("model")
                .label("Modèle")
                .choices(&[("gpt-4o", "GPT-4o"), ("claude-3.5", "Claude 3.5"), ("mistral", "Mistral")])
                .value("claude-3.5"));
            r.item(Dropdown::new("tags")
                .label("Tags (multi, saisie libre)")
                .choices_str(&["rust", "ui", "web"])
                .multiple(true)
                .value_list(&["rust", "web"])
                .allow_custom(true));
        })
        .row(|r| {
            r.item(DatePicker::new("due").label("Échéance").min("2026-01-01").max("2026-12-31").value("2026-08-19"));
            r.item(TimePicker::new("alarm").label("Alarme").value("09:30"));
        })

        .item(WithLayout::new(Dataframe::new("df")
            .label("Panier (éditable)")
            .headers(&["Produit", "Quantité", "Prix"])
            .data(&serde_json::json!([
                ["Pommes", 3, 2.5],
                ["Lait", 2, 1.1],
                ["Pain", 1, 1.8],
            ]))
            .interactive(true)
            .addable(true)
            .sortable(true))
            .width(520))

        .row(|r| {
            r.item(SortableList::new("prio")
                .label("Priorités (glisser-déposer)")
                .items(&[("p1", "Rapide"), ("p2", "Complet"), ("p3", "Tampon")]));
            r.item(WithLayout::new(Code::new("editor")
                .label("Éditeur Rust")
                .language("rust")
                .value("fn main() {\n    let msg = \"hello grio\";\n    println!(\"{msg}\");\n}\n")
                .interactive(true)
                .lines(true))
                .height(220));
        })

        .row(|r| {
            r.item(WithLayout::new(Gallery::new("shots").label("Galerie (clic = index)").columns(3).interactive(true)).width(340));
            r.item(WithLayout::new(Plot::new("chart")
                .label("Graphique SVG")
                .variant("line")
                .title("Appels en classe")
                .xlabel("séance")
                .ylabel("étudiants"))
                .height(300));
        })

        .panel("Fichiers (serveur)", |p| {
            p.min_width(400);
            p.item(Explorer::new("ex")
                .label("Explorer (racine du projet, *.rs)")
                .root(".")
                .pattern("*.rs"));
            p.item(Code::new("pretty")
                .label("Code généré (lecture seule)")
                .language("rust")
                .value("// ex. la sortie d'un generateur de code\nlet out = compile(source);\n")
                .output()
                .lines(true));
        })

        .row(|r| {
            r.item(WithLayout::new(Button::new("bar").label("Barres").secondary()).scale(1));
            r.item(WithLayout::new(Button::new("line").label("Lignes").secondary()).scale(1));
        })

        .item(Output::new("summary").label("Résumé serveur"))

        .panel("Retouche photo (calques → masque d'inpainting)", |p| {
            p.min_width(500);
            p.item(ImageEditor::new("photo")
                .label("Pinceau, gomme, formes, rognage, rotation, filtres, undo/redo")
                .layers(2)
                .value(""));
            p.item(Markdown::new("photo_note").text(
                "- **Pinceau/Gomme/Formes** dessinent sur le calque actif ; **zoom** (molette) et **Déplacer** (✋) naviguent.\n- **Crop** rogne, **↻ Rotation** pivote, **Filtres** modifie le fond.\n- À chaque geste, le serveur reçoit `{image, layers, mask}` — le **masque** (blanc sur noir) désigne les zones à repeindre (**inpainting**).",
            ));
        })

        .panel("À propos", |p| {
            p.item(Markdown::new("about").text(
                "- **checkbox/dropdown/date/time/dataframe/list/code** envoient leur valeur dans le `change`, récupérée par `ctx.get`.\n- **plot** : SVG dessiné en JS pur, alimenté par `ctx.set` (où `chart(n)` est un petit générateur de sinusoïde).\n- **gallery** : les images uploadées sont des data URLs ; un clic émet l'index en `d`.\n- **explorer** : liste les fichiers de la machine du **serveur** via `/api/explore` (racine bornée + filtre `*.rs`).\n- **imageeditor** : retouche sur canvas (calques RGBA) → **masque** blanc/noir exploitable pour de l'inpainting côté serveur.",
            ));
        })

        .launch("0.0.0.0:7860")
}

/// Séries affichées dans le graphique (`k` décale la phase ; `variant` est
/// `"line"`, `"bar"` ou `"scatter"`).
fn chart(k: f64, variant: &str) -> serde_json::Value {
    use serde_json::json;
    let labels: Vec<String> = (1..=8).map(|i| format!("S{i}")).collect();
    let a: Vec<f64> = (0..8).map(|i| 10.0 + 8.0 * ((i as f64 + k) / 2.0).sin()).collect();
    let b: Vec<f64> = (0..8).map(|i| 6.0 + 5.0 * ((i as f64 + 1.0 + k) / 2.0).cos()).collect();
    json!({
        "variant": variant,
        "labels": labels,
        "series": [
            { "name": "promo A", "data": a, "color": "#6366f1" },
            { "name": "promo B", "data": b, "color": "#f59e0b" },
        ]
    })
}