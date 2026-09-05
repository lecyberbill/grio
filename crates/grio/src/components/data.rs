//! Composants de visualisation de données, métriques, code et fichiers (Dataframe, Plot, Metric, Json, etc.).

use serde_json::{json, Value};

use super::{Component, Role};

/// Table de données éditable (équivalent `gr.Dataframe`).
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
        Self {
            id: id.into(),
            label: String::new(),
            headers: Vec::new(),
            value: json!([]),
            interactive: true,
            addable: true,
            sortable: true,
        }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
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
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
    /// Affiche les boutons d'ajout/suppression de rangées.
    pub fn addable(mut self, on: bool) -> Self {
        self.addable = on;
        self
    }
    /// Autorise le **tri par colonne** (clic sur l'entête, asc/desc).
    pub fn sortable(mut self, on: bool) -> Self {
        self.sortable = on;
        self
    }
}

impl Component for Dataframe {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "dataframe"
    }
    fn role(&self) -> Role {
        Role::Input
    }
    fn props(&self) -> Value {
        json!({ "label": self.label, "headers": self.headers, "value": self.value, "interactive": self.interactive, "addable": self.addable, "sortable": self.sortable })
    }
}

/// Graphique vectoriel (rendu **SVG** côté client, sans dépendance).
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
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Type de graphique : `"line"`, `"bar"` ou `"scatter"`.
    pub fn variant(mut self, v: &'static str) -> Self {
        self.variant = v;
        self
    }
    /// Titre affiché au-dessus du tracé.
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    /// Libellé de l'axe X.
    pub fn xlabel(mut self, t: impl Into<String>) -> Self {
        self.xlabel = Some(t.into());
        self
    }
    /// Libellé de l'axe Y.
    pub fn ylabel(mut self, t: impl Into<String>) -> Self {
        self.ylabel = Some(t.into());
        self
    }
    /// Couleurs de série (au moins une).
    pub fn colors(mut self, c: &[&str]) -> Self {
        self.colors = c.iter().map(|s| s.to_string()).collect();
        self
    }
    /// Taille du canevas en pixels.
    pub fn size(mut self, w: u32, h: u32) -> Self {
        self.width = w;
        self.height = h;
        self
    }
    /// Données initiales (séries).
    pub fn data<T: serde::Serialize>(mut self, v: &T) -> Self {
        self.value =
            serde_json::to_value(v).unwrap_or_else(|_| json!({ "labels": [], "series": [] }));
        self
    }
}

impl Component for Plot {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "plot"
    }
    fn role(&self) -> Role {
        Role::Output
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label, "variant": self.variant, "title": self.title,
            "xlabel": self.xlabel, "ylabel": self.ylabel, "colors": self.colors,
            "width": self.width, "height": self.height, "value": self.value
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
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Valeur principale affichée en grand (ex: "48.5", "120ms").
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    /// Variation / delta (ex: "+12.4%", "-20ms").
    pub fn delta(mut self, d: impl Into<String>) -> Self {
        self.delta = Some(d.into());
        self
    }
    /// Couleur sémantique du delta ("normal", "inverse", "off", ou code couleur).
    pub fn delta_color(mut self, c: impl Into<String>) -> Self {
        self.delta_color = Some(c.into());
        self
    }
    /// Unité de mesure affichée à côté de la valeur (ex: "tok/s", "MB", "req/s").
    pub fn unit(mut self, u: impl Into<String>) -> Self {
        self.unit = Some(u.into());
        self
    }
}

impl Component for Metric {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "metric"
    }
    fn role(&self) -> Role {
        Role::Output
    }
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

/// Result **Label** (`gr.Label` equivalent): a prominent value with a
/// semantic color. Update it through `ctx.set("id", "value")`.
#[derive(Clone, Debug)]
pub struct Label {
    id: String,
    label: String,
    value: String,
    variant: String,
    size: u32,
}

impl Label {
    /// Creates a label with its identifier. Output by default.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            value: String::new(),
            variant: "normal".into(),
            size: 26,
        }
    }
    /// Title shown above the value.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Initial value.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    /// Semantic color: `normal`, `success`, `warning`, `danger` or `off`.
    pub fn variant(mut self, v: impl Into<String>) -> Self {
        self.variant = v.into();
        self
    }
    /// Font size of the value in pixels (default: `26`).
    pub fn size(mut self, px: u32) -> Self {
        self.size = px;
        self
    }
}

impl Component for Label {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "label"
    }
    fn role(&self) -> Role {
        Role::Output
    }
    fn props(&self) -> Value {
        json!({ "label": self.label, "value": self.value, "variant": self.variant, "size": self.size })
    }
}

/// **JSON** viewer/editor: the server reads the parsed object via
/// `ctx.get::<serde_json::Value>`.
#[derive(Clone, Debug)]
pub struct Json {
    id: String,
    label: String,
    value: Value,
    interactive: bool,
    out: bool,
}

impl Json {
    /// Creates a JSON component with its identifier. Editor by default.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            value: Value::Null,
            interactive: true,
            out: false,
        }
    }
    /// Label displayed above.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Initial JSON value (object, array or scalar).
    pub fn value(mut self, v: Value) -> Self {
        self.value = v;
        self
    }
    /// Initial value from a JSON string (ignored when not parseable).
    pub fn value_str(mut self, s: &str) -> Self {
        if let Ok(v) = serde_json::from_str(s) {
            self.value = v;
        }
        self
    }
    /// Enables or disables editing (read-only if `false`).
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
    /// Declares the component **output** (plain viewer).
    pub fn output(mut self) -> Self {
        self.out = true;
        self
    }
    /// Declares the component **input** (editor — default).
    pub fn input(mut self) -> Self {
        self.out = false;
        self
    }
}

impl Component for Json {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "json"
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
            "label": self.label, "value": self.value, "interactive": self.interactive
        })
    }
}

/// **File** component (`gr.File` equivalent): multi-file upload.
#[derive(Clone, Debug)]
pub struct File {
    id: String,
    label: String,
    multiple: bool,
    types: Vec<String>,
    max_size: u64,
    interactive: bool,
}

impl File {
    /// Creates a file component with its identifier. Input by default.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            multiple: true,
            types: Vec::new(),
            max_size: 0,
            interactive: true,
        }
    }
    /// Label displayed above.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Allows multi-file selection (default: `true`).
    pub fn multiple(mut self, on: bool) -> Self {
        self.multiple = on;
        self
    }
    /// Accepted MIME types (default: any). E.g. `&["image/*", "application/pdf"]`.
    pub fn types(mut self, t: &[&str]) -> Self {
        self.types = t.iter().map(|s| s.to_string()).collect();
        self
    }
    /// Maximum size per file in bytes (`0` = unlimited, default).
    pub fn max_size(mut self, n: u64) -> Self {
        self.max_size = n;
        self
    }
    /// Enables or disables upload (read-only list if `false`).
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
}

impl Component for File {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "file"
    }
    fn role(&self) -> Role {
        Role::Input
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label, "multiple": self.multiple, "types": self.types,
            "max_size": self.max_size, "interactive": self.interactive
        })
    }
}

/// **Download** button (`gr.DownloadButton` equivalent).
#[derive(Clone, Debug)]
pub struct DownloadButton {
    id: String,
    label: String,
    filename: String,
    value: String,
}

impl DownloadButton {
    /// Creates a download button with its identifier. Output.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            filename: "download.bin".into(),
            value: String::new(),
        }
    }
    /// Button label.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Suggested download filename (default: `download.bin`).
    pub fn filename(mut self, n: impl Into<String>) -> Self {
        self.filename = n.into();
        self
    }
    /// Initial content as a data URL (or bare base64 → normalized at render).
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
}

impl Component for DownloadButton {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "download"
    }
    fn role(&self) -> Role {
        Role::Output
    }
    fn props(&self) -> Value {
        json!({ "label": self.label, "filename": self.filename, "value": self.value })
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
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Langage pour la colorisation (`rust`, `python`, `javascript`, `json`, `markdown`).
    pub fn language(mut self, l: impl Into<String>) -> Self {
        self.language = Some(l.into());
        self
    }
    /// Contenu initial.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    /// Autorise l'édition (textarea transparent sur la colorisation).
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
    /// Thème (`auto`, `light`, `dark`).
    pub fn theme(mut self, t: impl Into<String>) -> Self {
        self.theme = t.into();
        self
    }
    /// Affiche les numéros de ligne.
    pub fn lines(mut self, on: bool) -> Self {
        self.line_numbers = on;
        self
    }
    /// Déclare la zone **sortie** (lecture seule).
    pub fn output(mut self) -> Self {
        self.out = true;
        self
    }
    /// Déclare la zone **entrée** (édition — par défaut).
    pub fn input(mut self) -> Self {
        self.out = false;
        self
    }
}

impl Component for Code {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "code"
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
            "label": self.label, "language": self.language, "value": self.value,
            "interactive": self.interactive, "theme": self.theme,
            "line_numbers": self.line_numbers
        })
    }
}

/// Explorateur de fichiers côté **serveur** : navigue une arborescence
/// réelle via `GET /api/explore`, bornée à `root`.
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
        Self {
            id: id.into(),
            label: String::new(),
            root: ".".to_string(),
            pattern: None,
            value: String::new(),
        }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Dossier racine servi par le navigateur (côté serveur).
    pub fn root(mut self, r: impl Into<String>) -> Self {
        self.root = r.into();
        self
    }
    /// Filtrer les fichiers (globe simple, ex. `*.rs`).
    pub fn pattern(mut self, p: impl Into<String>) -> Self {
        self.pattern = Some(p.into());
        self
    }
    /// Chemin sélectionné initialement (relatif à `root`).
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
}

impl Component for Explorer {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "explorer"
    }
    fn role(&self) -> Role {
        Role::Input
    }
    fn props(&self) -> Value {
        json!({ "label": self.label, "root": self.root, "pattern": self.pattern, "value": self.value })
    }
}

/// Type de colonne supporté par le `DataEditor`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ColumnType {
    /// Texte libre standard.
    Text,
    /// Nombre numérique (saisie contrôlée / formatée).
    Number,
    /// Booléen avec case à cocher interactive (checkbox).
    Boolean,
    /// Menu déroulant avec liste de choix restreinte.
    Dropdown(Vec<String>),
}

impl ColumnType {
    /// Sérialisation JSON du type de colonne.
    pub fn to_json(&self) -> Value {
        match self {
            ColumnType::Text => json!({ "type": "text" }),
            ColumnType::Number => json!({ "type": "number" }),
            ColumnType::Boolean => json!({ "type": "boolean" }),
            ColumnType::Dropdown(choices) => json!({ "type": "dropdown", "choices": choices }),
        }
    }
}

/// Définition d'une colonne dans un `DataEditor`.
#[derive(Clone, Debug)]
pub struct ColumnDef {
    /// Identifiant de la colonne.
    pub id: String,
    /// Titre affiché dans l'en-tête du tableau.
    pub label: String,
    /// Type de données de la colonne.
    pub col_type: ColumnType,
    /// Indique si la colonne est modifiable par l'utilisateur.
    pub editable: bool,
    /// Largeur optionnelle en pixels.
    pub width: Option<u32>,
}

impl ColumnDef {
    /// Crée une nouvelle définition de colonne.
    pub fn new(id: impl Into<String>, label: impl Into<String>, col_type: ColumnType) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            col_type,
            editable: true,
            width: None,
        }
    }
    /// Définit si la colonne est éditable.
    pub fn editable(mut self, on: bool) -> Self {
        self.editable = on;
        self
    }
    /// Définit la largeur en pixels.
    pub fn width(mut self, w: u32) -> Self {
        self.width = Some(w);
        self
    }
    /// Sérialisation JSON.
    pub fn to_json(&self) -> Value {
        let mut obj = self.col_type.to_json();
        if let Value::Object(ref mut map) = obj {
            map.insert("id".into(), json!(self.id));
            map.insert("label".into(), json!(self.label));
            map.insert("editable".into(), json!(self.editable));
            if let Some(w) = self.width {
                map.insert("width".into(), json!(w));
            }
        }
        obj
    }
}

/// **DataEditor** : Grille de données interactive avancée avec colonnes typées
/// (texte, nombre, case à cocher, listes déroulantes), édition rapide,
/// ajout/suppression de lignes et copier/coller TSV/CSV.
///
/// ```rust
/// # use grio::*;
/// let mut editor = DataEditor::new("service_catalog")
///     .label("Catalogue des Services")
///     .column("id", "ID", ColumnType::Text)
///     .column("service", "Nom du Service", ColumnType::Text)
///     .column("active", "Actif", ColumnType::Boolean)
///     .column("sla", "SLA (h)", ColumnType::Number)
///     .allow_add(true)
///     .allow_delete(true)
///     .allow_paste(true);
/// ```
#[derive(Clone, Debug)]
pub struct DataEditor {
    id: String,
    label: String,
    columns: Vec<ColumnDef>,
    data: Vec<Vec<Value>>,
    interactive: bool,
    allow_add: bool,
    allow_delete: bool,
    allow_paste: bool,
    sortable: bool,
    max_height: Option<u32>,
}

impl DataEditor {
    /// Crée un nouvel éditeur de données avec son identifiant. Rôle `Input` par défaut.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            columns: Vec::new(),
            data: Vec::new(),
            interactive: true,
            allow_add: true,
            allow_delete: true,
            allow_paste: true,
            sortable: true,
            max_height: None,
        }
    }

    /// Libellé affiché au-dessus de la grille.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }

    /// Ajoute une colonne typée via un builder fluide.
    pub fn column(
        mut self,
        id: impl Into<String>,
        label: impl Into<String>,
        col_type: ColumnType,
    ) -> Self {
        self.columns.push(ColumnDef::new(id, label, col_type));
        self
    }

    /// Ajoute une colonne complète personnalisée.
    pub fn add_column(mut self, col: ColumnDef) -> Self {
        self.columns.push(col);
        self
    }

    /// Définit les lignes de données initiales (tableau 2D de `serde_json::Value`).
    pub fn data(mut self, rows: Vec<Vec<Value>>) -> Self {
        self.data = rows;
        self
    }

    /// Active ou désactive l'interactivité globale.
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }

    /// Autorise l'ajout de nouvelles lignes.
    pub fn allow_add(mut self, on: bool) -> Self {
        self.allow_add = on;
        self
    }

    /// Autorise la suppression de lignes.
    pub fn allow_delete(mut self, on: bool) -> Self {
        self.allow_delete = on;
        self
    }

    /// Autorise le copier-coller TSV/CSV direct depuis le presse-papier.
    pub fn allow_paste(mut self, on: bool) -> Self {
        self.allow_paste = on;
        self
    }

    /// Active ou désactive le tri par clic sur l'en-tête de colonne.
    pub fn sortable(mut self, on: bool) -> Self {
        self.sortable = on;
        self
    }

    /// Hauteur maximale avec défilement interne (en pixels).
    pub fn max_height(mut self, h: u32) -> Self {
        self.max_height = Some(h);
        self
    }
}

impl Component for DataEditor {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "dataeditor"
    }
    fn role(&self) -> Role {
        Role::Input
    }
    fn props(&self) -> Value {
        let cols_json: Vec<Value> = self.columns.iter().map(|c| c.to_json()).collect();
        json!({
            "label": self.label,
            "columns": cols_json,
            "data": self.data,
            "interactive": self.interactive,
            "allow_add": self.allow_add,
            "allow_delete": self.allow_delete,
            "allow_paste": self.allow_paste,
            "sortable": self.sortable,
            "max_height": self.max_height,
        })
    }
}

/// Traceur de séries temporelles accéléré par GPU matériel WebGL2 (`WebGlPlot`).
///
/// Capable de restituer 100 000 à 1 000 000+ de points à 60 FPS avec streaming binaire direct
/// et zéro surcharge de sérialisation JSON.
#[derive(Clone, Debug)]
pub struct WebGlPlot {
    id: String,
    label: String,
    title: Option<String>,
    xlabel: Option<String>,
    ylabel: Option<String>,
    colors: Vec<String>,
    height: u32,
    max_points: usize,
    show_fps: bool,
    interactive: bool,
    initial_series: Vec<WebGlSeries>,
}

/// Série de données initiale ou statique pour le composant `WebGlPlot`.
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WebGlSeries {
    /// Nom de la courbe (légende).
    pub name: String,
    /// Couleur hexadécimale (ex. `"#00f0ff"`).
    pub color: String,
    /// Échantillons de valeurs $Y$ (ou alternance $X, Y$).
    pub data: Vec<f32>,
}

impl WebGlPlot {
    /// Crée un nouveau traceur WebGL accéléré par GPU matériel.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            title: None,
            xlabel: None,
            ylabel: None,
            colors: vec![
                "#00f0ff".to_string(),
                "#ff007f".to_string(),
                "#7000ff".to_string(),
                "#00ff66".to_string(),
            ],
            height: 340,
            max_points: 200_000,
            show_fps: true,
            interactive: true,
            initial_series: Vec::new(),
        }
    }

    /// Libellé au-dessus du tracé.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }

    /// Titre principal du graphique.
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }

    /// Légende de l'axe horizontal X.
    pub fn xlabel(mut self, l: impl Into<String>) -> Self {
        self.xlabel = Some(l.into());
        self
    }

    /// Légende de l'axe vertical Y.
    pub fn ylabel(mut self, l: impl Into<String>) -> Self {
        self.ylabel = Some(l.into());
        self
    }

    /// Palette de couleurs hexadécimales pour les séries.
    pub fn colors(mut self, cols: &[&str]) -> Self {
        self.colors = cols.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Hauteur en pixels du canevas WebGL (par défaut 340px).
    pub fn height(mut self, h: u32) -> Self {
        self.height = h;
        self
    }

    /// Capacité maximale du tampon GPU en mémoire circulaire (nombre de points).
    pub fn max_points(mut self, n: usize) -> Self {
        self.max_points = n;
        self
    }

    /// Affiche le compteur de performance GPU en direct (FPS & points).
    pub fn show_fps(mut self, on: bool) -> Self {
        self.show_fps = on;
        self
    }

    /// Active l'interactivité (panoramique souris et zoom molette sur les axes).
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }

    /// Ajoute une série de données pré-remplie.
    pub fn series(
        mut self,
        name: impl Into<String>,
        color: impl Into<String>,
        data: &[f32],
    ) -> Self {
        self.initial_series.push(WebGlSeries {
            name: name.into(),
            color: color.into(),
            data: data.to_vec(),
        });
        self
    }
}

impl Component for WebGlPlot {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "webgl_plot"
    }
    fn role(&self) -> Role {
        Role::Output
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "title": self.title,
            "xlabel": self.xlabel,
            "ylabel": self.ylabel,
            "colors": self.colors,
            "height": self.height,
            "max_points": self.max_points,
            "show_fps": self.show_fps,
            "interactive": self.interactive,
            "series": self.initial_series,
        })
    }
}

/// Fonction d'agrégation supportée pour les métriques de `PivotTable`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum PivotAggregator {
    /// Somme arithmétique.
    #[serde(rename = "sum")]
    Sum,
    /// Moyenne arithmétique.
    #[serde(rename = "mean")]
    Mean,
    /// Décompte d'occurrences.
    #[serde(rename = "count")]
    Count,
    /// Minimum.
    #[serde(rename = "min")]
    Min,
    /// Maximum.
    #[serde(rename = "max")]
    Max,
}

impl PivotAggregator {
    /// Nom textuel de l'agrégateur.
    pub fn as_str(&self) -> &'static str {
        match self {
            PivotAggregator::Sum => "sum",
            PivotAggregator::Mean => "mean",
            PivotAggregator::Count => "count",
            PivotAggregator::Min => "min",
            PivotAggregator::Max => "max",
        }
    }
}

/// Composant de tableau croisé dynamique OLAP multidimensionnel (`PivotTable`).
///
/// Permet l'analyse et l'exploration de jeux de données avec pivotement interactif
/// des dimensions (lignes, colonnes) et calcul instantané côté client.
#[derive(Clone, Debug)]
pub struct PivotTable {
    id: String,
    label: String,
    headers: Vec<String>,
    data: Vec<Vec<Value>>,
    row_dimensions: Vec<String>,
    col_dimensions: Vec<String>,
    value_field: Option<String>,
    aggregator: PivotAggregator,
    height: Option<u32>,
}

impl PivotTable {
    /// Crée un tableau croisé dynamique OLAP.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            headers: Vec::new(),
            data: Vec::new(),
            row_dimensions: Vec::new(),
            col_dimensions: Vec::new(),
            value_field: None,
            aggregator: PivotAggregator::Sum,
            height: None,
        }
    }

    /// Libellé affiché au-dessus du tableau.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }

    /// Liste des noms de colonnes du jeu de données.
    pub fn headers(mut self, h: &[&str]) -> Self {
        self.headers = h.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Données brutes sous forme de matrice 2D.
    pub fn data(mut self, rows: Vec<Vec<Value>>) -> Self {
        self.data = rows;
        self
    }

    /// Dimensions de regroupement en lignes (par exemple `&["Region", "Category"]`).
    pub fn rows(mut self, r: &[&str]) -> Self {
        self.row_dimensions = r.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Dimensions de regroupement en colonnes (par exemple `&["Quarter"]`).
    pub fn cols(mut self, c: &[&str]) -> Self {
        self.col_dimensions = c.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Champ numérique à agréger (par exemple `"Revenue"`).
    pub fn value_field(mut self, v: impl Into<String>) -> Self {
        self.value_field = Some(v.into());
        self
    }

    /// Fonction d'agrégation (Sum, Mean, Count, Min, Max).
    pub fn aggregator(mut self, agg: PivotAggregator) -> Self {
        self.aggregator = agg;
        self
    }

    /// Hauteur maximale en pixels avec défilement interne.
    pub fn height(mut self, h: u32) -> Self {
        self.height = Some(h);
        self
    }
}

impl Component for PivotTable {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "pivot_table"
    }
    fn role(&self) -> Role {
        Role::Output
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "headers": self.headers,
            "data": self.data,
            "row_dimensions": self.row_dimensions,
            "col_dimensions": self.col_dimensions,
            "value_field": self.value_field,
            "aggregator": self.aggregator.as_str(),
            "height": self.height,
        })
    }
}
