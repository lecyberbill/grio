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
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeMode {
    /// Mode sombre forcé.
    Dark,
    /// Mode clair forcé.
    Light,
    /// Détection automatique selon les préférences de l'OS/navigateur.
    System,
}

impl Default for ThemeMode {
    fn default() -> Self {
        Self::System
    }
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
        Self { mode: ThemeMode::Dark, ..Default::default() }
    }
    /// Thème clair moderne par défaut.
    pub fn light() -> Self {
        Self { mode: ThemeMode::Light, ..Default::default() }
    }
    /// Thème adaptatif calqué sur l'OS/navigateur.
    pub fn system() -> Self {
        Self { mode: ThemeMode::System, ..Default::default() }
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
}

/// Collecteur d'éléments pour `App::row` / `column` / `panel` / `grid`.
pub struct RowBuilder {
    pub(crate) children: Vec<Box<dyn Component>>,
    pub(crate) gap: f64,
    pub(crate) layout: Layout,
}

impl Default for RowBuilder {
    fn default() -> Self {
        Self { children: Vec::new(), gap: 16.0, layout: Layout::default() }
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
            self.children.push(Box::new(WithLayout::new(r).set_layout(b.layout)));
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
            self.children.push(Box::new(WithLayout::new(col).set_layout(b.layout)));
        }
    }
    /// Ajoute une sous-grille au groupe.
    pub fn grid(&mut self, columns: usize, task: impl FnOnce(&mut RowBuilder)) {
        let mut b = RowBuilder::default();
        task(&mut b);
        let mut g = Grid::new(format!("grid-{}", self.children.len())).columns(columns).gap(b.gap);
        for c in b.children {
            g.children.push(c);
        }
        if b.layout == Layout::default() {
            self.children.push(Box::new(g));
        } else {
            self.children.push(Box::new(WithLayout::new(g).set_layout(b.layout)));
        }
    }
    /// Ajoute un panneau au groupe.
    pub fn panel(&mut self, label: impl Into<String>, task: impl FnOnce(&mut RowBuilder)) {
        let label = label.into();
        let mut b = RowBuilder::default();
        task(&mut b);
        let mut p = Panel::new(format!("panel-{}", label)).label(label).gap(b.gap);
        for c in b.children {
            p.children.push(c);
        }
        if b.layout == Layout::default() {
            self.children.push(Box::new(p));
        } else {
            self.children.push(Box::new(WithLayout::new(p).set_layout(b.layout)));
        }
    }
    /// Espace entre les composants du groupe, en pixels.
    pub fn gap(&mut self, g: f64) {
        self.gap = g;
    }
    /// Largeur du groupe, en pixels.
    pub fn width(&mut self, w: u32) { self.layout.width = Some(w); }
    /// Hauteur du groupe, en pixels.
    pub fn height(&mut self, h: u32) { self.layout.height = Some(h); }
    /// Proportion du groupe dans la colonne/racine (comme `scale` de Gradio).
    pub fn scale(&mut self, s: u32) { self.layout.scale = Some(s); }
    /// Largeur minimale du groupe, en pixels.
    pub fn min_width(&mut self, w: u32) { self.layout.min_width = Some(w); }
}

impl App {
    /// Crée une application avec un titre de page.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: String::new(),
            root: Column::new("root").gap(16.0),
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
        }
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

    /// Ajoute un composant directement à la racine (colonne).
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
        let mut g = Grid::new(format!("grid-{}", b.children.len())).columns(columns).gap(b.gap);
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
        let mut p = Panel::new(format!("panel-{}", label)).label(label).gap(b.gap);
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
    pub fn on_submit(mut self, f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static) -> Self {
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
    pub fn on_load(mut self, f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static) -> Self {
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
    pub fn on_change(mut self, id: impl Into<String>, f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static) -> Self {
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
    pub fn on_click(mut self, id: impl Into<String>, f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static) -> Self {
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
    pub fn on(mut self, event: &str, ids: impl IntoIterator<Item = impl Into<String>>, f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static) -> Self {
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
    pub fn on_play(self, id: impl Into<String>, f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static) -> Self {
        self.register(EventName::Play, Some(id.into()), f)
    }

    /// Handler exécuté quand un média (`id`) est mis en pause.
    pub fn on_pause(self, id: impl Into<String>, f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static) -> Self {
        self.register(EventName::Pause, Some(id.into()), f)
    }

    /// Handler exécuté quand la lecture d'un média (`id`) est arrêtée.
    pub fn on_stop(self, id: impl Into<String>, f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static) -> Self {
        self.register(EventName::Stop, Some(id.into()), f)
    }

    /// Handler exécuté à chaque fragment reçu d'un **flux streaming**
    /// (`Audio::live`/`Video::live`) pour le composant `id`. Lis le total via
    /// `ctx.get::<StreamInfo>(id)`.
    pub fn on_stream(self, id: impl Into<String>, f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static) -> Self {
        self.register(EventName::Stream, Some(id.into()), f)
    }

    fn register(mut self, event: EventName, component: Option<String>, f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static) -> Self {
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
    pub fn on_event(mut self, name: impl Into<String>, f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static) -> Self {
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
    pub fn success(mut self, f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static) -> Self {
        self.chain_link(RunCond::Success, f);
        self
    }

    /// Exécute `f` après le dernier handler, seulement s'il a échoué.
    /// S'il réussit, l'erreur est considérée gérée (les maillons suivants
    /// redeviennent des `success`).
    pub fn failure(mut self, f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static) -> Self {
        self.chain_link(RunCond::Failure, f);
        self
    }

    fn chain_link(&mut self, on: RunCond, f: impl Fn(&mut Context) -> Result<()> + Send + Sync + 'static) {
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
                let rt = tokio::runtime::Builder::new_current_thread().enable_all().build()?;
                rt.block_on(async move { crate::server::serve(self, addr.into()).await })
            })
        } else {
            let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build()?;
            rt.block_on(async move { crate::server::serve(self, addr.into()).await })
        }
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
    let u: Vec<Value> =
        all.into_iter().map(|(id, patch)| json!({ "id": id, "p": patch })).collect();
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
