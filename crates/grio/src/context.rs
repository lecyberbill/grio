//! Contexte fourni aux handlers : lecture des entrées, écriture des sorties,
//! mise à jour en temps réel (streaming, progress, alertes) et émission
//! d'événements applicatifs.
//!
//! Le `Context` est `Send` : les handlers peuvent s'exécuter sur un pool de
//! threads (`spawn_blocking`) sans bloquer le serveur.
//!
//! Lifecycle d'une passe : le serveur construit un `Context` partagé, `App`
//! applique aux handlers leur éventuel **flux déclaré** (`set_flow`), puis
//! accumule les mises à jour finales (`take_all`) et les événements (`take_emitted`).

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::de::DeserializeOwned;
use serde::Serialize;
use serde_json::{json, Value};
use tokio::sync::mpsc::UnboundedSender;

use crate::events::WireEvent;
use crate::{Error, Result};

/// Niveau d'une alerte utilisateur (`ctx.alert`), stylée côté client.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AlertLevel {
    /// Information neutre.
    Info,
    /// Confirmation d'une réussite.
    Success,
    /// Avertissement.
    Warn,
    /// Erreur.
    Error,
}

impl AlertLevel {
    /// Nom court sérialisé (`info`, `success`, `warn`, `error`).
    pub fn as_str(&self) -> &'static str {
        match self {
            AlertLevel::Info => "info",
            AlertLevel::Success => "success",
            AlertLevel::Warn => "warn",
            AlertLevel::Error => "error",
        }
    }
}

/// Contexte d'exécution d'une passe d'événements.
///
/// * **Lecture** : instantané des valeurs d'entrée (via `get`, `get_f64`, …).
/// * **Écriture** : `set`/`set_prop` remplacent la valeur (ou une propriété) —
///   livrées en temps réel **et** incluses dans la réponse finale (donc dans
///   `/api/predict`).
/// * **Streaming** : `append` (fragments concaténés côté client), `progress`
///   (barre) et `alert` (toast) sont poussés immédiatement **uniquement** en
///   temps réel — absents de la réponse finale.
/// * **Contrôle** : `cancelled()` signale l'annulation d'un nouveau
///   déclenchement sur la même action ; `emit` déclenche les `on_event`.
pub struct Context {
    inputs: Arc<HashMap<String, Value>>,
    push: Option<UnboundedSender<Value>>,
    cancel: Arc<AtomicBool>,
    event: Option<WireEvent>,
    flow_in: Option<HashSet<String>>,
    flow_out: Option<HashSet<String>>,
    skipped: HashSet<String>,
    all: Vec<(String, Value)>,
    emitted: Vec<(String, Value)>,
    wasm: Arc<crate::wasm::WasmRegistry>,
}

impl Context {
    pub(crate) fn new(
        inputs: Arc<HashMap<String, Value>>,
        push: Option<UnboundedSender<Value>>,
        cancel: Arc<AtomicBool>,
        event: Option<WireEvent>,
    ) -> Self {
        Self {
            inputs,
            push,
            cancel,
            event,
            flow_in: None,
            flow_out: None,
            skipped: HashSet::new(),
            all: Vec::new(),
            emitted: Vec::new(),
            wasm: Arc::new(crate::wasm::WasmRegistry::new()),
        }
    }

    pub(crate) fn with_wasm(mut self, wasm: Arc<crate::wasm::WasmRegistry>) -> Self {
        self.wasm = wasm;
        self
    }

    /// Événement d'origine de la passe (si la passe vient d'un événement
    /// client) — donne accès à la cible (`c`), l'action (`e`), les données
    /// (`d`) et l'instantané de valeurs (`v`).
    pub fn event(&self) -> Option<&WireEvent> {
        self.event.as_ref()
    }

    pub(crate) fn set_flow(&mut self, inputs: Option<&[String]>, outputs: Option<&[String]>) {
        self.flow_in = inputs.map(|l| l.iter().cloned().collect());
        self.flow_out = outputs.map(|l| l.iter().cloned().collect());
    }

    /// « Ne pas toucher » cette sortie : les prochains `set`/`set_prop`/
    /// `append`/`progress` sur `id` sont ignorés (rien n'est envoyé au client).
    pub fn skip(&mut self, id: impl Into<String>) {
        self.skipped.insert(id.into());
    }

    /// `true` si le composant `id` a été marqué ignoré par `skip`.
    pub fn skipped(&self, id: &str) -> bool {
        self.skipped.contains(id)
    }

    /// Inverse la décision de `skip(id)`.
    pub fn unskip(&mut self, id: impl Into<String>) {
        self.skipped.remove(&id.into());
    }

    /// Lit la valeur d'un composant d'entrée, déserialisée en `T`.
    ///
    /// Échoue si l'identifiant est inconnu, hors du **flux déclaré** du
    /// handler (voir `.flow()`), ou si la valeur ne peut pas être convertie
    /// (l'erreur est alors affichée comme toast côté client).
    pub fn get<T: DeserializeOwned>(&self, id: &str) -> Result<T> {
        if self.flow_in.as_ref().is_some_and(|s| !s.contains(id)) {
            return Err(Error::from(format!("input `{id}` hors du flux déclaré")));
        }
        let v = self
            .inputs
            .get(id)
            .ok_or_else(|| Error::from(format!("unknown input `{id}`")))?;
        Ok(serde_json::from_value(v.clone())?)
    }

    /// Lit une valeur numérique (`f64`).
    pub fn get_f64(&self, id: &str) -> Result<f64> {
        self.get(id)
    }

    /// Lit une valeur texte sans fallback.
    pub fn get_str(&self, id: &str) -> Option<&str> {
        if self.flow_in.as_ref().is_some_and(|s| !s.contains(id)) {
            return None;
        }
        self.inputs.get(id).and_then(|v| v.as_str())
    }

    /// Vérifie qu'un composant d'entrée existe (dans le flux déclaré si présent).
    pub fn has(&self, id: &str) -> bool {
        if self.flow_in.as_ref().is_some_and(|s| !s.contains(id)) {
            return false;
        }
        self.inputs.contains_key(id)
    }

    /// Remplace la valeur d'un composant (envoyée au client + API).
    ///
    /// Livrée en temps réel **et** incluse dans la réponse finale du handler.
    /// Ignorée si `id` est hors du flux déclaré (`flow`) ou marqué `skip`.
    pub fn set<T: Serialize>(&mut self, id: impl Into<String>, v: T) {
        let id = id.into();
        if self.blocked_writes(&id) {
            return;
        }
        let patch = json!({ "value": serde_json::to_value(v).unwrap_or(Value::Null) });
        self.push_update(id, patch);
    }

    /// Modifie une propriété de *configuration* d'un composant (ex.
    /// `"visible"`, `"disabled"`, `"label"`) sans changer sa valeur.
    /// Soumise aux mêmes gardes que `set`.
    pub fn set_prop<T: Serialize>(&mut self, id: impl Into<String>, prop: &str, v: T) {
        let id = id.into();
        if self.blocked_writes(&id) {
            return;
        }
        let patch = json!({ prop: serde_json::to_value(v).unwrap_or(Value::Null) });
        self.push_update(id, patch);
    }

    /// Contrôle de visibilité universel : affiche ou masque un composant à la volée.
    ///
    /// ```rust
    /// # use grio::*;
    /// # fn example(ctx: &mut Context) {
    /// ctx.set_visible("error_panel", false);
    /// # }
    /// ```
    pub fn set_visible(&mut self, id: impl Into<String>, visible: bool) {
        self.set_prop(id, "visible", visible);
    }

    /// **Injection de slot dynamique** : injecte un nouveau composant à chaud
    /// à l'intérieur d'un `DynamicContainer`.
    ///
    /// ```rust
    /// # use grio::*;
    /// # fn example(ctx: &mut Context) {
    /// ctx.append_component("dynamic_slot", Text::new("new_msg").value("Nouveau message"));
    /// # }
    /// ```
    pub fn append_component(
        &mut self,
        container_id: impl Into<String>,
        c: impl crate::components::Component + 'static,
    ) {
        let container_id = container_id.into();
        let html = crate::server::render_fragment(&c);
        self.send(json!({
            "t": "slot",
            "container": container_id,
            "mode": "append",
            "html": html,
        }));
    }

    /// **Remplacement de slot dynamique** : remplace la totalité des enfants
    /// d'un `DynamicContainer` par une nouvelle liste de composants.
    pub fn replace_children(
        &mut self,
        container_id: impl Into<String>,
        items: Vec<Box<dyn crate::components::Component>>,
    ) {
        let container_id = container_id.into();
        let mut html = String::new();
        for it in items {
            html.push_str(&crate::server::render_fragment(it.as_ref()));
        }
        self.send(json!({
            "t": "slot",
            "container": container_id,
            "mode": "replace",
            "html": html,
        }));
    }

    /// **Vidage de slot dynamique** : retire tous les enfants du conteneur.
    pub fn clear_container(&mut self, container_id: impl Into<String>) {
        let container_id = container_id.into();
        self.send(json!({
            "t": "slot",
            "container": container_id,
            "mode": "clear",
            "html": "",
        }));
    }

    /// **Streaming Big Data / Dataframe / DataEditor** : injecte un lot de lignes (batch)
    /// dans une table existante sans écraser les données précédentes (pour flux Snowflake / Polars).
    ///
    /// ```rust
    /// # use grio::*;
    /// # fn example(ctx: &mut Context) {
    /// ctx.append_rows("analytics_grid", vec![
    ///     vec![serde_json::json!("TX-9901"), serde_json::json!(42.50)],
    ///     vec![serde_json::json!("TX-9902"), serde_json::json!(108.20)],
    /// ]);
    /// # }
    /// ```
    pub fn append_rows(&mut self, id: impl Into<String>, rows: Vec<Vec<serde_json::Value>>) {
        let id = id.into();
        self.send(json!({
            "t": "patch",
            "c": id,
            "append_rows": rows,
        }));
    }

    /// **Streaming binaire direct / WebGL & Arrow** : injecte un flux binaire brut
    /// sans aucune sérialisation JSON via WebSocket (zéro copie côté GPU).
    ///
    /// ```rust
    /// # use grio::*;
    /// # fn example(ctx: &mut Context) {
    /// let samples: Vec<f32> = (0..10_000).map(|i| (i as f32 * 0.05).sin()).collect();
    /// ctx.append_f32_points("gpu_chart", &samples);
    /// # }
    /// ```
    pub fn append_binary(&mut self, id: impl Into<String>, data: &[u8]) {
        let id = id.into();
        let b64 = crate::media::encode(data);
        self.send(json!({
            "t": "bin",
            "c": id,
            "b64": b64,
        }));
    }

    /// **Streaming haute fréquence f32** : transmet un tableau continu d'échantillons `f32`
    /// directement interprétable comme un `Float32Array` WebGL côté navigateur.
    pub fn append_f32_points(&mut self, id: impl Into<String>, points: &[f32]) {
        let bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                points.as_ptr() as *const u8,
                points.len() * std::mem::size_of::<f32>(),
            )
        };
        self.append_binary(id, bytes);
    }

    /// **Streaming multi-séries WebGL** : pousse des points étiquetés par série.
    pub fn append_series_points(
        &mut self,
        id: impl Into<String>,
        series_index: u32,
        points: &[f32],
    ) {
        let id = id.into();
        let bytes: &[u8] = unsafe {
            std::slice::from_raw_parts(
                points.as_ptr() as *const u8,
                points.len() * std::mem::size_of::<f32>(),
            )
        };
        let b64 = crate::media::encode(bytes);
        self.send(json!({
            "t": "bin_series",
            "c": id,
            "s": series_index,
            "b64": b64,
        }));
    }

    /// **Streaming** : ajoute un fragment à la valeur d'un composant
    /// (les fragments sont concaténés côté client). Poussé immédiatement,
    /// uniquement en temps réel — absent de la réponse finale.
    pub fn append<T: Serialize>(&mut self, id: impl Into<String>, fragment: T) {
        let id = id.into();
        if self.blocked_writes(&id) {
            return;
        }
        let patch = json!({ "append": serde_json::to_value(fragment).unwrap_or(Value::Null) });
        self.send(json!({ "t": "update", "u": [json!({ "id": id, "p": patch })] }));
    }

    /// Met à jour un composant `progress` (`f` entre 0.0 et 1.0, `label`
    /// optionnel). Poussé immédiatement, uniquement en temps réel.
    pub fn progress<T: Serialize>(&mut self, id: impl Into<String>, f: f64, label: T) {
        let id = id.into();
        if self.blocked_writes(&id) {
            return;
        }
        let patch = json!({ "value": json!({ "progress": f, "label": label }) });
        self.send(json!({ "t": "update", "u": [json!({ "id": id, "p": patch })] }));
    }

    /// Affiche une alerte (toast) au client. Poussée immédiatement — n'est
    /// pas incluse dans la réponse finale.
    pub fn alert<T: Serialize>(&mut self, level: impl Into<AlertLevel>, msg: T) {
        let level = level.into();
        self.send(json!({ "t": "alert", "level": level.as_str(), "m": msg }));
        // Note : les alertes ne sont pas accumulées dans `all`.
    }

    /// **Changement dynamique de thème à chaud** : applique un nouveau `Theme`
    /// en temps réel sur le navigateur du client sans recharger la page.
    pub fn set_theme(&mut self, theme: crate::app::Theme) {
        let mode_str = match theme.mode {
            crate::app::ThemeMode::Dark => "dark",
            crate::app::ThemeMode::Light => "light",
            crate::app::ThemeMode::System => "system",
        };
        self.send(json!({
            "t": "theme",
            "mode": mode_str,
            "primary": theme.primary,
            "radius": theme.radius,
            "font": theme.font,
        }));
    }

    /// **Exécution sandboxée d'un greffon WebAssembly** : invoque une fonction
    /// exposée par un plugin WASM enregistré avec passage de charge utile JSON.
    ///
    /// ```rust
    /// # use grio::*;
    /// # fn example(ctx: &mut Context) -> grio::Result<()> {
    /// let res = ctx.call_wasm("text_moderator", "filter", &serde_json::json!({ "text": "bonjour" }))?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn call_wasm(&self, plugin_id: &str, method: &str, input: &Value) -> Result<Value> {
        let plugin = self.wasm.get(plugin_id).ok_or_else(|| {
            Error::from(format!("plugin WebAssembly `{plugin_id}` introuvable dans le registre"))
        })?;
        plugin.call(method, input)
    }

    /// **Exécution binaire directe d'un greffon WebAssembly** : invoque une fonction
    /// bas niveau avec des tampons d'octets sans overhead de sérialisation JSON.
    pub fn call_wasm_bytes(&self, plugin_id: &str, method: &str, input: &[u8]) -> Result<Vec<u8>> {
        let plugin = self.wasm.get(plugin_id).ok_or_else(|| {
            Error::from(format!("plugin WebAssembly `{plugin_id}` introuvable dans le registre"))
        })?;
        plugin.invoke_bytes(method, input)
    }

    /// `true` si le job courant a été annulé (nouveau déclenchement sur la
    /// même action). À vérifier dans les boucles longues.
    pub fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    /// Émet un événement applicatif local, distribué aux listeners
    /// `App::on_event(nom)`. Une garde anti-boucle limite la profondeur.
    pub fn emit<T: Serialize>(&mut self, name: impl Into<String>, data: T) {
        self.emitted.push((
            name.into(),
            serde_json::to_value(data).unwrap_or(Value::Null),
        ));
    }

    fn push_update(&mut self, id: String, patch: Value) {
        self.all.push((id.clone(), patch.clone()));
        // Livraison immédiate en temps réel, en plus de la réponse finale.
        self.send(json!({ "t": "update", "u": [json!({ "id": id, "p": patch })] }));
    }

    /// Garde commune aux écritures : hors flux déclaré (`flow_out`) ou marqué
    /// `skip` → la mise à jour est ignorée silencieusement.
    fn blocked_writes(&self, id: &str) -> bool {
        self.skipped.contains(id) || self.flow_out.as_ref().is_some_and(|s| !s.contains(id))
    }

    fn send(&self, msg: Value) {
        if let Some(tx) = &self.push {
            let _ = tx.send(msg);
        }
    }

    pub(crate) fn take_all(&mut self) -> Vec<(String, Value)> {
        std::mem::take(&mut self.all)
    }

    pub(crate) fn take_emitted(&mut self) -> Vec<(String, Value)> {
        std::mem::take(&mut self.emitted)
    }
}
