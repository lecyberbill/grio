use std::thread;
use std::time::Duration;

use grio::*;

fn main() -> grio::Result<()> {
    App::new("Greet · grio demo")
        .subtitle("Un équivalent minimal à Gradio — serveur, composants, événements et temps réel en Rust.")
        .add(
            Markdown::new("intro").text(
                "# Bienvenue\n\nCeci est un rendu **Markdown** servi par le moteur.\nSaisissez un nom, choisissez une intensité, puis cliquez sur *Run*.",
            ),
        )
        .row(|r| {
            r.item(Text::new("name").label("Name").value("World").placeholder("Votre nom…"));
            r.item(
                Slider::new("intensity")
                    .label("Intensity")
                    .min(0.0)
                    .max(5.0)
                    .step(1.0)
                    .value(2.0),
            );
        })
        .item(Output::new("greeting").label("Greeting"))
        .panel("Temps réel — streaming, progress, alertes", |p| {
            p.item(
                Markdown::new("rt").text(
                    "Cliquez sur **Lancer** : le handler tourne en arrière-plan (file d'attente + pool de threads), les fragments sont **poussés en continu** et la barre progresse. Re-cliquez pour **annuler**.",
                ),
            );
            p.item(Progress::new("pg").label("Génération"));
            p.item(Output::new("log").label("Sortie en streaming"));
            p.item(Button::new("generate").label("Lancer la génération"));
        })
        .on_event("reset", |_ctx| {
            println!("[event] reset ← serveur");
            Ok(())
        })
        .on_submit(|ctx| {
            let name: String = ctx.get("name")?;
            let intensity: f64 = ctx.get("intensity")?;
            let greeting = format!("Hello, {} {}!", name, "!".repeat(intensity as usize));
            ctx.set("greeting", greeting);
            Ok(())
        })
        .on_click("generate", |ctx| {
            // Tâche longue simulée : le handler reste synchrone, le moteur
            // l'exécute sur un thread de travail et pousse les mises à jour.
            ctx.set("log", "démarrage…\n");
            for i in 1..=10 {
                if ctx.cancelled() {
                    ctx.alert(AlertLevel::Warn, "Génération annulée");
                    return Ok(());
                }
                ctx.progress("pg", i as f64 / 10.0, format!("étape {i}/10"));
                ctx.append("log", format!("token {i}\n"));
                thread::sleep(Duration::from_millis(350));
            }
            ctx.alert(AlertLevel::Success, "Génération terminée");
            Ok(())
        })
        .launch("127.0.0.1:7860")?;
    Ok(())
}
