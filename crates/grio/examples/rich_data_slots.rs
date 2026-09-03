//! Exemple Phase 9 / Lot 2 : RichText, DataEditor & Dynamic Slots
//!
//! Cet exemple illustre :
//! 1. `RichText` : Saisie d'incidents avec barre d'outils markdown (Gras, Italique, Titres, Code, Liens).
//! 2. `DataEditor` : Grille interactive typée avec cases à cocher, listes déroulantes et copier-coller TSV/CSV.
//! 3. `DynamicContainer` : Injection réactive de composants à chaud via `ctx.append_component` et `ctx.replace_children`.
//! 4. `ctx.set_visible` : Masquage et affichage dynamique de sections.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;
use grio::*;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<()> {
    let mut app = App::new("Support IT & Gestion des Incidents");

    // Ligne 1 : Micro-éditeur RichText & Contrôles
    app = app.item(
        Row::new("r_editor")
            .item(
                RichText::new("ticket_desc")
                    .label("Description détaillée de l'incident (Markdown)")
                    .placeholder("Décrivez les étapes pour reproduire l'incident...")
                    .value("### Problème de connexion VPN\n\n- **Utilisateur :** Jean Dupont\n- **Message d'erreur :** `TLS handshake failed`\n- **Impact :** Bloquant pour le télétravail.")
                    .lines(7),
            )
            .item(
                Panel::new("p_actions")
                    .label("Actions & Contrôles")
                    .item(Text::new("requester").label("Demandeur").value("jean.dupont@entreprise.fr"))
                    .item(Dropdown::new("category").label("Catégorie").options(&["Réseau & VPN", "Compte & Accès", "Matériel", "Logiciel"]))
                    .item(Button::new("btn_toggle_panel").label("👁 Masquer/Afficher Catalogue"))
                    .item(Button::new("btn_add_slot").label("➕ Injecter Composant Dynamique"))
                    .item(Button::new("btn_clear_slot").label("🗑 Vider le Slot").variant("secondary")),
            ),
    );

    // Ligne 2 : Grille DataEditor (Catalogue de services & SLA)
    let initial_data = vec![
        vec![json!("SRV-01"), json!("Réinitialisation MDP"), json!(true), json!(1), json!("P1 - Critique")],
        vec![json!("SRV-02"), json!("Accès VPN Distant"), json!(true), json!(4), json!("P2 - Haute")],
        vec![json!("SRV-03"), json!("Demande de badge"), json!(false), json!(24), json!("P3 - Normale")],
        vec![json!("SRV-04"), json!("Poste de travail neuf"), json!(true), json!(48), json!("P4 - Basse")],
    ];

    app = app.item(
        Row::new("r_catalog")
            .item(
                DataEditor::new("services_grid")
                    .label("Catalogue des Services & Règles SLA (Double-clic pour éditer, Cases à cocher actives, Ctrl+V supporté)")
                    .column("id", "Réf.", ColumnType::Text)
                    .column("name", "Service IT", ColumnType::Text)
                    .column("active", "Actif", ColumnType::Boolean)
                    .column("sla", "SLA Max (h)", ColumnType::Number)
                    .column("priority", "Priorité", ColumnType::Dropdown(vec![
                        "P1 - Critique".into(),
                        "P2 - Haute".into(),
                        "P3 -扩大".into(),
                        "P4 - Basse".into(),
                    ]))
                    .data(initial_data)
                    .allow_add(true)
                    .allow_delete(true)
                    .allow_paste(true)
                    .max_height(280),
            ),
    );

    // Ligne 3 : DynamicContainer (Zone d'injection à chaud)
    app = app.item(
        Row::new("r_slots")
            .item(
                Panel::new("p_slot_zone")
                    .label("Zone de Conteneur Dynamique (Slots injectés à l'exécution)")
                    .item(
                        DynamicContainer::new("dynamic_slot")
                            .item(Output::new("slot_initial").label("État du slot").value("Slot vide en attente d'injection...")),
                    ),
            ),
    );

    // Gestionnaire : bascule de visibilité
    let catalog_visible = Arc::new(AtomicBool::new(true));
    let cat_vis_clone = catalog_visible.clone();
    app = app.on_click("btn_toggle_panel", move |ctx| {
        let prev = cat_vis_clone.fetch_xor(true, Ordering::SeqCst);
        let now = !prev;
        ctx.set_visible("r_catalog", now);
        if now {
            ctx.alert(AlertLevel::Info, "Catalogue de services affiché.");
        } else {
            ctx.alert(AlertLevel::Warn, "Catalogue de services masqué.");
        }
        Ok(())
    });

    // Gestionnaire : injection dynamique de composants à chaud
    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();
    app = app.on_click("btn_add_slot", move |ctx| {
        let c = counter_clone.fetch_add(1, Ordering::SeqCst) + 1;
        let new_comp = Output::new(format!("dynamic_item_{c}"))
            .label(format!("Composant injecté à chaud #{c}"))
            .value(format!("✅ Ticket #{c} validé et inspecté dynamiquement"));
        ctx.append_component("dynamic_slot", new_comp);
        ctx.alert(AlertLevel::Success, format!("Composant #{c} injecté dans le slot !"));
        Ok(())
    });

    // Gestionnaire : vidage du slot
    app = app.on_click("btn_clear_slot", |ctx| {
        ctx.clear_container("dynamic_slot");
        ctx.alert(AlertLevel::Info, "Slot dynamique vidé.");
        Ok(())
    });

    // Gestionnaire de soumission
    app = app.on_submit(|ctx| {
        let desc: String = ctx.get("ticket_desc").unwrap_or_default();
        let requester = ctx.get_str("requester").unwrap_or("");
        let cat = ctx.get_str("category").unwrap_or("");

        ctx.alert(
            AlertLevel::Success,
            format!("Ticket créé avec succès pour {requester} [{cat}] !\nLongueur description : {} caractères", desc.len()),
        );
        Ok(())
    });

    println!("🚀 Démarrage de l'exemple RichText, DataEditor & Dynamic Slots sur http://localhost:7865 ...");
    app.serve("127.0.0.1:7865").await
}
