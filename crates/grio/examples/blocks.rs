use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

use grio::*;

static SECRET_VISIBLE: AtomicBool = AtomicBool::new(true);

/// Lit un champ Texte comme nombre (`a`, `b` sont des chaînes).
fn num(ctx: &Context, id: &str) -> grio::Result<f64> {
    ctx.get_str(id)
        .unwrap_or("")
        .trim()
        .parse()
        .map_err(|_| format!("`{id}` n'est pas un nombre").into())
}

fn main() -> grio::Result<()> {
    App::new("Blocks · grio demo")
        .subtitle("Phase 1 — flux déclarés, chaînage .then/.success/.failure, ctx.event, skip/set_prop, interactive, load, multi-triggers, onglets & accordéon.")
        .run_label("Envoyer")

        // 1.6 — App::on_load : exécuté au montage de la page (connexion WS).
        .on_load(|ctx| {
            ctx.set("load_note", "Page montée — événement `load` reçu par le serveur.");
            Ok(())
        })
        .panel("1.6 · on_load (montage de la page)", |p| {
            p.item(Output::new("load_note").label("État"));
        })

        // 1.1 — Flux déclarés : chaque handler lit SEULEMENT `a`,`b` et n'écrit
        //       QUE `cmp_out` (.flow). Tout accès hors liste serait rejeté/ignoré.
        .row(|r| {
            r.item(Text::new("a").label("a").value("1"));
            r.item(Text::new("b").label("b").value("2"));
        })
        .row(|r| {
            r.item(Button::new("cmp_gt").label("a > b ?"));
            r.item(Button::new("cmp_lt").label("b > a ?"));
            r.item(Output::new("cmp_out").label("Résultat"));
        })
        .on_click("cmp_gt", |ctx| {
            let a: f64 = num(ctx, "a")?;
            let b: f64 = num(ctx, "b")?;
            ctx.set("cmp_out", if a > b { "a > b ✓" } else { "a ≤ b" });
            Ok(())
        })
        .flow(&["a", "b"], &["cmp_out"])
        .on_click("cmp_lt", |ctx| {
            // 1.3 — ctx.event() expose la cible (`c`), l'action (`e`) et les
            //       données (`d`) de l'événement d'origine.
            let target = ctx.event().map(|e| e.c.as_str()).unwrap_or("?");
            let a: f64 = num(ctx, "a")?;
            let b: f64 = num(ctx, "b")?;
            ctx.set("cmp_out", format!("(vu depuis `{target}`) {}", if b > a { "b > a ✓" } else { "b ≤ a" }));
            Ok(())
        })
        .flow(&["a", "b"], &["cmp_out"])

        // 1.7 — Multi-déclencheurs : la même fonction sur plusieurs boutons.
        .row(|r| {
            r.item(Button::new("opt_a").label("Option A"));
            r.item(Button::new("opt_b").label("Option B"));
            r.item(Output::new("multi_out").label("Dernier clic"));
        })
        .on("click", ["opt_a", "opt_b"], |ctx| {
            let t = ctx.event().map(|e| e.c.as_str()).unwrap_or("?");
            ctx.set("multi_out", format!("Dernier clic : `{t}`"));
            Ok(())
        })

        // 1.5 — Interactivité explicite : champs grisés non éditables.
        .row(|r| {
            r.item(Text::new("ro").label("Lecture seule").value("9").interactive(false));
            r.item(Output::new("ro_note").label("Note").value("non éditable (interactive = false)"));
            r.item(Slider::new("ro_slider").label("Curseur figé").min(0.0).max(10.0).step(1.0).value(5.0).interactive(false));
        })

        // 1.4 — skip + set_prop : ignorer une sortie, masquer/afficher une autre.
        .row(|r| {
            r.item(Button::new("skip_btn").label("set + skip ⇒ out_b figé"));
            r.item(Output::new("out_a").label("out_a"));
            r.item(Output::new("out_b").label("out_b").value("valeur initiale"));
        })
        .on_click("skip_btn", |ctx| {
            ctx.skip("out_b");
            ctx.set("out_a", "mise à jour A ✓");
            ctx.set("out_b", "NE doit PAS s'afficher");
            ctx.alert(AlertLevel::Info, "out_b est skip → il garde sa valeur");
            Ok(())
        })
        .row(|r| {
            r.item(Button::new("toggle_btn").label("Masquer le contenu"));
            r.item(Output::new("secret").label("Contenu").value("*** CONTENU CONFIDENTIEL ***"));
        })
        .on_click("toggle_btn", |ctx| {
            let prev = SECRET_VISIBLE.fetch_xor(true, Ordering::SeqCst);
            ctx.set_prop("secret", "visible", !prev);
            ctx.set_prop(
                "toggle_btn",
                "label",
                if !prev { "Afficher le contenu" } else { "Masquer le contenu" },
            );
            ctx.alert(AlertLevel::Success, if prev { "contenu masqué" } else { "contenu affiché" });
            Ok(())
        })

        // 1.2 — Chaînage de fins : .success / .failure sur un handler fautif.
        .add(Output::new("err_out").label("err_out"))
        .add(Output::new("err_log").label("err_log"))
        .add(Button::new("err_btn").label("handler : erreur si `a` vide"))
        .on_click("err_btn", |ctx| {
            let a = ctx.get_str("a").unwrap_or("").trim();
            if a.is_empty() {
                return Err("champ `a` vide".into());
            }
            ctx.set("err_out", format!("ok, a = {a}"));
            Ok(())
        })
        .success(|ctx| {
            ctx.set("err_log", "handler réussi → .success exécuté");
            Ok(())
        })
        .failure(|ctx| {
            ctx.alert(AlertLevel::Warn, "Échec attrapé par .failure (récupération)");
            ctx.set("err_log", "échec → .failure exécuté et géré");
            Ok(())
        })

        // 1.8 — Conteneurs : onglets + accordéon.
        .item(
            Tabs::new("tabs")
                .tab("Compare & co", |t| {
                    t.item(Markdown::new("txt_1").text("### Onglet 1\n\nLes **flux déclarés** (1.1), `ctx.event` (1.3) et les **multi-déclencheurs** (1.7) vivent dans les cartes ci-dessus."));
                })
                .tab("Chatbot (1.2)", |t| {
                    t.item(Text::new("chat_in").label("Message").placeholder("tapez puis Envoyer…"));
                    t.item(Output::new("chat_out").label("Conversation"));
                    t.item(Markdown::new("txt_2").text("#### Chaînage\n`on_submit(user).then(bot)` : le bot répond **après** l'utilisateur, mot à mot (streaming)."));
                }),
        )
        .item(
            Accordion::new("acc")
                .open(true)
                .section("Onglet chatbot", |s| {
                    s.item(Markdown::new("acc_1").text("Réponses **en streaming** via `ctx.append`, progress et alertes — voir `examples/greet.rs`."));
                })
                .section("Props dynamiques (1.4)", |s| {
                    s.item(Markdown::new("acc_2").text("`ctx.set_prop(\"id\", \"visible\"|\"label\"|…)` met à jour la **configuration** d'un composant sans changer sa valeur."));
                }),
        )

        // 1.2 — Chaînage : le chatbot (user_fn.then(bot_fn)).
        .on_submit(|ctx| {
            let msg: String = ctx.get("chat_in")?;
            if msg.trim().is_empty() {
                return Err("message vide".into());
            }
            ctx.append("chat_out", format!("Vous : {msg}\n"));
            Ok(())
        })
        .then(|ctx| {
            ctx.append("chat_out", "Bot : ");
            for _i in 1..=6 {
                if ctx.cancelled() {
                    ctx.append("chat_out", "\n[interrompu]\n");
                    return Ok(());
                }
                ctx.append("chat_out", "*");
                thread::sleep(Duration::from_millis(120));
            }
            ctx.append("chat_out", " (réponse simulée)\n");
            ctx.alert(AlertLevel::Success, "chatbot terminé");
            Ok(())
        })

        .launch("127.0.0.1:7860")?;
    Ok(())
}