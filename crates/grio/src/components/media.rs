//! Composants de médias et vision par ordinateur (Image, ImageEditor, Audio, Video, Gallery, etc.).

use serde_json::{json, Value};

use super::{Component, Role};

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
        Self {
            id: id.into(),
            label: String::new(),
            value: String::new(),
            interactive: true,
            out: false,
        }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Data URL initiale affichée.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    /// Ouvre/ferme le contrôle d'upload (fichier ou glisser-déposer).
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
    /// Déclare l'image **sortie** (résultat calculé, pas d'upload).
    pub fn output(mut self) -> Self {
        self.out = true;
        self
    }
    /// Déclare l'image **entrée** (upload — comportement par défaut).
    pub fn input(mut self) -> Self {
        self.out = false;
        self
    }
}

impl Component for Image {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "image"
    }
    fn role(&self) -> Role {
        if self.out {
            Role::Output
        } else {
            Role::Input
        }
    }
    fn props(&self) -> Value {
        json!({ "label": self.label, "value": self.value, "interactive": self.interactive })
    }
}

/// Éditeur d'image **côté client** : pinceau, gomme, formes (rect/ligne/
/// flèche), rognage (crop), rotation, filtres, annuler/rétablir, zoom/pan et
/// **calques** (2 par défaut, `.layers(n)`).
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
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Image d'arrière-plan initiale (data URL) chargée dans le canvas.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    /// Autorise ou non l'édition (affichage seul si `false`).
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
    /// Nombre de **calques** d'annotation (1..=4, défaut 2).
    pub fn layers(mut self, n: usize) -> Self {
        self.layers = n.clamp(1, 4);
        self
    }
    /// Active le pinceau et la gomme (défaut : `true`).
    pub fn brush(mut self, on: bool) -> Self {
        self.brush = on;
        self
    }
    /// Active le **rognage** (crop) (défaut : `true`).
    pub fn crop(mut self, on: bool) -> Self {
        self.crop = on;
        self
    }
    /// Active les formes : rectangle, ligne, flèche (défaut : `true`).
    pub fn shapes(mut self, on: bool) -> Self {
        self.shapes = on;
        self
    }
    /// Active les filtres (niveaux de gris, inversion, luminosité, flou).
    pub fn filters(mut self, on: bool) -> Self {
        self.filters = on;
        self
    }
    /// Active la rotation 90° (défaut : `true`).
    pub fn rotflip(mut self, on: bool) -> Self {
        self.rotflip = on;
        self
    }
    /// Déclare l'éditeur **sortie** (affichage seul de `value`).
    pub fn output(mut self) -> Self {
        self.out = true;
        self
    }
    /// Déclare l'éditeur **entrée** (édition — par défaut).
    pub fn input(mut self) -> Self {
        self.out = false;
        self
    }
}

impl Component for ImageEditor {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "imageeditor"
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
            "label": self.label, "value": self.value, "interactive": self.interactive,
            "layers": self.layers, "brush": self.brush, "crop": self.crop,
            "shapes": self.shapes, "filters": self.filters, "rotflip": self.rotflip
        })
    }
}

/// Audio : **upload** (entrée, interactive), **affichage/lecteur** (sortie) ou
/// **micro live** (`.live(true)`).
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
        Self {
            id: id.into(),
            label: String::new(),
            value: String::new(),
            interactive: true,
            out: false,
            live: false,
        }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Data URL initiale.
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    /// Ouvre/ferme le contrôle d'upload.
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
    /// Déclare l'audio **sortie** (lecteur de résultats).
    pub fn output(mut self) -> Self {
        self.out = true;
        self
    }
    /// Déclare l'audio **entrée** (upload — par défaut).
    pub fn input(mut self) -> Self {
        self.out = false;
        self
    }
    /// Mode **micro live** : capture micro client et streaming de chunks.
    pub fn live(mut self, on: bool) -> Self {
        self.live = on;
        self
    }
}

impl Component for Audio {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "audio"
    }
    fn role(&self) -> Role {
        if self.out {
            Role::Output
        } else {
            Role::Input
        }
    }
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
        Self {
            id: id.into(),
            label: String::new(),
            value: String::new(),
            interactive: false,
            out: true,
            live: false,
        }
    }
    /// Étiquette affichée au-dessus.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Data URL initiale (ou URL de flux).
    pub fn value(mut self, v: impl Into<String>) -> Self {
        self.value = v.into();
        self
    }
    /// Ouvre/ferme le contrôle d'upload.
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
    /// Déclare la vidéo **entrée** (upload).
    pub fn input(mut self) -> Self {
        self.out = false;
        self
    }
    /// Déclare la vidéo **sortie** (affichage — par défaut).
    pub fn output(mut self) -> Self {
        self.out = true;
        self
    }
    /// Mode **caméra live** : la caméra s'affiche en local et pousse des chunks.
    pub fn live(mut self, on: bool) -> Self {
        self.live = on;
        self
    }
}

impl Component for Video {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "video"
    }
    fn role(&self) -> Role {
        if self.out {
            Role::Output
        } else {
            Role::Input
        }
    }
    fn props(&self) -> Value {
        json!({ "label": self.label, "value": self.value, "interactive": self.interactive, "live": self.live })
    }
}

/// Galerie d'images en grille (affichage et/ou upload multiple, avec lightbox/zoom).
#[derive(Clone, Debug)]
pub struct Gallery {
    id: String,
    label: String,
    value: Value,
    columns: usize,
    rows: Option<usize>,
    height: Option<String>,
    min_height: Option<String>,
    max_height: Option<String>,
    limit: Option<usize>,
    item_height: Option<String>,
    item_width: Option<String>,
    aspect_ratio: Option<String>,
    object_fit: String,
    allow_preview: bool,
    interactive: bool,
    upload: bool,
    out: bool,
}

impl Gallery {
    /// Crée une galerie, avec son identifiant. Entrée (upload) par défaut.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            value: json!([]),
            columns: 3,
            rows: None,
            height: None,
            min_height: None,
            max_height: None,
            limit: None,
            item_height: None,
            item_width: None,
            aspect_ratio: None,
            object_fit: "cover".into(),
            allow_preview: true,
            interactive: true,
            upload: true,
            out: false,
        }
    }
    /// Étiquette/Titre affiché au-dessus de la galerie.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Alias pour label (titre).
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.label = t.into();
        self
    }
    /// Images initiales (data URLs ou URLs).
    pub fn value(mut self, items: &[&str]) -> Self {
        let v: Vec<String> = items.iter().map(|s| s.to_string()).collect();
        self.value = json!(v);
        self
    }
    /// Images initiales depuis une collection de Strings ou Data URLs.
    pub fn items(mut self, items: &[impl AsRef<str>]) -> Self {
        let v: Vec<String> = items.iter().map(|s| s.as_ref().to_string()).collect();
        self.value = json!(v);
        self
    }
    /// Images initiales depuis une collection d'objets sérialisables (`{image, caption}`).
    pub fn raw_items(mut self, items: impl serde::Serialize) -> Self {
        self.value = serde_json::to_value(items).unwrap_or_else(|_| json!([]));
        self
    }
    /// Nombre de colonnes de la grille.
    pub fn columns(mut self, n: usize) -> Self {
        self.columns = n.max(1);
        self
    }
    /// Nombre de lignes visibles simultanément (active le défilement si plus d'images).
    pub fn rows(mut self, n: usize) -> Self {
        self.rows = Some(n.max(1));
        self
    }
    /// Hauteur fixe ou relative de la galerie (ex: "500px", "60vh", 500.0).
    pub fn height(mut self, h: impl Into<String>) -> Self {
        self.height = Some(h.into());
        self
    }
    /// Hauteur minimale du conteneur de la galerie.
    pub fn min_height(mut self, mh: impl Into<String>) -> Self {
        self.min_height = Some(mh.into());
        self
    }
    /// Hauteur maximale du conteneur avant scrollbar (ex: "650px", "80vh").
    pub fn max_height(mut self, mh: impl Into<String>) -> Self {
        self.max_height = Some(mh.into());
        self
    }
    /// Limite maximale d'images affichées dans la galerie (les plus récentes en premier).
    pub fn limit(mut self, n: usize) -> Self {
        self.limit = Some(n);
        self
    }
    /// Hauteur d'une vignette / image de la galerie (ex: "140px", "180px", "220px").
    pub fn item_height(mut self, ih: impl Into<String>) -> Self {
        self.item_height = Some(ih.into());
        self
    }
    /// Largeur d'une vignette / image de la galerie (ex: "180px", "240px").
    pub fn item_width(mut self, iw: impl Into<String>) -> Self {
        self.item_width = Some(iw.into());
        self
    }
    /// Ratio d'aspect d'une vignette (ex: "1/1", "16/9", "4/3", "9/16").
    pub fn aspect_ratio(mut self, ar: impl Into<String>) -> Self {
        self.aspect_ratio = Some(ar.into());
        self
    }
    /// Mode d'ajustement des images (`"cover"`, `"contain"`, `"scale-down"`).
    pub fn object_fit(mut self, fit: impl Into<String>) -> Self {
        self.object_fit = fit.into();
        self
    }
    /// Active la visionneuse / lightbox plein écran au clic (défaut : `true`).
    pub fn allow_preview(mut self, allow: bool) -> Self {
        self.allow_preview = allow;
        self
    }
    /// Autorise la sélection/upload.
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
    /// Affiche le bouton d'ajout d'image.
    pub fn upload(mut self, on: bool) -> Self {
        self.upload = on;
        self
    }
    /// Déclare la galerie **sortie** (affichage seul).
    pub fn output(mut self) -> Self {
        self.out = true;
        self
    }
    /// Déclare la galerie **entrée** (upload — par défaut).
    pub fn input(mut self) -> Self {
        self.out = false;
        self
    }
}

impl Component for Gallery {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "gallery"
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
            "value": self.value,
            "columns": self.columns,
            "rows": self.rows,
            "height": self.height,
            "min_height": self.min_height,
            "max_height": self.max_height,
            "limit": self.limit,
            "item_height": self.item_height,
            "item_width": self.item_width,
            "aspect_ratio": self.aspect_ratio,
            "object_fit": self.object_fit,
            "allow_preview": self.allow_preview,
            "interactive": self.interactive,
            "upload": self.upload
        })
    }
}

/// **Bounding box** representation for [`AnnotatedImage`].
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize, PartialEq)]
pub struct BoundingBox {
    /// Normalized coordinates `[ymin, xmin, ymax, xmax]` in `0.0..=1.0`.
    pub box_coords: [f64; 4],
    /// Classification label (e.g. `"person"`, `"car"`, `"cat"`).
    pub label: String,
    /// Confidence score (e.g. `0.95`).
    pub score: Option<f64>,
    /// Hex stroke & tag color (e.g. `"#6366f1"`).
    pub color: String,
}

/// **AnnotatedImage** component (`gr.AnnotatedImage` equivalent): displays a base
/// image with vector bounding boxes, labels, and confidence tags.
#[derive(Clone, Debug)]
pub struct AnnotatedImage {
    id: String,
    label: String,
    image: String,
    boxes: Vec<BoundingBox>,
    show_labels: bool,
    show_scores: bool,
}

impl AnnotatedImage {
    /// Creates an annotated image viewer with its identifier. Output by default.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            image: String::new(),
            boxes: Vec::new(),
            show_labels: true,
            show_scores: true,
        }
    }
    /// Label displayed above the image viewer.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Base background image URL or data URL.
    pub fn image(mut self, img: impl Into<String>) -> Self {
        self.image = img.into();
        self
    }
    /// Adds a normalized bounding box `[ymin, xmin, ymax, xmax]` with label, score and color.
    #[allow(clippy::too_many_arguments)]
    pub fn box_norm(
        mut self,
        ymin: f64,
        xmin: f64,
        ymax: f64,
        xmax: f64,
        label: impl Into<String>,
        score: Option<f64>,
        color: impl Into<String>,
    ) -> Self {
        self.boxes.push(BoundingBox {
            box_coords: [ymin, xmin, ymax, xmax],
            label: label.into(),
            score,
            color: color.into(),
        });
        self
    }
    /// Sets the full list of bounding boxes.
    pub fn boxes(mut self, b: Vec<BoundingBox>) -> Self {
        self.boxes = b;
        self
    }
    /// Sets full JSON data: `{ "image": "...", "boxes": [...] }`.
    pub fn data<T: serde::Serialize>(mut self, d: &T) -> Self {
        if let Ok(v) = serde_json::to_value(d) {
            if let Some(img) = v.get("image").and_then(|x| x.as_str()) {
                self.image = img.to_string();
            }
            if let Some(arr) = v.get("boxes").and_then(|x| x.as_array()) {
                if let Ok(b) = serde_json::from_value::<Vec<BoundingBox>>(Value::Array(arr.clone()))
                {
                    self.boxes = b;
                }
            }
        }
        self
    }
    /// Shows or hides class label badges on boxes.
    pub fn show_labels(mut self, on: bool) -> Self {
        self.show_labels = on;
        self
    }
    /// Shows or hides percentage confidence scores.
    pub fn show_scores(mut self, on: bool) -> Self {
        self.show_scores = on;
        self
    }
}

impl Component for AnnotatedImage {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "annotatedimage"
    }
    fn role(&self) -> Role {
        Role::Output
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "image": self.image,
            "boxes": self.boxes,
            "show_labels": self.show_labels,
            "show_scores": self.show_scores
        })
    }
}

/// **ImageComparison** component (`gr.ImageSlider` equivalent): interactive
/// before-and-after image viewer with a draggable divider slider.
#[derive(Clone, Debug)]
pub struct ImageComparison {
    id: String,
    label: String,
    before: String,
    after: String,
    before_label: String,
    after_label: String,
    position: f64,
}

impl ImageComparison {
    /// Creates an image comparison viewer with its identifier. Output by default.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            before: String::new(),
            after: String::new(),
            before_label: "Before".into(),
            after_label: "After".into(),
            position: 50.0,
        }
    }
    /// Label displayed above the comparison viewer.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Sets before image URL and optional badge label.
    pub fn before(mut self, img: impl Into<String>, label: impl Into<String>) -> Self {
        self.before = img.into();
        self.before_label = label.into();
        self
    }
    /// Sets after image URL and optional badge label.
    pub fn after(mut self, img: impl Into<String>, label: impl Into<String>) -> Self {
        self.after = img.into();
        self.after_label = label.into();
        self
    }
    /// Initial slider position percentage (e.g. `50.0`).
    pub fn position(mut self, pos: f64) -> Self {
        self.position = pos.clamp(0.0, 100.0);
        self
    }
}

impl Component for ImageComparison {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "imagecomparison"
    }
    fn role(&self) -> Role {
        Role::Output
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "before": self.before,
            "after": self.after,
            "before_label": self.before_label,
            "after_label": self.after_label,
            "position": self.position
        })
    }
}

/// **AudioRecorder** component: direct microphone recording with dedicated REC
/// button, active recording animation, elapsed timer, and audio export.
#[derive(Clone, Debug)]
pub struct AudioRecorder {
    id: String,
    label: String,
    max_duration: f64,
    interactive: bool,
}

impl AudioRecorder {
    /// Creates an audio recorder component with default 60-second limit. Input by default.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            max_duration: 60.0,
            interactive: true,
        }
    }
    /// Label displayed above the recorder.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    /// Maximum recording duration in seconds (default `60.0`).
    pub fn max_duration(mut self, d: f64) -> Self {
        self.max_duration = d;
        self
    }
    /// Enables or disables recording interaction.
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
}

impl Component for AudioRecorder {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "audiorecorder"
    }
    fn role(&self) -> Role {
        Role::Input
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "max_duration": self.max_duration,
            "interactive": self.interactive
        })
    }
}

/// Surlignage / boîte de détection sur une page de document PDF (pour RAG et OCR).
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PdfHighlight {
    /// Numéro de page (1-indexé).
    pub page: u32,
    /// Coordonnée X normalisée (0.0 à 1.0) ou en pourcentage.
    pub x: f64,
    /// Coordonnée Y normalisée (0.0 à 1.0) ou en pourcentage.
    pub y: f64,
    /// Largeur normalisée (0.0 à 1.0).
    pub width: f64,
    /// Hauteur normalisée (0.0 à 1.0).
    pub height: f64,
    /// Libellé ou extrait textuel associé.
    pub label: Option<String>,
    /// Couleur du surlignage (code hexadécimal ou CSS).
    pub color: Option<String>,
}

/// **Pdf** : Visualiseur de documents PDF interactif avec navigation multi-pages,
/// zoom et surlignage dynamique de boîtes de détection / extractions RAG/OCR.
///
/// ```rust
/// # use grio::*;
/// Pdf::new("invoice")
///     .label("Facture & Extraction RAG")
///     .src("https://example.com/doc.pdf")
///     .page(1)
///     .highlight(1, 0.1, 0.2, 0.5, 0.08, "Montant Total: 1 250 €", "#6366f1");
/// ```
#[derive(Clone, Debug)]
pub struct Pdf {
    id: String,
    label: String,
    src: String,
    page: u32,
    total_pages: Option<u32>,
    zoom: f64,
    highlights: Vec<PdfHighlight>,
    show_toolbar: bool,
    interactive: bool,
}

impl Pdf {
    /// Crée un nouveau composant PDF avec son identifiant. Rôle `Output` par défaut.
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: String::new(),
            src: String::new(),
            page: 1,
            total_pages: None,
            zoom: 1.0,
            highlights: Vec::new(),
            show_toolbar: true,
            interactive: true,
        }
    }

    /// Libellé affiché au-dessus du visualiseur.
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }

    /// Source du fichier PDF (URL HTTP, chemin relatif ou data URL base64).
    pub fn src(mut self, s: impl Into<String>) -> Self {
        self.src = s.into();
        self
    }

    /// Page initiale affichée (1-indexée).
    pub fn page(mut self, p: u32) -> Self {
        self.page = p.max(1);
        self
    }

    /// Nombre total de pages (optionnel).
    pub fn total_pages(mut self, total: u32) -> Self {
        self.total_pages = Some(total);
        self
    }

    /// Facteur de zoom initial (ex. 1.0 pour 100%, 1.25 pour 125%).
    pub fn zoom(mut self, z: f64) -> Self {
        self.zoom = z.max(0.2).min(4.0);
        self
    }

    /// Ajoute une zone de surlignage RAG/OCR sur une page donnée.
    pub fn highlight(
        mut self,
        page: u32,
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        label: impl Into<String>,
        color: impl Into<String>,
    ) -> Self {
        self.highlights.push(PdfHighlight {
            page,
            x,
            y,
            width,
            height,
            label: Some(label.into()),
            color: Some(color.into()),
        });
        self
    }

    /// Définit la liste complète des surlignages.
    pub fn highlights(mut self, hl: Vec<PdfHighlight>) -> Self {
        self.highlights = hl;
        self
    }

    /// Affiche ou masque la barre d'outils de pagination et zoom.
    pub fn show_toolbar(mut self, on: bool) -> Self {
        self.show_toolbar = on;
        self
    }

    /// Active ou désactive l'interactivité utilisateur.
    pub fn interactive(mut self, on: bool) -> Self {
        self.interactive = on;
        self
    }
}

impl Component for Pdf {
    fn id(&self) -> &str {
        &self.id
    }
    fn kind(&self) -> &'static str {
        "pdf"
    }
    fn role(&self) -> Role {
        Role::Output
    }
    fn props(&self) -> Value {
        json!({
            "label": self.label,
            "src": self.src,
            "page": self.page,
            "total_pages": self.total_pages,
            "zoom": self.zoom,
            "highlights": self.highlights,
            "show_toolbar": self.show_toolbar,
            "interactive": self.interactive,
        })
    }
}

