//! Déclaration de l'application (builder `App`) et distribution des
//! événements vers les handlers.

use std::sync::Arc;

use serde_json::{json, Value};

use crate::components::{
    Button, Column, Component, Grid, IntoBox, Layout, Panel, Row, Tabs, WithLayout,
};
use crate::context::Context;
use crate::events::{EventName, WireEvent};
use crate::{Error, Result};

/// Mode d'affichage du thème (sombre, clair ou détection système).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ThemeMode {
    /// Mode sombre forcé.
    Dark,
    /// Mode clair forcé.
    Light,
    /// Détection automatique selon les préférences de l'OS/navigateur.
    #[default]
    System,
}

/// Personnalisation de l'apparence visuelle (Thème).
#[derive(Clone, Debug)]
pub struct Theme {
    /// Mode de base (Dark, Light, System).
    pub mode: ThemeMode,
    /// Couleur d'accent principale (hex ou rgb, ex: "#6366f1").
    pub primary: Option<String>,
    /// Rayon d'arrondi des bordures (ex: "8px", "12px").
    pub radius: Option<String>,
    /// Police de caractères principale.
    pub font: Option<String>,
    /// Affiche ou masque le bouton de bascule Dark/Light dans l'en-tête.
    pub toggle: bool,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            mode: ThemeMode::System,
            primary: None,
            radius: None,
            font: None,
            toggle: true,
        }
    }
}

impl Theme {
    /// Thème sombre moderne par défaut.
    pub fn dark() -> Self {
        Self {
            mode: ThemeMode::Dark,
            ..Default::default()
        }
    }
    /// Thème clair moderne par défaut.
    pub fn light() -> Self {
        Self {
            mode: ThemeMode::Light,
            ..Default::default()
        }
    }
    /// Thème adaptatif calqué sur l'OS/navigateur.
    pub fn system() -> Self {
        Self {
            mode: ThemeMode::System,
            ..Default::default()
        }
    }

    /// **Preset Tokyo Night** : Thème sombre néon bleu/violet moderne.
    pub fn tokyo_night() -> Self {
        Self {
            mode: ThemeMode::Dark,
            primary: Some("#7aa2f7".into()),
            radius: Some("10px".into()),
            font: Some("Inter, system-ui, sans-serif".into()),
            toggle: true,
        }
    }

    /// **Preset Nord** : Thème sombre aux nuances polaires et reposantes.
    pub fn nord() -> Self {
        Self {
            mode: ThemeMode::Dark,
            primary: Some("#88c0d0".into()),
            radius: Some("8px".into()),
            font: Some("Inter, system-ui, sans-serif".into()),
            toggle: true,
        }
    }

    /// **Preset Cyberpunk** : Thème sombre à fort contraste avec accents rose/néon.
    pub fn cyberpunk() -> Self {
        Self {
            mode: ThemeMode::Dark,
            primary: Some("#f43f5e".into()),
            radius: Some("4px".into()),
            font: Some("JetBrains Mono, Fira Code, monospace".into()),
            toggle: true,
        }
    }

    /// **Preset Catppuccin Mocha** : Thème pastel sombre avec accent lavande.
    pub fn catppuccin_mocha() -> Self {
        Self {
            mode: ThemeMode::Dark,
            primary: Some("#cba6f7".into()),
            radius: Some("12px".into()),
            font: Some("Inter, system-ui, sans-serif".into()),
            toggle: true,
        }
    }

    /// **Preset Corporate** : Thème clair épuré pour applications d'entreprise.
    pub fn corporate() -> Self {
        Self {
            mode: ThemeMode::Light,
            primary: Some("#2563eb".into()),
            radius: Some("6px".into()),
            font: Some("Roboto, Inter, system-ui, sans-serif".into()),
            toggle: true,
        }
    }

    /// Définit la couleur d'accentuation (ex: "#6366f1", "#10b981").
    pub fn primary(mut self, p: impl Into<String>) -> Self {
        self.primary = Some(p.into());
        self
    }
    /// Définit le rayon d'arrondi (ex: "6px", "10px", "16px").
    pub fn radius(mut self, r: impl Into<String>) -> Self {
        self.radius = Some(r.into());
        self
    }
    /// Définit la famille de police (ex: "Inter, sans-serif").
    pub fn font(mut self, f: impl Into<String>) -> Self {
        self.font = Some(f.into());
        self
    }
    /// Active ou désactive le bouton toggle Dark/Light dans l'UI.
    pub fn toggle(mut self, enabled: bool) -> Self {
        self.toggle = enabled;
        self
    }
}

/// Signature d'un handler : reçoit `&mut Context`, renvoie une erreur
/// éventuelle (affichée comme toast côté client). Peut rester synchrone
/// (ex. avec `sleep`) — le moteur l'exécute sur un pool de threads.
///
/// Stocké dans un `Arc` pour être partageable entre plusieurs déclencheurs
/// (`.on` lie la même fonction à plusieurs identifiants).
pub type HandlerFn = Arc<dyn Fn(&mut Context) -> Result<()> + Send + Sync>;

/// Condition d'exécution d'un maillon de chaîne (`.then` / `.success` /
/// `.failure`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RunCond {
    /// Toujours exécuté (après le handler principal).
    Always,
    /// Exécuté seulement si le handler précédent a réussi.
    Success,
    /// Exécuté seulement si le handler précédent a échoué (peut redresser).
    Failure,
}

/// Maillon d'une chaîne d'événements attachée à un handler principal.
pub struct Sibling {
    /// Condition d'exécution.
    pub on: RunCond,
    /// Fonction à exécuter.
    pub f: HandlerFn,
}

/// Enregistrement d'un listener : événement ciblé + fonction + options.
pub struct HandlerDef {
    /// Événement écouté.
    pub event: EventName,
    /// Composant ciblé (`None` = global, ex. submit/load).
    pub component: Option<String>,
    /// Fonction de traitement.
    pub f: HandlerFn,
    /// **Flux déclaré** : entrées autorisées en lecture (scoping `ctx.get`).
    pub inputs: Option<Vec<String>>,
    /// **Flux déclaré** : sorties autorisées en écriture (scoping `ctx.set`,
    /// `set_prop`, `append`, `progress`).
    pub outputs: Option<Vec<String>>,
    /// Chaîne d'exécution post-handler (`.then`, `.success`, `.failure`).
    pub chain: Vec<Sibling>,
}

/// Définition d'une page dans une application multi-pages.
#[derive(Clone, Debug)]
pub struct PageDef {
    /// Route d'accès (ex: `"/"`, `"/chat"`, `"/dashboard"`).
    pub route: String,
    /// Titre affiché dans la barre de navigation.
    pub title: String,
    /// Icône de navigation optionnelle (emoji ou texte).
    pub icon: Option<String>,
    /// Identifiant du conteneur de la page.
    pub id: String,
}

/// Application web déclarative.
///
/// Se construit par chaînage de méthodes ; `launch` démarre le serveur HTTP
/// (**UI + API REST**).
pub struct App {
    /// Titre affiché en tête de page.
    pub title: String,
    /// Sous-titre affiché sous le titre.
    pub subtitle: String,
    /// Racine de l'arbre de composants (colonne).
    pub root: Column,
    /// Pages enregistrées (si mode multi-pages).
    pub pages: Vec<PageDef>,
    /// Listeners enregistrés (soumission, changement, clic, applicatif).
    pub handlers: Vec<HandlerDef>,
    /// Mode live : chaque `change` redéclenche aussi les `submit`.
    pub live: bool,
    /// Libellé du bouton Run généré par `on_submit`.
    pub run_label: String,
    /// Identifiant du bouton Run (si `on_submit` a été appelé).
    pub run_button: Option<String>,
    /// Logs d'activité de la console (HTTP, WebSocket, API). Activé par défaut.
    pub verbose: bool,
    /// Clé d'API obligatoire pour `/api/predict` (si définie).
    pub api_key: Option<String>,
    /// Autorise les requêtes CORS cross-origin.
    pub allow_cors: bool,
    /// Active la documentation interactive `/docs` et OpenAPI `/api/openapi.json`.
    pub enable_docs: bool,
    /// Isole les sessions utilisateur par client. Activé par défaut.
    pub isolated_sessions: bool,
    /// Thème visuel de l'application.
    pub theme: Theme,
    /// Largeur maximale du conteneur de l'application (ex: 1200px).
    pub max_width: Option<u32>,
    /// Outils enregistrés pour le protocole Model Context Protocol (MCP `/mcp/v1`).
    pub mcp_tools: Vec<crate::mcp::McpTool>,
    /// Active le serveur Model Context Protocol (MCP) pour Claude Desktop / Cursor.
    pub enable_mcp: bool,
    /// Registre de plugins WebAssembly (Phase 15.3).
    pub wasm_registry: std::sync::Arc<crate::wasm::WasmRegistry>,
}

/// Collecteur d'éléments pour une page dans une application multi-pages.
pub struct PageBuilder {
    pub(crate) id: String,
    pub(crate) icon: Option<String>,
    pub(crate) children: Vec<Box<dyn Component>>,
    pub(crate) gap: f64,
}

impl PageBuilder {
    /// Crée un constructeur de page avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            icon: None,
            children: Vec::new(),
            gap: 16.0,
        }
    }

    /// Définit l'icône de la page (emoji ou texte court).
    pub fn icon(&mut self, icon: impl Into<String>) -> &mut Self {
        self.icon = Some(icon.into());
        self
    }

    /// Définit l'espacement entre les composants en pixels.
    pub fn gap(&mut self, g: f64) -> &mut Self {
        self.gap = g;
        self
    }

    /// Ajoute un composant à la page.
    pub fn item(&mut self, c: impl IntoBox) -> &mut Self {
        self.children.push(c.into_box());
        self
    }

    /// Ajoute une ligne (Row) à la page.
    pub fn row(&mut self, task: impl FnOnce(&mut RowBuilder)) -> &mut Self {
        let mut b = RowBuilder::default();
        task(&mut b);
        let mut r = Row::new(format!("{}-row-{}", self.id, self.children.len())).gap(b.gap);
        for c in b.children {
            r.children.push(c);
        }
        if b.layout == Layout::default() {
            self.children.push(Box::new(r));
        } else {
            self.children
                .push(Box::new(WithLayout::new(r).set_layout(b.layout)));
        }
        self
    }

    /// Ajoute une colonne (Column) à la page.
    pub fn column(&mut self, task: impl FnOnce(&mut RowBuilder)) -> &mut Self {
        let mut b = RowBuilder::default();
        task(&mut b);
        let mut col = Column::new(format!("{}-col-{}", self.id, self.children.len())).gap(b.gap);
        for c in b.children {
            col.children.push(c);
        }
        if b.layout == Layout::default() {
            self.children.push(Box::new(col));
        } else {
            self.children
                .push(Box::new(WithLayout::new(col).set_layout(b.layout)));
        }
        self
    }

    /// Ajoute une grille (Grid) à la page.
    pub fn grid(&mut self, columns: usize, task: impl FnOnce(&mut RowBuilder)) -> &mut Self {
        let mut b = RowBuilder::default();
        task(&mut b);
        let mut g = Grid::new(format!("{}-grid-{}", self.id, self.children.len()))
            .columns(columns)
            .gap(b.gap);
        for c in b.children {
            g.children.push(c);
        }
        if b.layout == Layout::default() {
            self.children.push(Box::new(g));
        } else {
            self.children
                .push(Box::new(WithLayout::new(g).set_layout(b.layout)));
        }
        self
    }

    /// Ajoute un panneau (Panel) à la page.
    pub fn panel(
        &mut self,
        label: impl Into<String>,
        task: impl FnOnce(&mut RowBuilder),
    ) -> &mut Self {
        let label = label.into();
        let mut b = RowBuilder::default();
        task(&mut b);
        let mut p = Panel::new(format!("{}-panel-{}", self.id, label))
            .label(label)
            .gap(b.gap);
        for c in b.children {
            p.children.push(c);
        }
        if b.layout == Layout::default() {
            self.children.push(Box::new(p));
        } else {
            self.children
                .push(Box::new(WithLayout::new(p).set_layout(b.layout)));
        }
        self
    }

    /// Ajoute un conteneur d'onglets (Tabs) à la page.
    pub fn tabs(&mut self, task: impl FnOnce(Tabs) -> Tabs) -> &mut Self {
        let t = task(Tabs::new(format!(
            "{}-tabs-{}",
            self.id,
            self.children.len()
        )));
        self.children.push(Box::new(t));
        self
    }
}

/// Collecteur d'éléments pour `App::row` / `column` / `panel` / `grid`.
pub struct RowBuilder {
    pub(crate) children: Vec<Box<dyn Component>>,
    pub(crate) gap: f64,
    pub(crate) layout: Layout,
}

impl Default for RowBuilder {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            gap: 16.0,
            layout: Layout::default(),
        }
    }
}

impl RowBuilder {
    /// Ajoute un composant au groupe en construction.
    pub fn item(&mut self, c: impl IntoBox) {
        self.children.push(c.into_box());
    }
    /// Ajoute un conteneur d'onglets au groupe.
    pub fn tabs(&mut self, task: impl FnOnce(Tabs) -> Tabs) {
        let t = task(Tabs::new(format!("tabs-{}", self.children.len())));
        self.children.push(Box::new(t));
    }
    /// Ajoute une sous-ligne au groupe.
    pub fn row(&mut self, task: impl FnOnce(&mut RowBuilder)) {
        let mut b = RowBuilder::default();
        task(&mut b);
        let mut r = Row::new(format!("row-{}", self.children.len())).gap(b.gap);
        for c in b.children {
            r.children.push(c);
        }
        if b.layout == Layout::default() {
            self.children.push(Box::new(r));
        } else {
            self.children
                .push(Box::new(WithLayout::new(r).set_layout(b.layout)));
        }
    }
    /// Ajoute une sous-colonne au groupe.
    pub fn column(&mut self, task: impl FnOnce(&mut RowBuilder)) {
        let mut b = RowBuilder::default();
        task(&mut b);
        let mut col = Column::new(format!("col-{}", self.children.len())).gap(b.gap);
        for c in b.children {
            col.children.push(c);
        }
        if b.layout == Layout::default() {
            self.children.push(Box::new(col));
        } else {
            self.children
                .push(Box::new(WithLayout::new(col).set_layout(b.layout)));
        }
    }
    /// Ajoute une sous-grille au groupe.
    pub fn grid(&mut self, columns: usize, task: impl FnOnce(&mut RowBuilder)) {
        let mut b = RowBuilder::default();
        task(&mut b);
        let mut g = Grid::new(format!("grid-{}", self.children.len()))
            .columns(columns)
            .gap(b.gap);
        for c in b.children {
            g.children.push(c);
        }
        if b.layout == Layout::default() {
            self.children.push(Box::new(g));
        } else {
            self.children
                .push(Box::new(WithLayout::new(g).set_layout(b.layout)));
        }
    }
    /// Ajoute un panneau au groupe.
    pub fn panel(&mut self, label: impl Into<String>, task: impl FnOnce(&mut RowBuilder)) {
        let label = label.into();
        let mut b = RowBuilder::default();
        task(&mut b);
        let mut p = Panel::new(format!("panel-{}", label))
            .label(label)
            .gap(b.gap);
        for c in b.children {
            p.children.push(c);
        }
        if b.layout == Layout::default() {
            self.children.push(Box::new(p));
        } else {
            self.children
                .push(Box::new(WithLayout::new(p).set_layout(b.layout)));
        }
    }
    /// Espace entre les composants du groupe, en pixels.
    pub fn gap(&mut self, g: f64) {
        self.gap = g;
    }
    /// Largeur du groupe, en pixels.
    pub fn width(&mut self, w: u32) {
        self.layout.width = Some(w);
    }
    /// Hauteur du groupe, en pixels.
    pub fn height(&mut self, h: u32) {
        self.layout.height = Some(h);
    }
    /// Largeur maximale du groupe, en pixels.
    pub fn max_width(&mut self, mw: u32) {
        self.layout.max_width = Some(mw);
    }
    /// Hauteur maximale du groupe, en pixels.
    pub fn max_height(&mut self, mh: u32) {
        self.layout.max_height = Some(mh);
    }
    /// Proportion du groupe dans la colonne/racine (comme `scale` de Gradio).
    pub fn scale(&mut self, s: u32) {
        self.layout.scale = Some(s);
    }
    /// Largeur minimale du groupe, en pixels.
    pub fn min_width(&mut self, w: u32) {
        self.layout.min_width = Some(w);
    }
}

impl App {
    /// Crée une application avec un titre de page.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: String::new(),
            root: Column::new("root").gap(16.0),
            pages: Vec::new(),
            handlers: Vec::new(),
            live: false,
            run_label: "Run".to_string(),
            run_button: None,
            verbose: true,
            api_key: None,
            allow_cors: true,
            enable_docs: true,
            isolated_sessions: true,
            theme: Theme::default(),
            max_width: None,
            mcp_tools: Vec::new(),
            enable_mcp: true,
            wasm_registry: std::sync::Arc::new(crate::wasm::WasmRegistry::new()),
        }
    }

    /// **Enregistre un greffon WebAssembly** : associe une instance de plugin WASM
    /// sandboxé à un identifiant unique utilisable dans les handlers via `ctx.call_wasm`.
    ///
    /// ```rust
    /// # use grio::*;
    /// # fn example() {
    /// let plugin = WasmPlugin::new("moderator");
    /// let app = App::new("Demo").wasm_plugin("mod", plugin);
    /// # }
    /// ```
    pub fn wasm_plugin(mut self, id: impl Into<String>, plugin: crate::wasm::WasmPlugin) -> Self {
        if let Some(reg) = std::sync::Arc::get_mut(&mut self.wasm_registry) {
            reg.register(id, plugin);
        } else {
            let mut reg = (*self.wasm_registry).clone();
            reg.register(id, plugin);
            self.wasm_registry = std::sync::Arc::new(reg);
        }
        self
    }

    /// Déclare une page dans une application multi-pages.
    pub fn page(
        mut self,
        route: impl Into<String>,
        title: impl Into<String>,
        task: impl FnOnce(&mut PageBuilder),
    ) -> Self {
        let route = route.into();
        let title = title.into();
        let mut b = PageBuilder::new(format!("page-{}", self.pages.len()));
        task(&mut b);

        let mut page_col = Column::new(&b.id).gap(b.gap);
        for c in b.children {
            page_col.push(c);
        }

        self.pages.push(PageDef {
            route,
            title,
            icon: b.icon,
            id: b.id,
        });
        self.root.push(Box::new(page_col));
        self
    }

    /// Déclare une page avec une icône explicite.
    pub fn page_with_icon(
        self,
        route: impl Into<String>,
        title: impl Into<String>,
        icon: impl Into<String>,
        task: impl FnOnce(&mut PageBuilder),
    ) -> Self {
        let icon_str = icon.into();
        self.page(route, title, |p| {
            p.icon(icon_str);
            task(p);
        })
    }

    /// Définit la largeur maximale de l'application (ex: 1200 pour un container centré à 1200px).
    pub fn max_width(mut self, mw: u32) -> Self {
        self.max_width = Some(mw);
        self
    }

    /// Personnalise le thème visuel (Dark, Light, couleurs, arrondis, police).
    pub fn theme(mut self, theme: Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Ajoute un conteneur d'onglets à l'application.
    pub fn tabs(mut self, task: impl FnOnce(Tabs) -> Tabs) -> Self {
        let t = task(Tabs::new(format!("tabs-{}", self.root.children().len())));
        self.root.push(Box::new(t));
        self
    }

    /// Enregistre un outil pour le protocole Model Context Protocol (MCP `/mcp/v1`).
    ///
    /// Permet à Claude Desktop, Cursor et Windsurf d'appeler directement cet outil.
    ///
    /// ```rust
    /// # use grio::*;
    /// # fn example() -> App {
    /// App::new("AI Hub")
    ///     .mcp_tool(
    ///         "query_db",
    ///         "Execute analytical SQL query on data warehouse",
    ///         serde_json::json!({
    ///             "type": "object",
    ///             "properties": {
    ///                 "sql": { "type": "string", "description": "SQL query to execute" }
    ///             },
    ///             "required": ["sql"]
    ///         }),
    ///         |args| {
    ///             let sql = args.get("sql").and_then(|v| v.as_str()).unwrap_or("");
    ///             Ok(serde_json::json!({ "status": "ok", "rows": 42, "executed": sql }))
    ///         }
    ///     )
    /// # }
    /// ```
    pub fn mcp_tool(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        input_schema: Value,
        handler: impl Fn(Value) -> Result<Value> + Send + Sync + 'static,
    ) -> Self {
        self.mcp_tools.push(crate::mcp::McpTool::new(
            name,
            description,
            input_schema,
            handler,
        ));
        self
    }

    /// Active ou désactive le serveur Model Context Protocol (`/mcp/v1`).
    pub fn mcp(mut self, enable: bool) -> Self {
        self.enable_mcp = enable;
        self
    }

    /// Définit une clé d'API obligatoire pour accéder à `POST /api/predict`.
    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    /// Active ou désactive les requêtes CORS cross-origin.
    pub fn cors(mut self, allow: bool) -> Self {
        self.allow_cors = allow;
        self
    }

    /// Active ou désactive la documentation Swagger / OpenAPI (`/docs` et `/api/openapi.json`).
    pub fn docs(mut self, enable: bool) -> Self {
        self.enable_docs = enable;
        self
    }

    /// Isole les valeurs d'état par session client (activé par défaut).
    pub fn isolate_sessions(mut self, isolate: bool) -> Self {
        self.isolated_sessions = isolate;
        self
    }

    /// Définit le sous-titre affiché sous le titre.
    pub fn subtitle(mut self, s: impl Into<String>) -> Self {
        self.subtitle = s.into();
        self
    }

    /// Désactive les logs d'activité de la console (la bannière de démarrage,
    /// elle, reste affichée).
    pub fn quiet(mut self) -> Self {
        self.verbose = false;
        self
    }

    /// En mode live, chaque `change` sur une entrée redéclenche aussi les handlers `submit`.
    pub fn live(mut self) -> Self {
        self.live = true;
        self
    }

    /// Personnalise le libellé du bouton Run (généré par `on_submit`).
    pub fn run_label(mut self, s: impl Into<String>) -> Self {
        self.run_label = s.into();
        self
    }

    /// Ajoute un composant à la racine de l'application.
    #[allow(clippy::should_implement_trait)]
    pub fn add(mut self, c: impl IntoBox) -> Self {
        self.root.push(c.into_box());
        self
    }

    /// Alias de [`App::add`].
    pub fn item(mut self, c: impl IntoBox) -> Self {
        self.root.push(c.into_box());
        self
    }

    /// Ajoute un conteneur horizontal (côte à côte).
    pub fn row(mut self, task: impl FnOnce(&mut RowBuilder)) -> Self {
        let mut b = RowBuilder::default();
        task(&mut b);
        let mut row = Row::new(format!("row-{}", b.children.len())).gap(b.gap);
        for c in b.children {
            row.children.push(c);
        }
        let built: Box<dyn Component> = if b.layout == Layout::default() {
            Box::new(row)
        } else {
            Box::new(WithLayout::new(row).set_layout(b.layout))
        };
        self.root.push(built);
        self
    }

    /// Ajoute un conteneur vertical (empilé).
    pub fn column(mut self, task: impl FnOnce(&mut RowBuilder)) -> Self {
        let mut b = RowBuilder::default();
        task(&mut b);
        let mut col = Column::new(format!("col-{}", b.children.len())).gap(b.gap);
        for c in b.children {
            col.children.push(c);
        }
        let built: Box<dyn Component> = if b.layout == Layout::default() {
            Box::new(col)
        } else {
            Box::new(WithLayout::new(col).set_layout(b.layout))
        };
        self.root.push(built);
        self
    }

    /// Ajoute un conteneur en grille (CSS Grid).
    pub fn grid(mut self, columns: usize, task: impl FnOnce(&mut RowBuilder)) -> Self {
        let mut b = RowBuilder::default();
        task(&mut b);
        let mut g = Grid::new(format!("grid-{}", b.children.len()))
            .columns(columns)
            .gap(b.gap);
        for c in b.children {
            g.children.push(c);
        }
        let built: Box<dyn Component> = if b.layout == Layout::default() {
            Box::new(g)
        } else {
            Box::new(WithLayout::new(g).set_layout(b.layout))
        };
        self.root.push(built);
        self
    }

    /// Ajoute une carte (panel) à titre + corps empilé.
    pub fn panel(mut self, label: impl Into<String>, task: impl FnOnce(&mut RowBuilder)) -> Self {
        let label = label.into();
        let mut b = RowBuilder::default();
        task(&mut b);
        let mut p = Panel::new(format!("panel-{}", label))
            .label(label)
            .gap(b.gap);
        for c in b.children {
            p.children.push(c);
        }
        let built: Box<dyn Component> = if b.layout == Layout::default() {
            Box::new(p)
        } else {
            Box::new(WithLayout::new(p).set_layout(b.layout))
        };
        self.root.push(built);
        self
    }

    /// Handler de soumission : exécuté à chaque clic sur Run (ou appel
    /// `/api/predict`). Ajoute automatiquement le bouton Run à l'UI.
    pub fn on_submit(
        mut self,
        f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    ) -> Self {
        self.handlers.push(HandlerDef {
            event: EventName::Submit,
            component: None,
            f: Arc::new(f),
            inputs: None,
            outputs: None,
            chain: Vec::new(),
        });
        let btn = Button::new("run").label(self.run_label.clone()).primary();
        self.run_button = Some("run".to_string());
        self.root.push(Box::new(btn));
        self
    }

    /// Handler exécuté au **montage de la page** (connexion WebSocket du client).
    pub fn on_load(
        mut self,
        f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    ) -> Self {
        self.handlers.push(HandlerDef {
            event: EventName::Load,
            component: None,
            f: Arc::new(f),
            inputs: None,
            outputs: None,
            chain: Vec::new(),
        });
        self
    }

    /// Handler exécuté quand un composant d'entrée (`id`) est modifié.
    pub fn on_change(
        mut self,
        id: impl Into<String>,
        f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    ) -> Self {
        self.handlers.push(HandlerDef {
            event: EventName::Change,
            component: Some(id.into()),
            f: Arc::new(f),
            inputs: None,
            outputs: None,
            chain: Vec::new(),
        });
        self
    }

    /// Handler exécuté quand un composant (`id`) reçoit un clic.
    pub fn on_click(
        mut self,
        id: impl Into<String>,
        f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    ) -> Self {
        self.handlers.push(HandlerDef {
            event: EventName::Click,
            component: Some(id.into()),
            f: Arc::new(f),
            inputs: None,
            outputs: None,
            chain: Vec::new(),
        });
        self
    }

    /// Lie la **même fonction** à plusieurs identifiants pour un événement
    /// `"click"`, `"change"`, `"play"`, `"pause"`, `"stop"` ou `"stream"`
    /// (multi-déclencheurs, type `gr.on`).
    pub fn on(
        mut self,
        event: &str,
        ids: impl IntoIterator<Item = impl Into<String>>,
        f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    ) -> Self {
        let ev = match event {
            "click" => EventName::Click,
            "change" => EventName::Change,
            "play" => EventName::Play,
            "pause" => EventName::Pause,
            "stop" => EventName::Stop,
            "stream" => EventName::Stream,
            _ => return self,
        };
        let shared: HandlerFn = Arc::new(f);
        for id in ids {
            self.handlers.push(HandlerDef {
                event: ev.clone(),
                component: Some(id.into()),
                f: Arc::clone(&shared),
                inputs: None,
                outputs: None,
                chain: Vec::new(),
            });
        }
        self
    }

    /// Handler exécuté quand la lecture démarre sur un média (`id`).
    pub fn on_play(
        self,
        id: impl Into<String>,
        f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    ) -> Self {
        self.register(EventName::Play, Some(id.into()), f)
    }

    /// Handler exécuté quand un média (`id`) est mis en pause.
    pub fn on_pause(
        self,
        id: impl Into<String>,
        f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    ) -> Self {
        self.register(EventName::Pause, Some(id.into()), f)
    }

    /// Handler exécuté quand la lecture d'un média (`id`) est arrêtée.
    pub fn on_stop(
        self,
        id: impl Into<String>,
        f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    ) -> Self {
        self.register(EventName::Stop, Some(id.into()), f)
    }

    /// Handler exécuté à chaque fragment reçu d'un **flux streaming**
    /// (`Audio::live`/`Video::live`) pour le composant `id`. Lis le total via
    /// `ctx.get::<StreamInfo>(id)`.
    pub fn on_stream(
        self,
        id: impl Into<String>,
        f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    ) -> Self {
        self.register(EventName::Stream, Some(id.into()), f)
    }

    fn register(
        mut self,
        event: EventName,
        component: Option<String>,
        f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    ) -> Self {
        self.handlers.push(HandlerDef {
            event,
            component,
            f: Arc::new(f),
            inputs: None,
            outputs: None,
            chain: Vec::new(),
        });
        self
    }

    /// Listener d'événement applicatif (déclenché par `ctx.emit(nom, …)`).
    pub fn on_event(
        mut self,
        name: impl Into<String>,
        f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    ) -> Self {
        self.handlers.push(HandlerDef {
            event: EventName::Custom(name.into()),
            component: None,
            f: Arc::new(f),
            inputs: None,
            outputs: None,
            chain: Vec::new(),
        });
        self
    }

    /// Déclare le **flux** du dernier handler enregistré : `inputs` (lectures
    /// autorisées via `ctx.get`) et `outputs` (écritures autorisées — les
    /// `ctx.set`/`append`/`progress` hors liste sont ignorés en silence).
    pub fn flow(mut self, inputs: &[&str], outputs: &[&str]) -> Self {
        if let Some(h) = self.handlers.last_mut() {
            h.inputs = Some(inputs.iter().map(|s| s.to_string()).collect());
            h.outputs = Some(outputs.iter().map(|s| s.to_string()).collect());
        }
        self
    }

    /// Exécute `f` après le **dernier handler** enregistré, quoi qu'il arrive.
    pub fn then(mut self, f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static) -> Self {
        self.chain_link(RunCond::Always, f);
        self
    }

    /// Exécute `f` après le dernier handler, seulement s'il a réussi.
    pub fn success(
        mut self,
        f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    ) -> Self {
        self.chain_link(RunCond::Success, f);
        self
    }

    /// Exécute `f` après le dernier handler, seulement s'il a échoué.
    /// S'il réussit, l'erreur est considérée gérée (les maillons suivants
    /// redeviennent des `success`).
    pub fn failure(
        mut self,
        f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    ) -> Self {
        self.chain_link(RunCond::Failure, f);
        self
    }

    fn chain_link(
        &mut self,
        on: RunCond,
        f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static,
    ) {
        if let Some(h) = self.handlers.last_mut() {
            h.chain.push(Sibling { on, f: Arc::new(f) });
        }
    }

    /// Démarre le serveur HTTP (UI + API REST) de manière asynchrone sur l'adresse donnée.
    pub async fn serve(self, addr: impl Into<String>) -> Result<()> {
        crate::server::serve(self, addr.into()).await
    }

    /// Démarre le serveur HTTP (UI + API REST) sur l'adresse donnée et
    /// bloque jusqu'à l'arrêt (crée un runtime Tokio dédié).
    pub fn launch(self, addr: impl Into<String>) -> Result<()> {
        if tokio::runtime::Handle::try_current().is_ok() {
            // Si un runtime existe déjà, on utilise block_in_place ou on délègue
            tokio::task::block_in_place(|| {
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()?;
                rt.block_on(async move { crate::server::serve(self, addr.into()).await })
            })
        } else {
            let rt = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()?;
            rt.block_on(async move { crate::server::serve(self, addr.into()).await })
        }
    }

    /// **Export HTML Autonome Statique** : Génère un document HTML 100% autonome
    /// avec tous les styles CSS et les scripts JS inlinés (0 dépendance).
    pub fn render_html_bundle(&self) -> String {
        crate::server::render_standalone_html(self)
    }

    /// **Mode Desktop Standalone** : Démarre le serveur et ouvre automatiquement
    /// l'application dans une fenêtre native dédiée (Frameless App Window).
    pub fn launch_desktop(self, addr: impl Into<String>) -> Result<()> {
        let addr_str: String = addr.into();
        let target_url = if addr_str.starts_with("0.0.0.0:") {
            format!(
                "http://localhost:{}",
                addr_str.trim_start_matches("0.0.0.0:")
            )
        } else if !addr_str.starts_with("http://") && !addr_str.starts_with("https://") {
            format!("http://{addr_str}")
        } else {
            addr_str.clone()
        };

        let url_to_open = target_url.clone();
        std::thread::spawn(move || {
            std::thread::sleep(std::time::Duration::from_millis(300));
            open_native_window(&url_to_open);
        });

        self.launch(addr_str)
    }
}

/// Ouvre l'URL dans une fenêtre d'application autonome dédiée.
fn open_native_window(url: &str) {
    #[cfg(target_os = "windows")]
    {
        let candidates = [
            r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
            r"C:\Program Files\Google\Chrome\Application\chrome.exe",
            r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
            r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
        ];

        let mut launched = false;
        for path in &candidates {
            if std::path::Path::new(path).exists() {
                let app_arg = format!("--app={url}");
                let size_arg = "--window-size=1050,800";
                if std::process::Command::new(path)
                    .args([&app_arg, size_arg])
                    .spawn()
                    .is_ok()
                {
                    launched = true;
                    break;
                }
            }
        }

        if !launched {
            let _ = std::process::Command::new("powershell")
                .args(["-NoProfile", "-Command", &format!("Start-Process '{url}'")])
                .spawn();
        }
    }

    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }

    #[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}

/// Traite un événement client : distribue aux handlers, fait tourner le bus
/// applicatif, et construit la réponse finale (toutes les mises à jour
/// accumulées, ou une erreur).
pub(crate) fn process_event(ev: &WireEvent, app: &App, ctx: &mut Context) -> Value {
    let mut error: Option<Error> = None;

    for h in handlers_for_client(ev, app) {
        ctx.set_flow(h.inputs.as_deref(), h.outputs.as_deref());
        if let Err(e) = run_handler(h, ctx) {
            error = Some(e);
            break;
        }
    }
    ctx.set_flow(None, None);

    if error.is_none() {
        let mut budget = 0usize;
        loop {
            let emitted = ctx.take_emitted();
            if emitted.is_empty() {
                break;
            }
            budget += emitted.len();
            if budget > 64 {
                error = Some(Error::from("event loop guard reached"));
                break;
            }
            for (name, _data) in emitted {
                for h in handlers_for_custom(app, &name) {
                    if let Err(e) = (h.f)(ctx) {
                        error = Some(e);
                        break;
                    }
                }
                if error.is_some() {
                    break;
                }
            }
            if error.is_some() {
                break;
            }
        }
    }

    if let Some(e) = error {
        return json!({ "t": "error", "m": e.to_string() });
    }

    let all = ctx.take_all();
    let u: Vec<Value> = all
        .into_iter()
        .map(|(id, patch)| json!({ "id": id, "p": patch }))
        .collect();
    json!({ "t": "update", "u": u })
}

/// Exécute un handler principal puis sa chaîne (`.then` / `.success` /
/// `.failure`). Un maillon `failure` réussi efface l'erreur accumulée.
fn run_handler(h: &HandlerDef, ctx: &mut Context) -> Result<()> {
    let mut error: Option<Error> = (h.f)(ctx).err();
    for sib in &h.chain {
        let should_run = match sib.on {
            RunCond::Always => true,
            RunCond::Success => error.is_none(),
            RunCond::Failure => error.is_some(),
        };
        if !should_run {
            continue;
        }
        if let Err(e) = (sib.f)(ctx) {
            if error.is_none() {
                error = Some(e);
            }
        } else if sib.on == RunCond::Failure {
            error = None;
        }
    }
    match error {
        Some(e) => Err(e),
        None => Ok(()),
    }
}

fn handlers_for_client<'a>(ev: &WireEvent, app: &'a App) -> Vec<&'a HandlerDef> {
    app.handlers
        .iter()
        .filter(|h| match &h.event {
            EventName::Submit => {
                app.run_button.as_deref() == Some(ev.c.as_str()) || (app.live && ev.e == "change")
            }
            EventName::Change => ev.e == "change" && h.component.as_deref() == Some(ev.c.as_str()),
            EventName::Click => ev.e == "click" && h.component.as_deref() == Some(ev.c.as_str()),
            EventName::Play => ev.e == "play" && h.component.as_deref() == Some(ev.c.as_str()),
            EventName::Pause => ev.e == "pause" && h.component.as_deref() == Some(ev.c.as_str()),
            EventName::Stop => ev.e == "stop" && h.component.as_deref() == Some(ev.c.as_str()),
            EventName::Stream => ev.e == "stream" && h.component.as_deref() == Some(ev.c.as_str()),
            EventName::Load => ev.e == "load",
            EventName::Custom(_) => false,
        })
        .collect()
}

fn handlers_for_custom<'a>(app: &'a App, name: &str) -> Vec<&'a HandlerDef> {
    app.handlers
        .iter()
        .filter(|h| matches!(&h.event, EventName::Custom(n) if n == name))
        .collect()
}
