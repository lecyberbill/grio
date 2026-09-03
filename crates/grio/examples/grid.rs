use grio::*;

fn main() -> grio::Result<()> {
    App::new("Grid & Conteneurs · grio demo")
        .subtitle("Démonstration du composant Grid, de l'imbrication de conteneurs (Row, Column, Grid) et des alignements.")
        .panel("1. Grille Responsive à 3 Colonnes (App::grid)", |p| {
            p.grid(3, |g| {
                g.item(Text::new("c1").label("Colonne 1").value("Texte A"));
                g.item(Text::new("c2").label("Colonne 2").value("Texte B"));
                g.item(Text::new("c3").label("Colonne 3").value("Texte C"));
                g.item(Slider::new("s1").label("Slider A").min(0.0).max(100.0).value(25.0));
                g.item(Slider::new("s2").label("Slider B").min(0.0).max(100.0).value(50.0));
                g.item(Slider::new("s3").label("Slider C").min(0.0).max(100.0).value(75.0));
            });
        })

        .panel("2. Grille à 2 Colonnes avec Espacements Personnalisés", |p| {
            p.grid(2, |g| {
                g.gap(24.0);
                g.item(Output::new("out_left").label("Panneau Gauche").value("Zone 1"));
                g.item(Output::new("out_right").label("Panneau Droit").value("Zone 2"));
            });
        })

        .panel("3. Imbrication : Colonnes dans une Ligne & Sous-grille", |p| {
            p.row(|r| {
                // Sous-colonne 1
                r.column(|col| {
                    col.scale(1);
                    col.item(Markdown::new("col1_desc").value("### Sous-colonne Gauche\nOrganisée verticalement."));
                    col.item(Text::new("user_input").label("Votre message").value("Bonjour !"));
                    col.item(Button::new("send_btn").label("Calculer"));
                });

                // Sous-colonne 2 contenant une sous-grille 2x2
                r.column(|col| {
                    col.scale(2);
                    col.item(Markdown::new("col2_desc").value("### Sous-colonne Droite (Grille 2×2 imbriquée)"));
                    col.grid(2, |subgrid| {
                        subgrid.item(Output::new("res_len").label("Longueur").value("0"));
                        subgrid.item(Output::new("res_upper").label("Majuscules").value("-"));
                        subgrid.item(Output::new("res_words").label("Mots").value("0"));
                        subgrid.item(Output::new("res_echo").label("Echo").value("-"));
                    });
                });
            });
        })

        .on_click("send_btn", |ctx| {
            let msg: String = ctx.get("user_input").unwrap_or_default();
            let words = msg.split_whitespace().count();
            ctx.set("res_len", msg.chars().count().to_string());
            ctx.set("res_upper", msg.to_uppercase());
            ctx.set("res_words", words.to_string());
            ctx.set("res_echo", format!("Reçu: {}", msg));
            Ok(())
        })

        .launch("127.0.0.1:7860")
}
