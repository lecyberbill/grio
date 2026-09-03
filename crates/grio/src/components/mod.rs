//! Composants de l'UI : éléments modulaires exposés à l'utilisateur du moteur.
//!
//! Chaque composant est déclaré via son constructeur, configuré par chaînage
//! de méthodes (builder), puis inséré dans un conteneur ou à la racine de
//! l'application (`App::item`, `App::row`, …).
//!
//! Le rôle de chaque composant (`Role::Input` / `Role::Output`) est
//! déclaratif : il permet d'identifier automatiquement les entrées et les
//! sorties, donc de générer l'API REST (`/api/predict`, `/api/schema`).

use serde_json::{json, Value};

pub mod data;
pub mod forms;
pub mod layout;
pub mod media;
pub mod special;

pub use data::*;
pub use forms::*;
pub use layout::*;
pub use media::*;
pub use special::*;

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
    /// Largeur maximale en pixels (CSS `max-width`).
    pub max_width: Option<u32>,
    /// Hauteur maximale en pixels (CSS `max-height`).
    pub max_height: Option<u32>,
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
        if let Some(mw) = self.max_width {
            o.insert("max_width".into(), json!(mw));
        }
        if let Some(mh) = self.max_height {
            o.insert("max_height".into(), json!(mh));
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
        Self {
            inner,
            layout: Layout::default(),
        }
    }
    /// Largeur en pixels.
    pub fn width(mut self, w: u32) -> Self {
        self.layout.width = Some(w);
        self
    }
    /// Hauteur en pixels.
    pub fn height(mut self, h: u32) -> Self {
        self.layout.height = Some(h);
        self
    }
    /// Largeur maximale en pixels.
    pub fn max_width(mut self, mw: u32) -> Self {
        self.layout.max_width = Some(mw);
        self
    }
    /// Hauteur maximale en pixels.
    pub fn max_height(mut self, mh: u32) -> Self {
        self.layout.max_height = Some(mh);
        self
    }
    /// Proportion relative dans la ligne (comme `scale` de Gradio).
    pub fn scale(mut self, s: u32) -> Self {
        self.layout.scale = Some(s);
        self
    }
    /// Largeur minimale en pixels.
    pub fn min_width(mut self, w: u32) -> Self {
        self.layout.min_width = Some(w);
        self
    }
    /// Remplace la mise en page en un appel.
    pub fn set_layout(mut self, l: Layout) -> Self {
        self.layout = l;
        self
    }
    /// Restitue le composant d'origine.
    pub fn into_inner(self) -> C {
        self.inner
    }
    /// Réglage courant.
    pub fn layout(&self) -> Layout {
        self.layout
    }
}

impl<C: Component> Component for WithLayout<C> {
    fn id(&self) -> &str {
        self.inner.id()
    }
    fn kind(&self) -> &'static str {
        self.inner.kind()
    }
    fn props(&self) -> Value {
        self.inner.props()
    }
    fn role(&self) -> Role {
        self.inner.role()
    }
    fn children(&self) -> Vec<&dyn Component> {
        self.inner.children()
    }
    fn layout(&self) -> Layout {
        self.layout
    }
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

pub(crate) fn children_refs(items: &[Box<dyn Component>]) -> Vec<&dyn Component> {
    items.iter().map(|b| b.as_ref() as &dyn Component).collect()
}
