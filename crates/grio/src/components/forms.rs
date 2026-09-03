//! Composants de formulaires et de saisie utilisateur (Text, Slider, Checkbox, Dropdown, etc.).

use serde_json::{json, Value};

use super::{Component, Role};

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
        Self {
            id: id.into(),
            label: String::new(),
            value: String::new(),
            placeholder: None,
            lines: 1,
            interactive: true,
        }
    }
    /// Définit l'étiquette affichée au-dessus du champ.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Valeur initiale.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    /// Texte indicatif affiché quand le champ est vide.
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = Some(p.into());
        self
    }
    /// Nombre de lignes affichées (1 = input classique, >1 = textarea).
    pub fn lines(mut self, n: usize) -> Self {
        self.lines = n.max(1);
        self
    }
    /// Autorise ou non l'édition par l'utilisateur (champ grisé ; la valeur
    /// reste envoyée dans le snapshot d'entrées).
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
}

impl Component for Text {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "text"
    }
    fn role(&self) -> Role {
        Role::Input
    }
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
        Self {
            id: id.into(),
            label: String::new(),
            min: 0.0,
            max: 100.0,
            step: 1.0,
            value: 0.0,
            interactive: true,
        }
    }
    /// Étiquette affichée au-dessus du curseur.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Borne minimum (f64).
    pub fn min(mut self, v: f64) -> Self {
        self.min = v;
        self
    }
    /// Borne maximum (f64).
    pub fn max(mut self, v: f64) -> Self {
        self.max = v;
        self
    }
    /// Pas d'incrément (f64).
    pub fn step(mut self, v: f64) -> Self {
        self.step = v;
        self
    }
    /// Valeur initiale.
    pub fn value(mut self, v: f64) -> Self {
        self.value = v;
        self
    }
    /// Autorise ou non la manipulation par l'utilisateur (curseur grisé ;
    /// la valeur reste envoyée dans le snapshot d'entrées).
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
}

impl Component for Slider {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "slider"
    }
    fn role(&self) -> Role {
        Role::Input
    }
    fn props(&self) -> Value {
        json!({ "label": self.label, "min": self.min, "max": self.max, "step": self.step, "value": self.value, "interactive": self.interactive })
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
        Self {
            id: id.into(),
            label: String::new(),
            variant: "primary".to_string(),
            primary: false,
        }
    }
    /// Texte affiché sur le bouton.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Style secondaire (contour). Style principal par défaut.
    pub fn secondary(mut self) -> Self {
        self.variant = "secondary".to_string();
        self
    }
    /// Définit la variante de style (`primary`, `secondary`, etc.).
    pub fn variant(mut self, v: impl Into<String>) -> Self {
        self.variant = v.into();
        self
    }
    /// Marque le bouton comme déclencheur de soumission (`on_submit`).
    pub fn primary(mut self) -> Self {
        self.primary = true;
        self
    }
}

impl Component for Button {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "button"
    }
    fn props(&self) -> Value {
        json!({ "label": self.label, "variant": self.variant, "primary": self.primary })
    }
}

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
        Self {
            id: id.into(),
            label: String::new(),
            value: false,
            interactive: true,
        }
    }
    /// Étiquette affichée à droite de la case.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// État initial (coché ou non).
    pub fn value(mut self, v: bool) -> Self {
        self.value = v;
        self
    }
    /// Autorise ou non le clic (case grisée).
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
}

impl Component for Checkbox {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "checkbox"
    }
    fn role(&self) -> Role {
        Role::Input
    }
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
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Choix proposés, sous forme `(libellé, valeur)`.
    pub fn choices(mut self, items: &[(&str, &str)]) -> Self {
        self.choices = items
            .iter()
            .map(|(l, v)| (l.to_string(), v.to_string()))
            .collect();
        self
    }
    /// Choix proposés avec libellé = valeur.
    pub fn choices_str(mut self, items: &[&str]) -> Self {
        self.choices = items
            .iter()
            .map(|s| (s.to_string(), s.to_string()))
            .collect();
        self
    }
    /// Alias pour choices_str.
    pub fn options(self, items: &[&str]) -> Self {
        self.choices_str(items)
    }
    /// Valeur sélectionnée initialement (choix unique).
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = json!(v.into());
        self
    }
    /// Valeurs sélectionnées initialement (choix multiple).
    pub fn value_list(mut self, v: &[&str]) -> Self {
        self.value = json!(v);
        self
    }
    /// Sélection multiple (liste).
    pub fn multiple(mut self, on: bool) -> Self {
        self.multiple = on;
        self
    }
    /// Autorise la saisie d'une valeur hors liste.
    pub fn allow_custom(mut self, on: bool) -> Self {
        self.allow_custom = on;
        self
    }
    /// Autorise ou non l'interaction.
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
}

impl Component for Dropdown {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "dropdown"
    }
    fn role(&self) -> Role {
        Role::Input
    }
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
        Self {
            id: id.into(),
            label: String::new(),
            value: String::new(),
            min: None,
            max: None,
            interactive: true,
        }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Date initiale (ISO `YYYY-MM-DD`).
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    /// Borne minimale (ISO).
    pub fn min(mut self, m: impl Into<String>) -> Self {
        self.min = Some(m.into());
        self
    }
    /// Borne maximale (ISO).
    pub fn max(mut self, m: impl Into<String>) -> Self {
        self.max = Some(m.into());
        self
    }
    /// Autorise ou non l'interaction.
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
}

impl Component for DatePicker {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "date"
    }
    fn role(&self) -> Role {
        Role::Input
    }
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
        Self {
            id: id.into(),
            label: String::new(),
            value: String::new(),
            interactive: true,
        }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Heure initiale (`HH:MM`).
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    /// Autorise ou non l'interaction.
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
}

impl Component for TimePicker {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "time"
    }
    fn role(&self) -> Role {
        Role::Input
    }
    fn props(&self) -> Value {
        json!({ "label": self.label, "value": self.value, "interactive": self.interactive })
    }
}

/// Liste d'éléments réordonnables par **glissé-déposé**.
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
        Self {
            id: id.into(),
            label: String::new(),
            items: Vec::new(),
            value: Vec::new(),
            interactive: true,
        }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Éléments proposés, sous forme `(valeur, libellé)`.
    pub fn items(mut self, list: &[(&str, &str)]) -> Self {
        self.items = list
            .iter()
            .map(|(v, l)| (v.to_string(), l.to_string()))
            .collect();
        self.value = list.iter().map(|(v, _)| v.to_string()).collect();
        self
    }
    /// Éléments proposés (valeur = libellé).
    pub fn items_str(mut self, list: &[&str]) -> Self {
        self.items = list
            .iter()
            .map(|s| (s.to_string(), s.to_string()))
            .collect();
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
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
}

impl Component for SortableList {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "list"
    }
    fn role(&self) -> Role {
        Role::Input
    }
    fn props(&self) -> Value {
        let items: Vec<Value> = self
            .items
            .iter()
            .map(|(v, l)| json!({ "value": v, "label": l }))
            .collect();
        json!({ "label": self.label, "items": items, "value": self.value, "interactive": self.interactive })
    }
}

/// **Number** field (`gr.Number` equivalent): direct numeric entry with
/// min/max/step bounds and a ± stepper. Emits a `change` with the numeric
/// value (clamped and snapped to `step`) — read it back via
/// `ctx.get::<f64>`.
#[derive(Clone, Debug)]
pub struct Number {
    id: String,
    label: String,
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    unit: String,
    interactive: bool,
}

impl Number {
    /// Creates a number field with its identifier.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            value: 0.0,
            min: 0.0,
            max: 1.0e6,
            step: 1.0,
            unit: String::new(),
            interactive: true,
        }
    }
    /// Label displayed above the field.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Initial value.
    pub fn value(mut self, v: f64) -> Self {
        self.value = v;
        self
    }
    /// Lower bound (default: `0`).
    pub fn min(mut self, v: f64) -> Self {
        self.min = v;
        self
    }
    /// Upper bound (default: `1e6`).
    pub fn max(mut self, v: f64) -> Self {
        self.max = v;
        self
    }
    /// Stepper increment (default: `1`).
    pub fn step(mut self, v: f64) -> Self {
        self.step = v;
        self
    }
    /// Unit shown next to the value (e.g. `"€"`, `"ms"`).
    pub fn unit(mut self, u: impl Into<String>) -> Self {
        self.unit = u.into();
        self
    }
    /// Enables or disables editing (frozen if `false`).
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
}

impl Component for Number {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "number"
    }
    fn role(&self) -> Role {
        Role::Input
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label, "value": self.value, "min": self.min,
            "max": self.max, "step": self.step, "unit": self.unit,
            "interactive": self.interactive
        })
    }
}

/// **Radio** component (`gr.Radio` equivalent): mutually exclusive selection
/// from a predefined list of options.
#[derive(Clone, Debug)]
pub struct Radio {
    id: String,
    label: String,
    choices: Vec<String>,
    value: String,
    direction: String,
    style: String,
    interactive: bool,
}

impl Radio {
    /// Creates a radio group component with its identifier. Input by default.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            choices: Vec::new(),
            value: String::new(),
            direction: "horizontal".into(),
            style: "pills".into(),
            interactive: true,
        }
    }
    /// Label displayed above the options.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Sets the list of selectable options from string slices.
    pub fn choices(mut self, c: &[&str]) -> Self {
        self.choices = c.iter().map(|s| s.to_string()).collect();
        if self.value.is_empty() && !self.choices.is_empty() {
            self.value = self.choices[0].clone();
        }
        self
    }
    /// Sets the list of selectable options from owned `String`s.
    pub fn choices_str(mut self, c: Vec<String>) -> Self {
        if self.value.is_empty() && !c.is_empty() {
            self.value = c[0].clone();
        }
        self.choices = c;
        self
    }
    /// Initial selected value.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    /// Layout direction (`"horizontal"` or `"vertical"`). Default is `"horizontal"`.
    pub fn direction(mut self, d: impl Into<String>) -> Self {
        self.direction = d.into();
        self
    }
    /// Visual presentation (`"pills"` or `"radio"`). Default is `"pills"`.
    pub fn style(mut self, s: impl Into<String>) -> Self {
        self.style = s.into();
        self
    }
    /// Enables or disables user interaction.
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
}

impl Component for Radio {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "radio"
    }
    fn role(&self) -> Role {
        Role::Input
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "choices": self.choices,
            "value": self.value,
            "direction": self.direction,
            "style": self.style,
            "interactive": self.interactive
        })
    }
}

/// **SliderRange** component (`gr.RangeSlider` equivalent):
/// dual-thumb slider allowing selection of an interval `[min_val, max_val]`.
#[derive(Clone, Debug)]
pub struct SliderRange {
    id: String,
    label: String,
    min: f64,
    max: f64,
    step: f64,
    value: (f64, f64),
    unit: String,
    interactive: bool,
}

impl SliderRange {
    /// Creates a range slider component with default range `[0.0, 1.0]`. Input by default.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            min: 0.0,
            max: 1.0,
            step: 0.05,
            value: (0.2, 0.8),
            unit: String::new(),
            interactive: true,
        }
    }
    /// Label displayed above the range slider.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Lower bound.
    pub fn min(mut self, m: f64) -> Self {
        self.min = m;
        self
    }
    /// Upper bound.
    pub fn max(mut self, m: f64) -> Self {
        self.max = m;
        self
    }
    /// Step resolution.
    pub fn step(mut self, s: f64) -> Self {
        self.step = s;
        self
    }
    /// Initial range `(low, high)`.
    pub fn value(mut self, low: f64, high: f64) -> Self {
        self.value = (low, high);
        self
    }
    /// Optional unit symbol displayed alongside values (e.g. `"%"`, `"px"`, `"s"`).
    pub fn unit(mut self, u: impl Into<String>) -> Self {
        self.unit = u.into();
        self
    }
    /// Enables or disables user interaction.
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
}

impl Component for SliderRange {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "sliderrange"
    }
    fn role(&self) -> Role {
        Role::Input
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "min": self.min,
            "max": self.max,
            "step": self.step,
            "value": [self.value.0, self.value.1],
            "unit": self.unit,
            "interactive": self.interactive
        })
    }
}

/// **ColorPicker** component (`gr.ColorPicker` equivalent): interactive color selector.
#[derive(Clone, Debug)]
pub struct ColorPicker {
    id: String,
    label: String,
    value: String,
    presets: Vec<String>,
    interactive: bool,
}

impl ColorPicker {
    /// Creates a color picker component with default color `"#6366f1"`. Input by default.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            value: "#6366f1".into(),
            presets: vec![
                "#6366f1".into(),
                "#8b5cf6".into(),
                "#ec4899".into(),
                "#ef4444".into(),
                "#f59e0b".into(),
                "#10b981".into(),
                "#06b6d4".into(),
                "#1e293b".into(),
                "#ffffff".into(),
            ],
            interactive: true,
        }
    }
    /// Label displayed above the color picker.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Initial hex color code.
    pub fn value(mut self, hex: impl Into<String>) -> Self {
        self.value = hex.into();
        self
    }
    /// Quick preset colors from string slices.
    pub fn presets(mut self, p: &[&str]) -> Self {
        self.presets = p.iter().map(|s| s.to_string()).collect();
        self
    }
    /// Quick preset colors from owned `String`s.
    pub fn presets_str(mut self, p: Vec<String>) -> Self {
        self.presets = p;
        self
    }
    /// Enables or disables user interaction.
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
}

impl Component for ColorPicker {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "colorpicker"
    }
    fn role(&self) -> Role {
        Role::Input
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "value": self.value,
            "presets": self.presets,
            "interactive": self.interactive
        })
    }
}

/// **RichText** component : Micro-éditeur Markdown avec barre d'outils
/// (Gras, Italique, Titres, Code, Liens, Listes à puces et numérotées).
///
/// Idéal pour la saisie de tickets d'incidents, descriptions riches et notes documentées.
///
/// ```rust
/// # use grio::*;
/// RichText::new("description")
///     .label("Détails de l'incident")
///     .placeholder("Décrivez le problème rencontré...")
///     .value("**Symptôme :** impossible de se connecter au VPN.")
///     .lines(8);
/// ```
#[derive(Clone, Debug)]
pub struct RichText {
    id: String,
    label: String,
    value: String,
    placeholder: String,
    lines: u32,
    interactive: bool,
    show_preview: bool,
}

impl RichText {
    /// Crée un nouvel éditeur de texte enrichi avec son identifiant. Rôle `Input` par défaut.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            value: String::new(),
            placeholder: String::new(),
            lines: 6,
            interactive: true,
            show_preview: true,
        }
    }
    /// Libellé affiché au-dessus de l'éditeur.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Contenu markdown initial.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    /// Texte indicatif affiché quand le champ est vide.
    pub fn placeholder(mut self, ph: impl Into<String>) -> Self {
        self.placeholder = ph.into();
        self
    }
    /// Nombre de lignes approximatif de la zone de saisie (hauteur).
    pub fn lines(mut self, n: u32) -> Self {
        self.lines = n.max(2);
        self
    }
    /// Active ou désactive l'interactivité.
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
    /// Active ou désactive l'onglet / bouton d'aperçu direct Markdown rendu.
    pub fn show_preview(mut self, on: bool) -> Self {
        self.show_preview = on;
        self
    }
}

impl Component for RichText {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "richtext"
    }
    fn role(&self) -> Role {
        Role::Input
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "value": self.value,
            "placeholder": self.placeholder,
            "lines": self.lines,
            "interactive": self.interactive,
            "show_preview": self.show_preview,
        })
    }
}

