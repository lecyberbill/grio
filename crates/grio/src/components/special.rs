//! Composants spécialisés, affichage textuel brut et widgets avancés IA (Chatbot, Map, Html, CodeDiff, Model3D, Timer, etc.).

use serde_json::{json, Value};

use super::{Component, Role};

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
        Self {
            id: id.into(),
            label: String::new(),
            value: String::new(),
        }
    }
    /// Étiquette affichée en tête de la carte.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Valeur initiale.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
}

impl Component for Output {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "output"
    }
    fn role(&self) -> Role {
        Role::Output
    }
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
        Self {
            id: id.into(),
            text: String::new(),
        }
    }
    /// Contenu Markdown initial.
    pub fn text(mut self, t: impl Into<String>) -> Self {
        self.text = t.into();
        self
    }
    /// Alias de [`Markdown::text`] pour cohérence avec les autres composants.
    pub fn value(mut self, t: impl Into<String>) -> Self {
        self.text = t.into();
        self
    }
}

impl Component for Markdown {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "markdown"
    }
    fn role(&self) -> Role {
        Role::Output
    }
    fn props(&self) -> Value {
        json!({ "text": self.text })
    }
}

/// Barre ou jauge de progression (cible de `ctx.progress`).
#[derive(Clone, Debug)]
pub struct Progress {
    id: String,
    label: String,
    variant: String,
    size: Option<usize>,
}

impl Progress {
    /// Crée un composant de progression, avec son identifiant.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            variant: "bar".to_string(),
            size: None,
        }
    }
    /// Étiquette affichée au-dessus ou à côté de la progression.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Définit la variante visuelle (`"bar"`, `"circle"`, `"pie"`).
    pub fn variant(mut self, v: impl Into<String>) -> Self {
        self.variant = v.into();
        self
    }
    /// Configure la variante barre horizontale (défaut).
    pub fn bar(mut self) -> Self {
        self.variant = "bar".to_string();
        self
    }
    /// Configure la variante anneau circulaire SVG.
    pub fn circle(mut self) -> Self {
        self.variant = "circle".to_string();
        self
    }
    /// Configure la variante camembert sectoriel (pie chart).
    pub fn pie(mut self) -> Self {
        self.variant = "pie".to_string();
        self
    }
    /// Dimension optionnelle en pixels (ex: diamètre du cercle ou camembert).
    pub fn size(mut self, s: usize) -> Self {
        self.size = Some(s);
        self
    }
}

impl Component for Progress {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "progress"
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "variant": self.variant,
            "size": self.size
        })
    }
}

/// Periodic **Timer** (`gr.Timer` equivalent).
#[derive(Clone, Debug)]
pub struct Timer {
    id: String,
    label: String,
    interval: f64,
    running: bool,
}

impl Timer {
    /// Creates a timer with its identifier. Output by default.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            interval: 1.0,
            running: true,
        }
    }
    /// Label displayed.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Tick interval in seconds (default: `1`).
    pub fn interval(mut self, s: f64) -> Self {
        self.interval = s.max(0.05);
        self
    }
    /// Starts the timer on mount (default: `true`).
    pub fn running(mut self, on: bool) -> Self {
        self.running = on;
        self
    }
}

impl Component for Timer {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "timer"
    }
    fn role(&self) -> Role {
        Role::Output
    }
    fn props(&self) -> Value {
        json!({ "label": self.label, "interval": self.interval, "running": self.running })
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
        Self {
            role: "user".into(),
            content: content.into(),
        }
    }
    /// Crée un message assistant (bot).
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: "assistant".into(),
            content: content.into(),
        }
    }
    /// Crée un message système.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: "system".into(),
            content: content.into(),
        }
    }
}

/// Composant d'historique de conversation (Chatbot pour LLM / agents).
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
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Messages initiaux du chat.
    pub fn messages(mut self, m: Vec<ChatMessage>) -> Self {
        self.messages = m;
        self
    }
    /// Ajoute un message initial dans la conversation.
    pub fn message(mut self, role: impl Into<String>, content: impl Into<String>) -> Self {
        self.messages.push(ChatMessage {
            role: role.into(),
            content: content.into(),
        });
        self
    }
    /// Hauteur en pixels du conteneur scrollable.
    pub fn height(mut self, h: u32) -> Self {
        self.height = Some(h);
        self
    }
    /// Texte indicatif quand l'historique est vide.
    pub fn placeholder(mut self, p: impl Into<String>) -> Self {
        self.placeholder = p.into();
        self
    }
}

impl Component for Chatbot {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "chatbot"
    }
    fn role(&self) -> Role {
        Role::Output
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "value": self.messages,
            "height": self.height,
            "placeholder": self.placeholder
        })
    }
}

/// **HighlightedText** segment: a piece of text with an optional category/entity label.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct TextSegment {
    /// Text chunk content.
    pub text: String,
    /// Category or entity tag (e.g. `"PER"`, `"ORG"`, `"LOC"`, `"POSITIVE"`).
    pub label: Option<String>,
}

/// **HighlightedText** component (`gr.HighlightedText` equivalent).
#[derive(Clone, Debug)]
pub struct HighlightedText {
    id: String,
    label: String,
    segments: Vec<TextSegment>,
    color_map: Vec<(String, String)>,
    show_legend: bool,
}

impl HighlightedText {
    /// Creates a highlighted text component with its identifier. Output by default.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            segments: Vec::new(),
            color_map: Vec::new(),
            show_legend: true,
        }
    }
    /// Label displayed above the highlighted text.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Appends a segment with an optional category tag.
    pub fn segment(mut self, text: impl Into<String>, label: Option<&str>) -> Self {
        self.segments.push(TextSegment {
            text: text.into(),
            label: label.map(|s| s.to_string()),
        });
        self
    }
    /// Sets segments from a slice of `(text, Option<label>)`.
    pub fn segments(mut self, segs: &[(&str, Option<&str>)]) -> Self {
        self.segments = segs
            .iter()
            .map(|(t, l)| TextSegment {
                text: t.to_string(),
                label: l.map(|s| s.to_string()),
            })
            .collect();
        self
    }
    /// Sets full JSON data: array of `[text, label]` or `Vec<TextSegment>`.
    pub fn data<T: serde::Serialize>(mut self, d: &T) -> Self {
        if let Ok(v) = serde_json::to_value(d) {
            if let Some(arr) = v.as_array() {
                self.segments = arr
                    .iter()
                    .filter_map(|item| {
                        if let Some(pair) = item.as_array() {
                            let text = pair
                                .first()
                                .and_then(|x| x.as_str())
                                .unwrap_or("")
                                .to_string();
                            let label = pair.get(1).and_then(|x| x.as_str()).map(|s| s.to_string());
                            Some(TextSegment { text, label })
                        } else {
                            serde_json::from_value::<TextSegment>(item.clone()).ok()
                        }
                    })
                    .collect();
            }
        }
        self
    }
    /// Assigns specific hex colors for labels, e.g. `&[("PER", "#10b981"), ("ORG", "#6366f1")]`.
    pub fn color_map(mut self, map: &[(&str, &str)]) -> Self {
        self.color_map = map
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        self
    }
    /// Shows or hides the color legend bar above the text.
    pub fn show_legend(mut self, on: bool) -> Self {
        self.show_legend = on;
        self
    }
}

impl Component for HighlightedText {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "highlightedtext"
    }
    fn role(&self) -> Role {
        Role::Output
    }
    fn props(&self) -> Value {
        let colors: serde_json::Map<String, Value> = self
            .color_map
            .iter()
            .map(|(k, v)| (k.clone(), Value::String(v.clone())))
            .collect();
        json!({
            "label": self.label,
            "segments": self.segments,
            "color_map": colors,
            "show_legend": self.show_legend
        })
    }
}

/// **CodeDiff** component: comparative diff viewer.
#[derive(Clone, Debug)]
pub struct CodeDiff {
    id: String,
    label: String,
    old_code: String,
    new_code: String,
    language: String,
    split_view: bool,
}

impl CodeDiff {
    /// Creates a code diff viewer with its identifier. Output by default.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            old_code: String::new(),
            new_code: String::new(),
            language: "rust".into(),
            split_view: false,
        }
    }
    /// Label displayed above the diff viewer.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Base original code string.
    pub fn old_code(mut self, s: impl Into<String>) -> Self {
        self.old_code = s.into();
        self
    }
    /// Updated / proposed new code string.
    pub fn new_code(mut self, s: impl Into<String>) -> Self {
        self.new_code = s.into();
        self
    }
    /// Programming language tag for styling.
    pub fn language(mut self, lang: impl Into<String>) -> Self {
        self.language = lang.into();
        self
    }
    /// Enables split view (side-by-side) instead of unified diff.
    pub fn split_view(mut self, on: bool) -> Self {
        self.split_view = on;
        self
    }
}

impl Component for CodeDiff {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "codediff"
    }
    fn role(&self) -> Role {
        Role::Output
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "old_code": self.old_code,
            "new_code": self.new_code,
            "language": self.language,
            "split_view": self.split_view
        })
    }
}

/// **Model3D** component (`gr.Model3D` equivalent).
#[derive(Clone, Debug)]
pub struct Model3D {
    id: String,
    label: String,
    value: String,
    clear_color: String,
    interactive: bool,
}

impl Model3D {
    /// Creates a 3D model viewer with its identifier. Output by default.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            value: String::new(),
            clear_color: "#1e293b".into(),
            interactive: true,
        }
    }
    /// Label displayed above the 3D viewer.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// 3D model data (raw OBJ string or data URL).
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    /// Background clear color hex code.
    pub fn clear_color(mut self, hex: impl Into<String>) -> Self {
        self.clear_color = hex.into();
        self
    }
    /// Enables mouse rotation and zoom controls.
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
}

impl Component for Model3D {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "model3d"
    }
    fn role(&self) -> Role {
        Role::Output
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "value": self.value,
            "clear_color": self.clear_color,
            "interactive": self.interactive
        })
    }
}

/// **Html** component (`gr.HTML` equivalent).
#[derive(Clone, Debug)]
pub struct Html {
    id: String,
    label: String,
    value: String,
    out: bool,
}

impl Html {
    /// Creates a custom HTML component with its identifier. Output by default.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            value: String::new(),
            out: true,
        }
    }
    /// Optional label displayed above the custom HTML container.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Raw HTML string content.
    pub fn value(mut self, h: impl Into<String>) -> Self {
        self.value = h.into();
        self
    }
    /// Declares the component as **Input** (value emitted on change).
    pub fn input(mut self) -> Self {
        self.out = false;
        self
    }
    /// Declares the component as **Output** (default, viewer).
    pub fn output(mut self) -> Self {
        self.out = true;
        self
    }
}

impl Component for Html {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "html"
    }
    fn role(&self) -> Role {
        if self.out {
            Role::Output
        } else {
            Role::Input
        }
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "value": self.value
        })
    }
}

/// **MapMarker**: Represents a geographical point on a [`Map`].
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MapMarker {
    /// Latitude (e.g. `48.8566`).
    pub lat: f64,
    /// Longitude (e.g. `2.3522`).
    pub lon: f64,
    /// Label or tooltip description displayed on hover/click.
    pub label: Option<String>,
    /// Marker pin hex color (default: `#6366f1`).
    pub color: Option<String>,
    /// Optional identifier for event routing.
    pub id: Option<String>,
}

/// **MapCircle**: Represents a geographical radius circle on a [`Map`].
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct MapCircle {
    /// Center latitude.
    pub lat: f64,
    /// Center longitude.
    pub lon: f64,
    /// Radius in meters.
    pub radius: f64,
    /// Stroke and fill color.
    pub color: Option<String>,
}

/// **Map** (OpenStreetMap) component: Interactive geographic map.
#[derive(Clone, Debug)]
pub struct Map {
    id: String,
    label: String,
    center_lat: f64,
    center_lon: f64,
    zoom: u8,
    markers: Vec<MapMarker>,
    circles: Vec<MapCircle>,
    height: u32,
    interactive: bool,
    out: bool,
}

impl Map {
    /// Creates a new Map component with its identifier. Centered on Paris by default.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            center_lat: 48.8566,
            center_lon: 2.3522,
            zoom: 12,
            markers: Vec::new(),
            circles: Vec::new(),
            height: 420,
            interactive: true,
            out: true,
        }
    }

    /// Label displayed above the map.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }

    /// Sets the initial map center coordinates [latitude, longitude].
    pub fn center(mut self, lat: f64, lon: f64) -> Self {
        self.center_lat = lat;
        self.center_lon = lon;
        self
    }

    /// Sets the initial zoom level (1 to 19, default: 12).
    pub fn zoom(mut self, z: u8) -> Self {
        self.zoom = z.clamp(1, 19);
        self
    }

    /// Adds a marker with coordinates, label, and optional accent color.
    pub fn marker(
        mut self,
        lat: f64,
        lon: f64,
        label: impl Into<String>,
        color: Option<&str>,
    ) -> Self {
        self.markers.push(MapMarker {
            lat,
            lon,
            label: Some(label.into()),
            color: color.map(|s| s.to_string()),
            id: None,
        });
        self
    }

    /// Adds a marker with an explicit ID for event identification.
    pub fn marker_with_id(
        mut self,
        id: impl Into<String>,
        lat: f64,
        lon: f64,
        label: impl Into<String>,
        color: Option<&str>,
    ) -> Self {
        self.markers.push(MapMarker {
            lat,
            lon,
            label: Some(label.into()),
            color: color.map(|s| s.to_string()),
            id: Some(id.into()),
        });
        self
    }

    /// Adds a radius circle (radius in meters).
    pub fn circle(mut self, lat: f64, lon: f64, radius: f64, color: Option<&str>) -> Self {
        self.circles.push(MapCircle {
            lat,
            lon,
            radius,
            color: color.map(|s| s.to_string()),
        });
        self
    }

    /// Sets the map display height in pixels (default: `420`).
    pub fn height(mut self, px: u32) -> Self {
        self.height = px;
        self
    }

    /// Enables or disables map interactivity.
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }

    /// Declares the component as **Input**.
    pub fn input(mut self) -> Self {
        self.out = false;
        self
    }
    /// Declares the component as **Output** (default, viewer).
    pub fn output(mut self) -> Self {
        self.out = true;
        self
    }
}

impl Component for Map {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "map"
    }
    fn role(&self) -> Role {
        if self.out {
            Role::Output
        } else {
            Role::Input
        }
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "center": [self.center_lat, self.center_lon],
            "zoom": self.zoom,
            "markers": self.markers,
            "circles": self.circles,
            "height": self.height,
            "interactive": self.interactive,
        })
    }
}

/// Socket de connexion (port d'entrée ou de sortie) sur un nœud de workflow [`NodeGraph`].
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct NodeSocket {
    /// Identifiant du socket sur le nœud.
    pub id: String,
    /// Libellé affiché à côté du point de connexion.
    pub label: String,
    /// Type de données attendu ou émis (ex. `text`, `image`, `json`, `model`).
    pub data_type: String,
}

/// Nœud d'un workflow visuel [`NodeGraph`].
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GraphNode {
    /// Identifiant unique du nœud.
    pub id: String,
    /// Titre affiché sur le nœud.
    pub title: String,
    /// Catégorie ou type de brique (ex. `input`, `llm`, `vision`, `tool`, `output`).
    pub category: String,
    /// Coordonnée X sur la grille infinie (en pixels).
    pub x: f64,
    /// Coordonnée Y sur la grille infinie (en pixels).
    pub y: f64,
    /// Sockets d'entrée.
    pub inputs: Vec<NodeSocket>,
    /// Sockets de sortie.
    pub outputs: Vec<NodeSocket>,
    /// État d'exécution (`idle`, `running`, `success`, `error`).
    pub status: String,
    /// Données ou paramètres spécifiques au nœud.
    pub data: Value,
}

impl GraphNode {
    /// Crée un nouveau nœud avec son identifiant, son titre et sa catégorie.
    pub fn new(
        id: impl Into<String>,
        title: impl Into<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            category: category.into(),
            x: 50.0,
            y: 50.0,
            inputs: Vec::new(),
            outputs: Vec::new(),
            status: "idle".into(),
            data: Value::Null,
        }
    }

    /// Position sur la grille `(x, y)`.
    pub fn pos(mut self, x: f64, y: f64) -> Self {
        self.x = x;
        self.y = y;
        self
    }

    /// Ajoute un socket d'entrée.
    pub fn input(mut self, id: impl Into<String>, data_type: impl Into<String>) -> Self {
        let sid = id.into();
        self.inputs.push(NodeSocket {
            label: sid.clone(),
            id: sid,
            data_type: data_type.into(),
        });
        self
    }

    /// Ajoute un socket de sortie.
    pub fn output(mut self, id: impl Into<String>, data_type: impl Into<String>) -> Self {
        let sid = id.into();
        self.outputs.push(NodeSocket {
            label: sid.clone(),
            id: sid,
            data_type: data_type.into(),
        });
        self
    }

    /// Définit l'état visuel du nœud (`idle`, `running`, `success`, `error`).
    pub fn status(mut self, s: impl Into<String>) -> Self {
        self.status = s.into();
        self
    }

    /// Paramètres de données arbitraires sérialisables.
    pub fn data<T: serde::Serialize>(mut self, d: &T) -> Self {
        self.data = serde_json::to_value(d).unwrap_or(Value::Null);
        self
    }
}

/// Connexion directionnelle entre deux sockets de nœuds dans [`NodeGraph`].
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct GraphEdge {
    /// Nœud source émetteur.
    pub from_node: String,
    /// Socket de sortie source.
    pub from_socket: String,
    /// Nœud cible récepteur.
    pub to_node: String,
    /// Socket d'entrée cible.
    pub to_socket: String,
}

/// **NodeGraph** : Éditeur de workflow de graphe de flux de nœuds (DAG / ComfyUI-style)
/// avec câblage par courbes de Bézier, déplacement sur grille infinie et exécution réactive.
///
/// ```rust
/// # use grio::*;
/// NodeGraph::new("pipeline")
///     .label("Orchestrateur IA")
///     .node(GraphNode::new("n1", "Input Prompt", "input").output("text", "Text").pos(40.0, 60.0))
///     .node(GraphNode::new("n2", "LLM Mistral", "llm").input("prompt", "Text").output("res", "Text").pos(280.0, 60.0))
///     .node(GraphNode::new("n3", "Format Output", "output").input("data", "Text").pos(520.0, 60.0))
///     .edge("n1", "text", "n2", "prompt")
///     .edge("n2", "res", "n3", "data");
/// ```
#[derive(Clone, Debug)]
pub struct NodeGraph {
    id: String,
    label: String,
    nodes: Vec<GraphNode>,
    edges: Vec<GraphEdge>,
    height: u32,
    interactive: bool,
    out: bool,
}

impl NodeGraph {
    /// Crée un nouvel éditeur de workflow de graphe de nœuds. Rôle `Input` par défaut.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            nodes: Vec::new(),
            edges: Vec::new(),
            height: 480,
            interactive: true,
            out: false,
        }
    }

    /// Libellé affiché au-dessus de l'éditeur de workflow.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }

    /// Ajoute un nœud dans le graphe.
    pub fn node(mut self, n: GraphNode) -> Self {
        self.nodes.push(n);
        self
    }

    /// Définit la liste complète des nœuds.
    pub fn nodes(mut self, nodes: Vec<GraphNode>) -> Self {
        self.nodes = nodes;
        self
    }

    /// Ajoute une arête de connexion entre deux nœuds et leurs sockets respectifs.
    pub fn edge(
        mut self,
        from_node: impl Into<String>,
        from_socket: impl Into<String>,
        to_node: impl Into<String>,
        to_socket: impl Into<String>,
    ) -> Self {
        self.edges.push(GraphEdge {
            from_node: from_node.into(),
            from_socket: from_socket.into(),
            to_node: to_node.into(),
            to_socket: to_socket.into(),
        });
        self
    }

    /// Définit la liste complète des arêtes.
    pub fn edges(mut self, edges: Vec<GraphEdge>) -> Self {
        self.edges = edges;
        self
    }

    /// Hauteur en pixels du canevas de graphe (par défaut `480`).
    pub fn height(mut self, h: u32) -> Self {
        self.height = h;
        self
    }

    /// Active ou désactive l'interactivité.
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }

    /// Déclare le composant en **Sortie** (visualiseur de workflow en lecture seule).
    pub fn output(mut self) -> Self {
        self.out = true;
        self
    }

    /// Déclare le composant en **Entrée** (éditeur interactif — par défaut).
    pub fn input(mut self) -> Self {
        self.out = false;
        self
    }
}

impl Component for NodeGraph {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "nodegraph"
    }
    fn role(&self) -> Role {
        if self.out {
            Role::Output
        } else {
            Role::Input
        }
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "nodes": self.nodes,
            "edges": self.edges,
            "height": self.height,
            "interactive": self.interactive,
        })
    }
}
