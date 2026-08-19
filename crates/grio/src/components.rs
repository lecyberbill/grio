//! Composants de l'UI : les éléments exposés à l'utilisateur du moteur.
//!
//! Chaque composant est déclaré via son constructeur, configuré par chaînage
//! de méthodes (builder), puis inséré dans un conteneur ou à la racine de
//! l'application (`App::item`, `App::row`, …).
//!
//! Le rôle de chaque composant (`Role::Input` / `Role::Output`) est
//! déclaratif : il permet d'identifier automatiquement les entrées et les
//! sorties, donc de générer l'API REST (`/api/predict`, `/api/schema`).

use serde_json::{json, Value};

use crate::app::RowBuilder;

/// Rôle d'un composant pour l'API REST automatique.
///
/// * `Input` — composant d'entrée : ses valeurs alimentent `on_submit`.
/// * `Output` — composant de sortie : il reçoit les résultats calculés.
/// * `None` — aucun rôle (boutons, conteneurs, décoration).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    /// Composant d'entrée : alimente `on_submit` et `/api/predict`.
    Input,
    /// Composant de sortie : reçoit les résultats.
    Output,
    /// Aucun rôle (boutons, conteneurs, décoration).
    None,
}

/// Contrat minimal d'un composant.
///
/// Un type qui implémente `Component` est utilisable n'importe où dans
/// l'arbre de l'application. Le rendu lui-même est délégué au front
/// (`app.js`), le serveur ne transmet qu'un identifiant, un « kind » et une
/// configuration JSON (`props`).
pub trait Component: Send + Sync {
    /// Identifiant unique du composant — cible des événements et licences
    /// `ctx.get/set`.
    fn id(&self) -> &str;

    /// Nom logique du composant (ex. `"text"`, `"slider"`, `"row"`). Doit
    /// correspondre à une entrée du registre JavaScript.
    fn kind(&self) -> &'static str;

    /// Configuration sérialisée envoyée au client au montage.
    fn props(&self) -> Value;

    /// Rôle pour le classement automatique de l'API REST.
    fn role(&self) -> Role {
        Role::None
    }

    /// Mise en page commune (dimensions, proportion) — fusionnée dans
    /// `props` au rendu par le serveur. Vaut `None` partout par défaut.
    fn layout(&self) -> Layout {
        Layout::default()
    }

    /// Enfants — utilisé pour le rendu récursif des conteneurs.
    fn children(&self) -> Vec<&dyn Component> {
        Vec::new()
    }
}

/// Mise en page commune à **tous** les composants : dimensions en pixels et
/// proportion relative au sein d'une ligne (comme `scale`/`min_width` de
/// Gradio). Les réglages non remplis (`None`) sont simplement omis du rendu.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Layout {
    /// Largeur en pixels (CSS `width`).
    pub width: Option<u32>,
    /// Hauteur en pixels (CSS `height`).
    pub height: Option<u32>,
    /// Proportion relative dans la ligne (CSS `flex-grow`).
    pub scale: Option<u32>,
    /// Largeur minimale en pixels (CSS `min-width`).
    pub min_width: Option<u32>,
}

impl Layout {
    /// JSON des seuls réglages non vides — fusionné dans `props` au rendu.
    pub fn json(self) -> Value {
        let mut o = serde_json::Map::new();
        if let Some(w) = self.width {
            o.insert("width".into(), json!(w));
        }
        if let Some(h) = self.height {
            o.insert("height".into(), json!(h));
        }
        if let Some(s) = self.scale {
            o.insert("scale".into(), json!(s));
        }
        if let Some(m) = self.min_width {
            o.insert("min_width".into(), json!(m));
        }
        Value::Object(o)
    }
}

/// Enveloppe **modulaire** : ajoute une mise en page (`.width()`, `.height()`,
/// `.scale()`, `.min_width()`) à n'importe quel composant — brique **ou**
/// conteneur (`Row`/`Column`/`Panel` construits à la main) — sans rien
/// modifier au type d'origine :
///
/// ```rust
/// # use grio::*;
/// WithLayout::new(Text::new("a")).width(240).min_width(120);
/// WithLayout::new(Row::new("r").item(Text::new("a"))).scale(2);
/// ```
pub struct WithLayout<C> {
    inner: C,
    layout: Layout,
}

impl<C: Component> WithLayout<C> {
    /// Enveloppe `inner` avec une mise en page vide.
    pub fn new(inner: C) -> Self {
        Self { inner, layout: Layout::default() }
    }
    /// Largeur en pixels.
    pub fn width(mut self, w: u32) -> Self { self.layout.width = Some(w); self }
    /// Hauteur en pixels.
    pub fn height(mut self, h: u32) -> Self { self.layout.height = Some(h); self }
    /// Proportion relative dans la ligne (comme `scale` de Gradio).
    pub fn scale(mut self, s: u32) -> Self { self.layout.scale = Some(s); self }
    /// Largeur minimale en pixels.
    pub fn min_width(mut self, w: u32) -> Self { self.layout.min_width = Some(w); self }
    /// Remplace la mise en page en un appel.
    pub fn set_layout(mut self, l: Layout) -> Self { self.layout = l; self }
    /// Restitue le composant d'origine.
    pub fn into_inner(self) -> C { self.inner }
    /// Réglage courant.
    pub fn layout(&self) -> Layout { self.layout }
}

impl<C: Component> Component for WithLayout<C> {
    fn id(&self) -> &str { self.inner.id() }
    fn kind(&self) -> &'static str { self.inner.kind() }
    fn props(&self) -> Value { self.inner.props() }
    fn role(&self) -> Role { self.inner.role() }
    fn children(&self) -> Vec<&dyn Component> { self.inner.children() }
    fn layout(&self) -> Layout { self.layout }
}

/// Conversion commode `T: Component → Box<dyn Component>`.
pub trait IntoBox {
    /// Emboîte la valeur dans un trait-objet.
    fn into_box(self) -> Box<dyn Component>;
}

impl<T> IntoBox for T
where
    T: Component + 'static,
{
    fn into_box(self) -> Box<dyn Component> {
        Box::new(self)
    }
}

fn children_refs(items: &[Box<dyn Component>]) -> Vec<&dyn Component> {
    items.iter().map(|b| b.as_ref() as &dyn Component).collect()
}

/// Champ de texte sur une ligne (ou plusieurs lignes via `.lines(n)`).
#[derive(Clone, Debug)]
pub struct Text {
    id: String,
    label: String,
    value: String,
    placeholder: Option<String>,
    lines: usize,
    interactive: bool,
}

impl Text {
    /// Crée un champ texte, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), label: String::new(), value: String::new(), placeholder: None, lines: 1, interactive: true }
    }
    /// Définit l'étiquette affichée au-dessus du champ.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Valeur initiale.
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = v.into(); self }
    /// Texte indicatif affiché quand le champ est vide.
    pub fn placeholder(mut self, p: impl Into<String>) -> Self { self.placeholder = Some(p.into()); self }
    /// Nombre de lignes affichées (1 = input classique, >1 = textarea).
    pub fn lines(mut self, n: usize) -> Self { self.lines = n.max(1); self }
    /// Autorise ou non l'édition par l'utilisateur (champ grisé ; la valeur
    /// reste envoyée dans le snapshot d'entrées).
    pub fn interactive(mut self, on: bool) -> Self { self.interactive = on; self }
}

impl Component for Text {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "text" }
    fn role(&self) -> Role { Role::Input }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "value": self.value,
            "placeholder": self.placeholder,
            "lines": self.lines,
            "interactive": self.interactive
        })
    }
}

/// Curseur numérique.
#[derive(Clone, Debug)]
pub struct Slider {
    id: String,
    label: String,
    min: f64,
    max: f64,
    step: f64,
    value: f64,
    interactive: bool,
}

impl Slider {
    /// Crée un curseur, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), label: String::new(), min: 0.0, max: 100.0, step: 1.0, value: 0.0, interactive: true }
    }
    /// Étiquette affichée au-dessus du curseur.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Borne minimum (f64).
    pub fn min(mut self, v: f64) -> Self { self.min = v; self }
    /// Borne maximum (f64).
    pub fn max(mut self, v: f64) -> Self { self.max = v; self }
    /// Pas d'incrément (f64).
    pub fn step(mut self, v: f64) -> Self { self.step = v; self }
    /// Valeur initiale.
    pub fn value(mut self, v: f64) -> Self { self.value = v; self }
    /// Autorise ou non la manipulation par l'utilisateur (curseur grisé ;
    /// la valeur reste envoyée dans le snapshot d'entrées).
    pub fn interactive(mut self, on: bool) -> Self { self.interactive = on; self }
}

impl Component for Slider {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "slider" }
    fn role(&self) -> Role { Role::Input }
    fn props(&self) -> Value {
        json!({ "label": self.label, "min": self.min, "max": self.max, "step": self.step, "value": self.value, "interactive": self.interactive })
    }
}

/// Sortie texte en lecture seule (carte).
#[derive(Clone, Debug)]
pub struct Output {
    id: String,
    label: String,
    value: String,
}

impl Output {
    /// Crée une sortie texte, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), label: String::new(), value: String::new() }
    }
    /// Étiquette affichée en tête de la carte.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Valeur initiale.
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = v.into(); self }
}

impl Component for Output {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "output" }
    fn role(&self) -> Role { Role::Output }
    fn props(&self) -> Value {
        json!({ "label": self.label, "value": self.value })
    }
}

/// Bloc de texte Markdown (mini-rendu côté client).
#[derive(Clone, Debug)]
pub struct Markdown {
    id: String,
    text: String,
}

impl Markdown {
    /// Crée un bloc Markdown, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), text: String::new() }
    }
    /// Contenu Markdown initial.
    pub fn text(mut self, t: impl Into<String>) -> Self { self.text = t.into(); self }
    /// Alias de [`Markdown::text`] pour cohérence avec les autres composants.
    pub fn value(mut self, t: impl Into<String>) -> Self { self.text = t.into(); self }
}

impl Component for Markdown {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "markdown" }
    fn role(&self) -> Role { Role::Output }
    fn props(&self) -> Value {
        json!({ "text": self.text })
    }
}

/// Bouton. `primary` étiquette le bouton de soumission global (Run).
#[derive(Clone, Debug)]
pub struct Button {
    id: String,
    label: String,
    variant: String,
    primary: bool,
}

impl Button {
    /// Crée un bouton, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), label: String::new(), variant: "primary".to_string(), primary: false }
    }
    /// Texte affiché sur le bouton.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Style secondaire (contour). Style principal par défaut.
    pub fn secondary(mut self) -> Self { self.variant = "secondary".to_string(); self }
    /// Marque le bouton comme déclencheur de soumission (`on_submit`).
    pub fn primary(mut self) -> Self { self.primary = true; self }
}

impl Component for Button {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "button" }
    fn props(&self) -> Value {
        json!({ "label": self.label, "variant": self.variant, "primary": self.primary })
    }
}

/// Barre de progression (cible de `ctx.progress`).
///
/// Le composant ne stocke pas de valeur déclarative : il est entièrement
/// piloté par les mises à jour envoyées par un handler (`ctx.progress`).
#[derive(Clone, Debug)]
pub struct Progress {
    id: String,
    label: String,
}

impl Progress {
    /// Crée une barre de progression, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), label: String::new() }
    }
    /// Étiquette affichée au-dessus de la barre.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
}

impl Component for Progress {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "progress" }
    fn props(&self) -> Value {
        json!({ "label": self.label })
    }
}

/// Image : **upload** (entrée, interactive) et/ou **affichage** (serveur → client
/// via `ctx.set`). La valeur transportée est une **data URL**
/// (`data:image/png;base64,…`).
#[derive(Clone, Debug)]
pub struct Image {
    id: String,
    label: String,
    value: String,
    interactive: bool,
    out: bool,
}

impl Image {
    /// Crée une image, avec son identifiant. Entrée (upload) par défaut.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), label: String::new(), value: String::new(), interactive: true, out: false }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Data URL initiale affichée.
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = v.into(); self }
    /// Ouvre/ferme le contrôle d'upload (fichier ou glisser-déposer).
    pub fn interactive(mut self, on: bool) -> Self { self.interactive = on; self }
    /// Déclare l'image **sortie** (résultat calculé, pas d'upload).
    pub fn output(mut self) -> Self { self.out = true; self }
    /// Déclare l'image **entrée** (upload — comportement par défaut).
    pub fn input(mut self) -> Self { self.out = false; self }
}

impl Component for Image {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "image" }
    fn role(&self) -> Role { if self.out { Role::Output } else { Role::Input } }
    fn props(&self) -> Value {
        json!({ "label": self.label, "value": self.value, "interactive": self.interactive })
    }
}

/// Éditeur d'image **côté client** : pinceau, gomme, formes (rect/ligne/
/// flèche), rognage (crop), rotation, filtres, annuler/rétablir, zoom/pan et
/// **calques** (2 par défaut, `.layers(n)`).
///
/// **Sortie (entrée du serveur)** — la valeur émise au `change` est un objet :
///
/// ```json
/// {
///   "image":  "data:image/png;base64,…",   // composite (fond + calques visibles)
///   "layers": ["data:…", "data:…"],         // un PNG par calque d'annotation (RGBA)
///   "mask":   "data:image/png;base64,…"     // calques rendus en blanc sur fond noir
/// }
/// ```
///
/// Le `mask` donne directement les zones à retoucher : blanc = trait d'un
/// pinceau/forme — typique de l'**inpainting** (le serveur repeint les zones
/// blanches du masque). Lire via `ctx.get::<serde_json::Value>("id")`.
#[derive(Clone, Debug)]
pub struct ImageEditor {
    id: String,
    label: String,
    value: String,
    interactive: bool,
    layers: usize,
    brush: bool,
    crop: bool,
    shapes: bool,
    filters: bool,
    rotflip: bool,
    out: bool,
}

impl ImageEditor {
    /// Crée un éditeur d'image, avec son identifiant. Entrée par défaut.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            value: String::new(),
            interactive: true,
            layers: 2,
            brush: true,
            crop: true,
            shapes: true,
            filters: true,
            rotflip: true,
            out: false,
        }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Image d'arrière-plan initiale (data URL) chargée dans le canvas.
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = v.into(); self }
    /// Autorise ou non l'édition (affichage seul si `false`).
    pub fn interactive(mut self, on: bool) -> Self { self.interactive = on; self }
    /// Nombre de **calques** d'annotation (1..=4, défaut 2).
    pub fn layers(mut self, n: usize) -> Self { self.layers = n.clamp(1, 4); self }
    /// Active le pinceau et la gomme (défaut : `true`).
    pub fn brush(mut self, on: bool) -> Self { self.brush = on; self }
    /// Active le **rognage** (crop) (défaut : `true`).
    pub fn crop(mut self, on: bool) -> Self { self.crop = on; self }
    /// Active les formes : rectangle, ligne, flèche (défaut : `true`).
    pub fn shapes(mut self, on: bool) -> Self { self.shapes = on; self }
    /// Active les filtres (niveaux de gris, inversion, luminosité, flou).
    pub fn filters(mut self, on: bool) -> Self { self.filters = on; self }
    /// Active la rotation 90° (défaut : `true`).
    pub fn rotflip(mut self, on: bool) -> Self { self.rotflip = on; self }
    /// Déclare l'éditeur **sortie** (affichage seul de `value`).
    pub fn output(mut self) -> Self { self.out = true; self }
    /// Déclare l'éditeur **entrée** (édition — par défaut).
    pub fn input(mut self) -> Self { self.out = false; self }
}

impl Component for ImageEditor {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "imageeditor" }
    fn role(&self) -> Role { if self.out { Role::Output } else { Role::Input } }
    fn props(&self) -> Value {
        json!({
            "label": self.label, "value": self.value, "interactive": self.interactive,
            "layers": self.layers, "brush": self.brush, "crop": self.crop,
            "shapes": self.shapes, "filters": self.filters, "rotflip": self.rotflip
        })
    }
}

/// Audio : **upload** (entrée, interactive), **affichage/lecteur** (sortie) ou
/// **micro live** (`.live(true)` → streaming via `{t:"stream"}`).
#[derive(Clone, Debug)]
pub struct Audio {
    id: String,
    label: String,
    value: String,
    interactive: bool,
    out: bool,
    live: bool,
}

impl Audio {
    /// Crée un audio, avec son identifiant. Entrée par défaut.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), label: String::new(), value: String::new(), interactive: true, out: false, live: false }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Data URL initiale.
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = v.into(); self }
    /// Ouvre/ferme le contrôle d'upload.
    pub fn interactive(mut self, on: bool) -> Self { self.interactive = on; self }
    /// Déclare l'audio **sortie** (lecteur de résultats).
    pub fn output(mut self) -> Self { self.out = true; self }
    /// Déclare l'audio **entrée** (upload — par défaut).
    pub fn input(mut self) -> Self { self.out = false; self }
    /// Mode **micro live** : le client capture le micro et pousse des chunks
    /// (`{t:"stream"}`) — voir `App::on_stream`.
    pub fn live(mut self, on: bool) -> Self { self.live = on; self }
}

impl Component for Audio {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "audio" }
    fn role(&self) -> Role { if self.out { Role::Output } else { Role::Input } }
    fn props(&self) -> Value {
        json!({ "label": self.label, "value": self.value, "interactive": self.interactive, "live": self.live })
    }
}

/// Vidéo : **affichage/lecteur** (sortie, par défaut) ou **caméra live**
/// (`.live(true)`) ou **upload** (`.interactive(true)`).
#[derive(Clone, Debug)]
pub struct Video {
    id: String,
    label: String,
    value: String,
    interactive: bool,
    out: bool,
    live: bool,
}

impl Video {
    /// Crée une vidéo, avec son identifiant. Sortie (affichage) par défaut.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), label: String::new(), value: String::new(), interactive: false, out: true, live: false }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Data URL initiale (ou URL de flux).
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = v.into(); self }
    /// Ouvre/ferme le contrôle d'upload.
    pub fn interactive(mut self, on: bool) -> Self { self.interactive = on; self }
    /// Déclare la vidéo **entrée** (upload).
    pub fn input(mut self) -> Self { self.out = false; self }
    /// Déclare la vidéo **sortie** (affichage — par défaut).
    pub fn output(mut self) -> Self { self.out = true; self }
    /// Mode **caméra live** : la caméra s'affiche en local et pousse des
    /// chunks `{t:"stream"}`.
    pub fn live(mut self, on: bool) -> Self { self.live = on; self }
}

impl Component for Video {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "video" }
    fn role(&self) -> Role { if self.out { Role::Output } else { Role::Input } }
    fn props(&self) -> Value {
        json!({ "label": self.label, "value": self.value, "interactive": self.interactive, "live": self.live })
    }
}

/// Conteneur horizontal : les enfants s'affichent côte à côte (flex).
pub struct Row {
    pub(crate) id: String,
    pub(crate) gap: f64,
    pub(crate) wrap: bool,
    pub(crate) align: Option<String>,
    pub(crate) justify: Option<String>,
    pub(crate) children: Vec<Box<dyn Component>>,
}

impl Row {
    /// Crée une rangée, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            gap: 16.0,
            wrap: true,
            align: None,
            justify: None,
            children: Vec::new(),
        }
    }
    /// Espace entre les enfants, en pixels.
    pub fn gap(mut self, g: f64) -> Self { self.gap = g; self }
    /// Autorise ou interdit le retour à la ligne (`flex-wrap`).
    pub fn wrap(mut self, w: bool) -> Self { self.wrap = w; self }
    /// Alignement transversal (CSS `align-items` : `"start"`, `"center"`, `"end"`, `"stretch"`).
    pub fn align(mut self, a: impl Into<String>) -> Self { self.align = Some(a.into()); self }
    /// Justification principale (CSS `justify-content` : `"start"`, `"center"`, `"end"`, `"space-between"`, `"space-around"`).
    pub fn justify(mut self, j: impl Into<String>) -> Self { self.justify = Some(j.into()); self }
    /// Ajoute un composant dans la rangée.
    pub fn item(mut self, c: impl IntoBox) -> Self { self.children.push(c.into_box()); self }
}

impl Component for Row {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "row" }
    fn props(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("gap".into(), json!(self.gap));
        obj.insert("wrap".into(), json!(self.wrap));
        if let Some(ref a) = self.align {
            obj.insert("align".into(), json!(a));
        }
        if let Some(ref j) = self.justify {
            obj.insert("justify".into(), json!(j));
        }
        Value::Object(obj)
    }
    fn children(&self) -> Vec<&dyn Component> { children_refs(&self.children) }
}

/// Conteneur vertical : les enfants s'empilent (flex column).
pub struct Column {
    pub(crate) id: String,
    pub(crate) gap: f64,
    pub(crate) align: Option<String>,
    pub(crate) justify: Option<String>,
    pub(crate) children: Vec<Box<dyn Component>>,
}

impl Column {
    /// Crée une colonne, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            gap: 16.0,
            align: None,
            justify: None,
            children: Vec::new(),
        }
    }
    /// Espace entre les enfants, en pixels.
    pub fn gap(mut self, g: f64) -> Self { self.gap = g; self }
    /// Alignement transversal (CSS `align-items` : `"start"`, `"center"`, `"end"`, `"stretch"`).
    pub fn align(mut self, a: impl Into<String>) -> Self { self.align = Some(a.into()); self }
    /// Justification principale (CSS `justify-content` : `"start"`, `"center"`, `"end"`, `"space-between"`).
    pub fn justify(mut self, j: impl Into<String>) -> Self { self.justify = Some(j.into()); self }
    /// Ajoute un composant dans la colonne.
    pub fn item(mut self, c: impl IntoBox) -> Self { self.children.push(c.into_box()); self }
    pub(crate) fn push(&mut self, c: Box<dyn Component>) { self.children.push(c); }
}

impl Component for Column {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "column" }
    fn props(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("gap".into(), json!(self.gap));
        if let Some(ref a) = self.align {
            obj.insert("align".into(), json!(a));
        }
        if let Some(ref j) = self.justify {
            obj.insert("justify".into(), json!(j));
        }
        Value::Object(obj)
    }
    fn children(&self) -> Vec<&dyn Component> { children_refs(&self.children) }
}

/// Conteneur en grille CSS : organise ses enfants en lignes et colonnes.
pub struct Grid {
    pub(crate) id: String,
    pub(crate) columns: usize,
    pub(crate) gap: f64,
    pub(crate) gap_x: Option<f64>,
    pub(crate) gap_y: Option<f64>,
    pub(crate) children: Vec<Box<dyn Component>>,
}

impl Grid {
    /// Crée une grille avec son identifiant et son nombre de colonnes par défaut (2).
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            columns: 2,
            gap: 16.0,
            gap_x: None,
            gap_y: None,
            children: Vec::new(),
        }
    }
    /// Nombre de colonnes de la grille.
    pub fn columns(mut self, cols: usize) -> Self {
        self.columns = cols.max(1);
        self
    }
    /// Espacement global (horizontal et vertical) en pixels.
    pub fn gap(mut self, g: f64) -> Self { self.gap = g; self }
    /// Espacement horizontal en pixels.
    pub fn gap_x(mut self, gx: f64) -> Self { self.gap_x = Some(gx); self }
    /// Espacement vertical en pixels.
    pub fn gap_y(mut self, gy: f64) -> Self { self.gap_y = Some(gy); self }
    /// Ajoute un composant dans la grille.
    pub fn item(mut self, c: impl IntoBox) -> Self { self.children.push(c.into_box()); self }
}

impl Component for Grid {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "grid" }
    fn props(&self) -> Value {
        let mut obj = serde_json::Map::new();
        obj.insert("columns".into(), json!(self.columns));
        obj.insert("gap".into(), json!(self.gap));
        if let Some(gx) = self.gap_x {
            obj.insert("gap_x".into(), json!(gx));
        }
        if let Some(gy) = self.gap_y {
            obj.insert("gap_y".into(), json!(gy));
        }
        Value::Object(obj)
    }
    fn children(&self) -> Vec<&dyn Component> { children_refs(&self.children) }
}

/// Carte (panel) avec un titre et un corps empilé en colonne.
pub struct Panel {
    pub(crate) id: String,
    pub(crate) label: String,
    pub(crate) gap: f64,
    pub(crate) children: Vec<Box<dyn Component>>,
}

impl Panel {
    /// Crée une carte, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), label: String::new(), gap: 16.0, children: Vec::new() }
    }
    /// Titre affiché dans l'entête de la carte.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Espace entre les enfants, en pixels.
    pub fn gap(mut self, g: f64) -> Self { self.gap = g; self }
    /// Ajoute un composant dans la carte.
    pub fn item(mut self, c: impl IntoBox) -> Self { self.children.push(c.into_box()); self }
}

impl Component for Panel {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "panel" }
    fn props(&self) -> Value { json!({ "label": self.label, "gap": self.gap }) }
    fn children(&self) -> Vec<&dyn Component> { children_refs(&self.children) }
}

/// Collecteur d'éléments pour une section d'onglet / accordéon.
pub struct SectionBuilder {
    children: Vec<Box<dyn Component>>,
    gap: f64,
}

impl Default for SectionBuilder {
    fn default() -> Self {
        Self { children: Vec::new(), gap: 14.0 }
    }
}

impl SectionBuilder {
    /// Ajoute un composant à la section en construction.
    pub fn item(&mut self, c: impl IntoBox) {
        self.children.push(c.into_box());
    }
    /// Ajoute une ligne fluide à la section.
    pub fn row(&mut self, task: impl FnOnce(&mut RowBuilder)) {
        let mut b = RowBuilder::default();
        task(&mut b);
        let mut r = Row::new(format!("row-{}", self.children.len())).gap(b.gap);
        for c in b.children {
            r.children.push(c);
        }
        self.children.push(Box::new(r));
    }
    /// Ajoute une colonne à la section.
    pub fn column(&mut self, task: impl FnOnce(&mut RowBuilder)) {
        let mut b = RowBuilder::default();
        task(&mut b);
        let mut col = Column::new(format!("col-{}", self.children.len())).gap(b.gap);
        for c in b.children {
            col.children.push(c);
        }
        self.children.push(Box::new(col));
    }
    /// Ajoute une grille à la section.
    pub fn grid(&mut self, columns: usize, task: impl FnOnce(&mut RowBuilder)) {
        let mut b = RowBuilder::default();
        task(&mut b);
        let mut g = Grid::new(format!("grid-{}", self.children.len())).columns(columns).gap(b.gap);
        for c in b.children {
            g.children.push(c);
        }
        self.children.push(Box::new(g));
    }
    /// Ajoute un panneau repliable à la section.
    pub fn panel(&mut self, label: impl Into<String>, task: impl FnOnce(&mut RowBuilder)) {
        let label = label.into();
        let mut b = RowBuilder::default();
        task(&mut b);
        let mut p = Panel::new(format!("panel-{}", label)).label(label).gap(b.gap);
        for c in b.children {
            p.children.push(c);
        }
        self.children.push(Box::new(p));
    }
    /// Espace entre les composants de la section, en pixels.
    pub fn gap(&mut self, g: f64) {
        self.gap = g;
    }
}

/// Conteneur à onglets : chaque `.tab(label, …)` devient un panneau.
pub struct Tabs {
    id: String,
    labels: Vec<String>,
    children: Vec<Box<dyn Component>>,
    selected: usize,
}

impl Tabs {
    /// Crée un bloc d'onglets, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), labels: Vec::new(), children: Vec::new(), selected: 0 }
    }
    /// Définit l'onglet sélectionné initialement (0-indexed).
    pub fn selected(mut self, idx: usize) -> Self { self.selected = idx; self }
    /// Ajoute un onglet (étiquette + contenu empilé en colonne).
    pub fn tab(mut self, label: impl Into<String>, task: impl FnOnce(&mut SectionBuilder)) -> Self {
        let mut b = SectionBuilder::default();
        task(&mut b);
        let mut col = Column::new(format!("{}-{}", self.id, self.labels.len())).gap(b.gap);
        for c in b.children {
            col.push(c);
        }
        self.labels.push(label.into());
        self.children.push(Box::new(col));
        self
    }
}

impl Component for Tabs {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "tabs" }
    fn props(&self) -> Value { json!({ "labels": self.labels, "selected": self.selected }) }
    fn children(&self) -> Vec<&dyn Component> { children_refs(&self.children) }
}

/// Accordéon : liste de sections repliables (balises `<details>` natives).
pub struct Accordion {
    id: String,
    labels: Vec<String>,
    children: Vec<Box<dyn Component>>,
    open: bool,
}

impl Accordion {
    /// Crée un accordéon, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), labels: Vec::new(), children: Vec::new(), open: false }
    }
    /// La première section est dépliée au chargement.
    pub fn open(mut self, open: bool) -> Self { self.open = open; self }
    /// Ajoute une section repliable (titre + contenu empilé en colonne).
    pub fn section(mut self, label: impl Into<String>, task: impl FnOnce(&mut SectionBuilder)) -> Self {
        let mut b = SectionBuilder::default();
        task(&mut b);
        let mut col = Column::new(format!("{}-{}", self.id, self.labels.len())).gap(b.gap);
        for c in b.children {
            col.push(c);
        }
        self.labels.push(label.into());
        self.children.push(Box::new(col));
        self
    }
}

impl Component for Accordion {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "accordion" }
    fn props(&self) -> Value { json!({ "labels": self.labels, "open": self.open }) }
    fn children(&self) -> Vec<&dyn Component> { children_refs(&self.children) }
}

// ---------------------------------------------------------------------------
// Widgets avancés (Phase 4 — catalogue Gradio-like)
// ---------------------------------------------------------------------------

/// Case à cocher (booléen).
#[derive(Clone, Debug)]
pub struct Checkbox {
    id: String,
    label: String,
    value: bool,
    interactive: bool,
}

impl Checkbox {
    /// Crée une case à cocher, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), label: String::new(), value: false, interactive: true }
    }
    /// Étiquette affichée à droite de la case.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// État initial (coché ou non).
    pub fn value(mut self, v: bool) -> Self { self.value = v; self }
    /// Autorise ou non le clic (case grisée).
    pub fn interactive(mut self, on: bool) -> Self { self.interactive = on; self }
}

impl Component for Checkbox {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "checkbox" }
    fn role(&self) -> Role { Role::Input }
    fn props(&self) -> Value {
        json!({ "label": self.label, "value": self.value, "interactive": self.interactive })
    }
}

/// Liste déroulante (choix unique ou multiple, saisie libre optionnelle).
#[derive(Clone, Debug)]
pub struct Dropdown {
    id: String,
    label: String,
    choices: Vec<(String, String)>,
    value: Value,
    multiple: bool,
    allow_custom: bool,
    interactive: bool,
}

impl Dropdown {
    /// Crée une liste déroulante, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            choices: Vec::new(),
            value: Value::Null,
            multiple: false,
            allow_custom: false,
            interactive: true,
        }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Choix proposés, sous forme `(libellé, valeur)`.
    pub fn choices(mut self, items: &[(&str, &str)]) -> Self {
        self.choices = items.iter().map(|(l, v)| (l.to_string(), v.to_string())).collect();
        self
    }
    /// Choix proposés avec libellé = valeur.
    pub fn choices_str(mut self, items: &[&str]) -> Self {
        self.choices = items.iter().map(|s| (s.to_string(), s.to_string())).collect();
        self
    }
    /// Alias pour choices_str.
    pub fn options(self, items: &[&str]) -> Self {
        self.choices_str(items)
    }
    /// Valeur sélectionnée initialement (choix unique).
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = json!(v.into()); self }
    /// Valeurs sélectionnées initialement (choix multiple).
    pub fn value_list(mut self, v: &[&str]) -> Self {
        self.value = json!(v);
        self
    }
    /// Sélection multiple (liste).
    pub fn multiple(mut self, on: bool) -> Self { self.multiple = on; self }
    /// Autorise la saisie d'une valeur hors liste.
    pub fn allow_custom(mut self, on: bool) -> Self { self.allow_custom = on; self }
    /// Autorise ou non l'interaction.
    pub fn interactive(mut self, on: bool) -> Self { self.interactive = on; self }
}

impl Component for Dropdown {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "dropdown" }
    fn role(&self) -> Role { Role::Input }
    fn props(&self) -> Value {
        let choices: Vec<Value> = self
            .choices
            .iter()
            .map(|(l, v)| json!({ "label": l, "value": v }))
            .collect();
        json!({
            "label": self.label, "choices": choices, "value": self.value,
            "multiple": self.multiple, "allow_custom": self.allow_custom, "interactive": self.interactive
        })
    }
}

/// Sélecteur de date (au format ISO `YYYY-MM-DD`).
#[derive(Clone, Debug)]
pub struct DatePicker {
    id: String,
    label: String,
    value: String,
    min: Option<String>,
    max: Option<String>,
    interactive: bool,
}

impl DatePicker {
    /// Crée un sélecteur de date, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), label: String::new(), value: String::new(), min: None, max: None, interactive: true }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Date initiale (ISO `YYYY-MM-DD`).
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = v.into(); self }
    /// Borne minimale (ISO).
    pub fn min(mut self, m: impl Into<String>) -> Self { self.min = Some(m.into()); self }
    /// Borne maximale (ISO).
    pub fn max(mut self, m: impl Into<String>) -> Self { self.max = Some(m.into()); self }
    /// Autorise ou non l'interaction.
    pub fn interactive(mut self, on: bool) -> Self { self.interactive = on; self }
}

impl Component for DatePicker {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "date" }
    fn role(&self) -> Role { Role::Input }
    fn props(&self) -> Value {
        json!({ "label": self.label, "value": self.value, "min": self.min, "max": self.max, "interactive": self.interactive })
    }
}

/// Sélecteur d'heure (`HH:MM`).
#[derive(Clone, Debug)]
pub struct TimePicker {
    id: String,
    label: String,
    value: String,
    interactive: bool,
}

impl TimePicker {
    /// Crée un sélecteur d'heure, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), label: String::new(), value: String::new(), interactive: true }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Heure initiale (`HH:MM`).
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = v.into(); self }
    /// Autorise ou non l'interaction.
    pub fn interactive(mut self, on: bool) -> Self { self.interactive = on; self }
}

impl Component for TimePicker {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "time" }
    fn role(&self) -> Role { Role::Input }
    fn props(&self) -> Value {
        json!({ "label": self.label, "value": self.value, "interactive": self.interactive })
    }
}

/// Table de données éditable (équivalent `gr.Dataframe`).
///
/// La valeur transportée est un objet `{ "headers": [..], "data": [[..]] }`,
/// récupérable côté serveur via `ctx.get::<serde_json::Value>` (ou
/// `ctx.get::<DataframeValue>` personnalisé).
#[derive(Clone, Debug)]
pub struct Dataframe {
    id: String,
    label: String,
    headers: Vec<String>,
    value: Value,
    interactive: bool,
    addable: bool,
    sortable: bool,
}

impl Dataframe {
    /// Crée un tableau, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), label: String::new(), headers: Vec::new(), value: json!([]), interactive: true, addable: true, sortable: true }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Noms de colonnes (sinon la première ligne des données sert d'entête).
    pub fn headers(mut self, items: &[&str]) -> Self {
        self.headers = items.iter().map(|s| s.to_string()).collect();
        self
    }
    /// Données initiales — rangées de rangées (sérialisables).
    pub fn data<T: serde::Serialize>(mut self, v: &T) -> Self {
        self.value = serde_json::to_value(v).unwrap_or(json!([]));
        self
    }
    /// Autorise l'édition des cellules.
    pub fn interactive(mut self, on: bool) -> Self { self.interactive = on; self }
    /// Affiche les boutons d'ajout/suppression de rangées.
    pub fn addable(mut self, on: bool) -> Self { self.addable = on; self }
    /// Autorise le **tri par colonne** (clic sur l'entête, asc/desc).
    pub fn sortable(mut self, on: bool) -> Self { self.sortable = on; self }
}

impl Component for Dataframe {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "dataframe" }
    fn role(&self) -> Role { Role::Input }
    fn props(&self) -> Value {
        json!({ "label": self.label, "headers": self.headers, "value": self.value, "interactive": self.interactive, "addable": self.addable, "sortable": self.sortable })
    }
}

/// Graphique vectoriel (rendu **SVG** côté client, sans dépendance).
///
/// Les données sont des séries JSON poussées avec `ctx.set(id, json)` :
/// `{ "labels": [..], "series": [{ "name", "data": [..] }] }` (line/bar) ou
/// `{ "series": [{ "name", "points": [[x, y], …] }] }` (scatter).
#[derive(Clone, Debug)]
pub struct Plot {
    id: String,
    label: String,
    variant: &'static str,
    title: Option<String>,
    xlabel: Option<String>,
    ylabel: Option<String>,
    colors: Vec<String>,
    width: u32,
    height: u32,
    value: Value,
}

impl Plot {
    /// Crée un graphique, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            variant: "line",
            title: None,
            xlabel: None,
            ylabel: None,
            colors: vec!["#6366f1".into(), "#8b5cf6".into(), "#16a34a".into()],
            width: 480,
            height: 280,
            value: json!({ "labels": [], "series": [] }),
        }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Type de graphique : `"line"`, `"bar"` ou `"scatter"`.
    pub fn variant(mut self, v: &'static str) -> Self { self.variant = v; self }
    /// Titre affiché au-dessus du tracé.
    pub fn title(mut self, t: impl Into<String>) -> Self { self.title = Some(t.into()); self }
    /// Libellé de l'axe X.
    pub fn xlabel(mut self, t: impl Into<String>) -> Self { self.xlabel = Some(t.into()); self }
    /// Libellé de l'axe Y.
    pub fn ylabel(mut self, t: impl Into<String>) -> Self { self.ylabel = Some(t.into()); self }
    /// Couleurs de série (au moins une).
    pub fn colors(mut self, c: &[&str]) -> Self { self.colors = c.iter().map(|s| s.to_string()).collect(); self }
    /// Taille du canevas en pixels.
    pub fn size(mut self, w: u32, h: u32) -> Self { self.width = w; self.height = h; self }
    /// Données initiales (séries).
    pub fn data<T: serde::Serialize>(mut self, v: &T) -> Self {
        self.value = serde_json::to_value(v).unwrap_or_else(|_| json!({ "labels": [], "series": [] }));
        self
    }
}

impl Component for Plot {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "plot" }
    fn role(&self) -> Role { Role::Output }
    fn props(&self) -> Value {
        json!({
            "label": self.label, "variant": self.variant, "title": self.title,
            "xlabel": self.xlabel, "ylabel": self.ylabel, "colors": self.colors,
            "width": self.width, "height": self.height, "value": self.value
        })
    }
}

/// Galerie d'images en grille (affichage et/ou upload multiple).
#[derive(Clone, Debug)]
pub struct Gallery {
    id: String,
    label: String,
    value: Vec<String>,
    columns: usize,
    interactive: bool,
    upload: bool,
    out: bool,
}

impl Gallery {
    /// Crée une galerie, avec son identifiant. Entrée (upload) par défaut.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), label: String::new(), value: Vec::new(), columns: 3, interactive: true, upload: true, out: false }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Images initiales (data URLs ou URLs).
    pub fn value(mut self, items: &[&str]) -> Self {
        self.value = items.iter().map(|s| s.to_string()).collect();
        self
    }
    /// Images initiales depuis une collection de Strings ou Data URLs.
    pub fn items(mut self, items: &[impl AsRef<str>]) -> Self {
        self.value = items.iter().map(|s| s.as_ref().to_string()).collect();
        self
    }
    /// Nombre de colonnes de la grille.
    pub fn columns(mut self, n: usize) -> Self { self.columns = n.max(1); self }
    /// Autorise la sélection/upload.
    pub fn interactive(mut self, on: bool) -> Self { self.interactive = on; self }
    /// Affiche le bouton d'ajout d'image.
    pub fn upload(mut self, on: bool) -> Self { self.upload = on; self }
    /// Déclare la galerie **sortie** (affichage seul).
    pub fn output(mut self) -> Self { self.out = true; self }
    /// Déclare la galerie **entrée** (upload — par défaut).
    pub fn input(mut self) -> Self { self.out = false; self }
}

impl Component for Gallery {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "gallery" }
    fn role(&self) -> Role { if self.out { Role::Output } else { Role::Input } }
    fn props(&self) -> Value {
        json!({ "label": self.label, "value": self.value, "columns": self.columns, "interactive": self.interactive, "upload": self.upload })
    }
}

/// Liste d'éléments réordonnables par **glissé-déposé**.
///
/// La valeur est un tableau (sérialisé) de l'ordre courant des éléments —
/// `ctx.get::<Vec<String>>` renvoie cet ordre.
#[derive(Clone, Debug)]
pub struct SortableList {
    id: String,
    label: String,
    items: Vec<(String, String)>,
    value: Vec<String>,
    interactive: bool,
}

impl SortableList {
    /// Crée une liste réordonnable, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), label: String::new(), items: Vec::new(), value: Vec::new(), interactive: true }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Éléments proposés, sous forme `(valeur, libellé)`.
    pub fn items(mut self, list: &[(&str, &str)]) -> Self {
        self.items = list.iter().map(|(v, l)| (v.to_string(), l.to_string())).collect();
        self.value = list.iter().map(|(v, _)| v.to_string()).collect();
        self
    }
    /// Éléments proposés (valeur = libellé).
    pub fn items_str(mut self, list: &[&str]) -> Self {
        self.items = list.iter().map(|s| (s.to_string(), s.to_string())).collect();
        self.value = list.iter().map(|s| s.to_string()).collect();
        self
    }
    /// Éléments individuels (liste hétérogène).
    pub fn add(mut self, value: impl Into<String>, label: impl Into<String>) -> Self {
        self.items.push((value.into(), label.into()));
        self.value = self.items.iter().map(|(v, _)| v.clone()).collect();
        self
    }
    /// Ordre initial (sous-ensemble autorisé).
    pub fn value(mut self, order: &[&str]) -> Self {
        self.value = order.iter().map(|s| s.to_string()).collect();
        self
    }
    /// Autorise ou non le réordonnancement.
    pub fn interactive(mut self, on: bool) -> Self { self.interactive = on; self }
}

impl Component for SortableList {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "list" }
    fn role(&self) -> Role { Role::Input }
    fn props(&self) -> Value {
        let items: Vec<Value> = self.items.iter().map(|(v, l)| json!({ "value": v, "label": l })).collect();
        json!({ "label": self.label, "items": items, "value": self.value, "interactive": self.interactive })
    }
}

/// Zone de code avec **colorisation syntaxique** (tokenizer maison).
#[derive(Clone, Debug)]
pub struct Code {
    id: String,
    label: String,
    language: Option<String>,
    value: String,
    interactive: bool,
    theme: String,
    line_numbers: bool,
    out: bool,
}

impl Code {
    /// Crée une zone de code, avec son identifiant. Éditable par défaut.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            language: None,
            value: String::new(),
            interactive: true,
            theme: "auto".to_string(),
            line_numbers: true,
            out: false,
        }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Langage pour la colorisation (`rust`, `python`, `javascript`, `json`, `markdown`).
    pub fn language(mut self, l: impl Into<String>) -> Self { self.language = Some(l.into()); self }
    /// Contenu initial.
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = v.into(); self }
    /// Autorise l'édition (textarea transparent sur la colorisation).
    pub fn interactive(mut self, on: bool) -> Self { self.interactive = on; self }
    /// Thème (`auto`, `light`, `dark`).
    pub fn theme(mut self, t: impl Into<String>) -> Self { self.theme = t.into(); self }
    /// Affiche les numéros de ligne.
    pub fn lines(mut self, on: bool) -> Self { self.line_numbers = on; self }
    /// Déclare la zone **sortie** (lecture seule).
    pub fn output(mut self) -> Self { self.out = true; self }
    /// Déclare la zone **entrée** (édition — par défaut).
    pub fn input(mut self) -> Self { self.out = false; self }
}

impl Component for Code {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "code" }
    fn role(&self) -> Role { if self.out { Role::Output } else { Role::Input } }
    fn props(&self) -> Value {
        json!({
            "label": self.label, "language": self.language, "value": self.value,
            "interactive": self.interactive, "theme": self.theme,
            "line_numbers": self.line_numbers
        })
    }
}

/// Explorateur de fichiers côté **serveur** : navigue une arborescence
/// réelle via `GET /api/explore`, bornée à `root`. La valeur est le chemin
/// relatif sélectionné.
#[derive(Clone, Debug)]
pub struct Explorer {
    id: String,
    label: String,
    root: String,
    pattern: Option<String>,
    value: String,
}

impl Explorer {
    /// Crée un explorateur, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self { id: id.into(), label: String::new(), root: ".".to_string(), pattern: None, value: String::new() }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Dossier racine servi par le navigateur (côté serveur).
    pub fn root(mut self, r: impl Into<String>) -> Self { self.root = r.into(); self }
    /// Filtrer les fichiers (globe simple, ex. `*.rs`).
    pub fn pattern(mut self, p: impl Into<String>) -> Self { self.pattern = Some(p.into()); self }
    /// Chemin sélectionné initialement (relatif à `root`).
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = v.into(); self }
}

impl Component for Explorer {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "explorer" }
    fn role(&self) -> Role { Role::Input }
    fn props(&self) -> Value {
        json!({ "label": self.label, "root": self.root, "pattern": self.pattern, "value": self.value })
    }
}

/// Message individuel pour le composant [`Chatbot`].
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct ChatMessage {
    /// Rôle de l'émetteur (`"user"`, `"assistant"`, `"system"`).
    pub role: String,
    /// Contenu textuel (supporte le Markdown).
    pub content: String,
}

impl ChatMessage {
    /// Crée un message utilisateur.
    pub fn user(content: impl Into<String>) -> Self {
        Self { role: "user".into(), content: content.into() }
    }
    /// Crée un message assistant (bot).
    pub fn assistant(content: impl Into<String>) -> Self {
        Self { role: "assistant".into(), content: content.into() }
    }
    /// Crée un message système.
    pub fn system(content: impl Into<String>) -> Self {
        Self { role: "system".into(), content: content.into() }
    }
}

/// Composant d'historique de conversation (Chatbot pour LLM / agents).
///
/// Affiche des bulles de messages stylisées (utilisateur à droite, assistant à gauche),
/// supporte le Markdown, les blocs de code et la mise à jour incrémentale temps réel.
#[derive(Clone, Debug)]
pub struct Chatbot {
    id: String,
    label: String,
    messages: Vec<ChatMessage>,
    height: Option<u32>,
    placeholder: String,
}

impl Chatbot {
    /// Crée un composant Chatbot avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            messages: Vec::new(),
            height: Some(420),
            placeholder: "Commencez la conversation...".into(),
        }
    }
    /// Étiquette affichée au-dessus de la fenêtre de chat.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Messages initiaux du chat.
    pub fn messages(mut self, m: Vec<ChatMessage>) -> Self { self.messages = m; self }
    /// Ajoute un message initial dans la conversation.
    pub fn message(mut self, role: impl Into<String>, content: impl Into<String>) -> Self {
        self.messages.push(ChatMessage { role: role.into(), content: content.into() });
        self
    }
    /// Hauteur en pixels du conteneur scrollable.
    pub fn height(mut self, h: u32) -> Self { self.height = Some(h); self }
    /// Texte indicatif quand l'historique est vide.
    pub fn placeholder(mut self, p: impl Into<String>) -> Self { self.placeholder = p.into(); self }
}

impl Component for Chatbot {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "chatbot" }
    fn role(&self) -> Role { Role::Output }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "value": self.messages,
            "height": self.height,
            "placeholder": self.placeholder
        })
    }
}

/// Carte d'indicateur de performance IA (Throughput, Latence, Mémoire, Précision).
#[derive(Clone, Debug)]
pub struct Metric {
    id: String,
    label: String,
    value: String,
    delta: Option<String>,
    delta_color: Option<String>,
    unit: Option<String>,
}

impl Metric {
    /// Crée une nouvelle carte de métrique.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            value: String::new(),
            delta: None,
            delta_color: None,
            unit: None,
        }
    }
    /// Titre / libellé de l'indicateur.
    pub fn label(mut self, l: impl Into<String>) -> Self { self.label = l.into(); self }
    /// Valeur principale affichée en grand (ex: "48.5", "120ms").
    pub fn value(mut self, v: impl Into<String>) -> Self { self.value = v.into(); self }
    /// Variation / delta (ex: "+12.4%", "-20ms").
    pub fn delta(mut self, d: impl Into<String>) -> Self { self.delta = Some(d.into()); self }
    /// Couleur sémantique du delta ("normal", "inverse", "off", ou code couleur).
    pub fn delta_color(mut self, c: impl Into<String>) -> Self { self.delta_color = Some(c.into()); self }
    /// Unité de mesure affichée à côté de la valeur (ex: "tok/s", "MB", "req/s").
    pub fn unit(mut self, u: impl Into<String>) -> Self { self.unit = Some(u.into()); self }
}

impl Component for Metric {
    fn id(&self) -> &str { &self.id }
    fn kind(&self) -> &'static str { "metric" }
    fn role(&self) -> Role { Role::Output }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "value": self.value,
            "delta": self.delta,
            "delta_color": self.delta_color,
            "unit": self.unit
        })
    }
}