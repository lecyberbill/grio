//! Conteneurs et composants de mise en page (Row, Column, Grid, Panel, Tabs, Accordion, Drawer).

use serde_json::{json, Value};

use super::{children_refs, Component, IntoBox, Role};
use crate::app::RowBuilder;

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
    pub fn gap(mut self, g: f64) -> Self {
        self.gap = g;
        self
    }
    /// Autorise ou interdit le retour à la ligne (`flex-wrap`).
    pub fn wrap(mut self, w: bool) -> Self {
        self.wrap = w;
        self
    }
    /// Alignement transversal (CSS `align-items` : `"start"`, `"center"`, `"end"`, `"stretch"`).
    pub fn align(mut self, a: impl Into<String>) -> Self {
        self.align = Some(a.into());
        self
    }
    /// Justification principale (CSS `justify-content` : `"start"`, `"center"`, `"end"`, `"space-between"`, `"space-around"`).
    pub fn justify(mut self, j: impl Into<String>) -> Self {
        self.justify = Some(j.into());
        self
    }
    /// Ajoute un composant dans la rangée.
    pub fn item(mut self, c: impl IntoBox) -> Self {
        self.children.push(c.into_box());
        self
    }
}

impl Component for Row {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "row"
    }
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
    fn children(&self) -> Vec<&dyn Component> {
        children_refs(&self.children)
    }
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
    pub fn gap(mut self, g: f64) -> Self {
        self.gap = g;
        self
    }
    /// Alignement transversal (CSS `align-items` : `"start"`, `"center"`, `"end"`, `"stretch"`).
    pub fn align(mut self, a: impl Into<String>) -> Self {
        self.align = Some(a.into());
        self
    }
    /// Justification principale (CSS `justify-content` : `"start"`, `"center"`, `"end"`, `"space-between"`).
    pub fn justify(mut self, j: impl Into<String>) -> Self {
        self.justify = Some(j.into());
        self
    }
    /// Ajoute un composant dans la colonne.
    pub fn item(mut self, c: impl IntoBox) -> Self {
        self.children.push(c.into_box());
        self
    }
    pub(crate) fn push(&mut self, c: Box<dyn Component>) {
        self.children.push(c);
    }
}

impl Component for Column {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "column"
    }
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
    fn children(&self) -> Vec<&dyn Component> {
        children_refs(&self.children)
    }
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
    pub fn gap(mut self, g: f64) -> Self {
        self.gap = g;
        self
    }
    /// Espacement horizontal en pixels.
    pub fn gap_x(mut self, gx: f64) -> Self {
        self.gap_x = Some(gx);
        self
    }
    /// Espacement vertical en pixels.
    pub fn gap_y(mut self, gy: f64) -> Self {
        self.gap_y = Some(gy);
        self
    }
    /// Ajoute un composant dans la grille.
    pub fn item(mut self, c: impl IntoBox) -> Self {
        self.children.push(c.into_box());
        self
    }
}

impl Component for Grid {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "grid"
    }
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
    fn children(&self) -> Vec<&dyn Component> {
        children_refs(&self.children)
    }
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
        Self {
            id: id.into(),
            label: String::new(),
            gap: 16.0,
            children: Vec::new(),
        }
    }
    /// Titre affiché dans l'entête de la carte.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Espace entre les enfants, en pixels.
    pub fn gap(mut self, g: f64) -> Self {
        self.gap = g;
        self
    }
    /// Ajoute un composant dans la carte.
    pub fn item(mut self, c: impl IntoBox) -> Self {
        self.children.push(c.into_box());
        self
    }
}

impl Component for Panel {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "panel"
    }
    fn props(&self) -> Value {
        json!({ "label": self.label, "gap": self.gap })
    }
    fn children(&self) -> Vec<&dyn Component> {
        children_refs(&self.children)
    }
}

/// Collecteur d'éléments pour une section d'onglet / accordéon.
pub struct SectionBuilder {
    pub(crate) children: Vec<Box<dyn Component>>,
    pub(crate) gap: f64,
}

impl Default for SectionBuilder {
    fn default() -> Self {
        Self {
            children: Vec::new(),
            gap: 14.0,
        }
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
        let mut g = Grid::new(format!("grid-{}", self.children.len()))
            .columns(columns)
            .gap(b.gap);
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
        let mut p = Panel::new(format!("panel-{}", label))
            .label(label)
            .gap(b.gap);
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
        Self {
            id: id.into(),
            labels: Vec::new(),
            children: Vec::new(),
            selected: 0,
        }
    }
    /// Définit l'onglet sélectionné initialement (0-indexed).
    pub fn selected(mut self, idx: usize) -> Self {
        self.selected = idx;
        self
    }
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
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "tabs"
    }
    fn props(&self) -> Value {
        json!({ "labels": self.labels, "selected": self.selected })
    }
    fn children(&self) -> Vec<&dyn Component> {
        children_refs(&self.children)
    }
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
        Self {
            id: id.into(),
            labels: Vec::new(),
            children: Vec::new(),
            open: false,
        }
    }
    /// La première section est dépliée au chargement.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }
    /// Ajoute une section repliable (titre + contenu empilé en colonne).
    pub fn section(
        mut self,
        label: impl Into<String>,
        task: impl FnOnce(&mut SectionBuilder),
    ) -> Self {
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
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "accordion"
    }
    fn props(&self) -> Value {
        json!({ "labels": self.labels, "open": self.open })
    }
    fn children(&self) -> Vec<&dyn Component> {
        children_refs(&self.children)
    }
}

/// Conteneur tiroir coulissant (horizontal : gauche/droite, ou vertical : bas/haut).
pub struct Drawer {
    pub(crate) id: String,
    pub(crate) title: String,
    pub(crate) placement: String,
    pub(crate) size: u32,
    pub(crate) open: bool,
    pub(crate) backdrop: bool,
    pub(crate) gap: f64,
    pub(crate) children: Vec<Box<dyn Component>>,
}

impl Drawer {
    /// Crée un tiroir coulissant avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            title: String::new(),
            placement: "right".into(),
            size: 380,
            open: false,
            backdrop: true,
            gap: 16.0,
            children: Vec::new(),
        }
    }

    /// Titre affiché dans l'entête du tiroir.
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = t.into();
        self
    }

    /// Position et direction du tiroir (`"left"`, `"right"`, `"bottom"`, `"top"`).
    pub fn placement(mut self, p: impl Into<String>) -> Self {
        self.placement = p.into();
        self
    }

    /// Taille du tiroir en pixels (largeur si left/right, hauteur si bottom/top).
    pub fn size(mut self, px: u32) -> Self {
        self.size = px;
        self
    }

    /// État initial ouvert/fermé.
    pub fn open(mut self, on: bool) -> Self {
        self.open = on;
        self
    }

    /// Affiche ou non le fond sombre translucide (backdrop).
    pub fn backdrop(mut self, on: bool) -> Self {
        self.backdrop = on;
        self
    }

    /// Espace entre les éléments à l'intérieur du tiroir en pixels.
    pub fn gap(mut self, g: f64) -> Self {
        self.gap = g;
        self
    }

    /// Ajoute un élément directement dans le tiroir.
    pub fn item(mut self, c: impl IntoBox) -> Self {
        self.children.push(c.into_box());
        self
    }

    /// Configure le contenu du tiroir via un builder de section.
    pub fn content(mut self, task: impl FnOnce(&mut SectionBuilder)) -> Self {
        let mut b = SectionBuilder::default();
        task(&mut b);
        for c in b.children {
            self.children.push(c);
        }
        self
    }
}

impl Component for Drawer {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "drawer"
    }
    fn role(&self) -> Role {
        Role::Output
    }
    fn props(&self) -> Value {
        json!({
            "title": self.title,
            "placement": self.placement,
            "size": self.size,
            "open": self.open,
            "backdrop": self.backdrop,
            "gap": self.gap,
        })
    }
    fn children(&self) -> Vec<&dyn Component> {
        children_refs(&self.children)
    }
}

/// **DynamicContainer** : Conteneur dynamique (slot runtime) dont les enfants
/// peuvent être insérés, remplacés ou vidés à chaud depuis le serveur (`Context`).
///
/// ```rust
/// # use grio::*;
/// DynamicContainer::new("slot_ticket_details")
///     .item(Text::new("placeholder").value("Sélectionnez un ticket pour afficher les détails."));
/// ```
pub struct DynamicContainer {
    id: String,
    children: Vec<Box<dyn Component>>,
    direction: String,
    gap: f32,
}

impl DynamicContainer {
    /// Crée un nouveau conteneur dynamique avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            children: Vec::new(),
            direction: "column".into(),
            gap: 12.0,
        }
    }

    /// Espacement entre les composants enfants injectés (en pixels).
    pub fn gap(mut self, g: f32) -> Self {
        self.gap = g;
        self
    }

    /// Direction d'empilement (`column` ou `row`).
    pub fn direction(mut self, d: impl Into<String>) -> Self {
        self.direction = d.into();
        self
    }

    /// Ajoute un composant initial par défaut.
    pub fn item(mut self, c: impl IntoBox) -> Self {
        self.children.push(c.into_box());
        self
    }

    /// Remplace ou configure les composants initiaux via une closure.
    pub fn content(mut self, task: impl FnOnce(&mut SectionBuilder)) -> Self {
        let mut b = SectionBuilder::default();
        task(&mut b);
        for c in b.children {
            self.children.push(c);
        }
        self
    }
}

impl Component for DynamicContainer {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "dynamic_container"
    }
    fn role(&self) -> Role {
        Role::Output
    }
    fn props(&self) -> Value {
        json!({
            "direction": self.direction,
            "gap": self.gap,
        })
    }
    fn children(&self) -> Vec<&dyn Component> {
        children_refs(&self.children)
    }
}

