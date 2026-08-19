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
        }
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
        let v = self.inputs.get(id).ok_or_else(|| Error::from(format!("unknown input `{id}`")))?;
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

    /// `true` si le job courant a été annulé (nouveau déclenchement sur la
    /// même action). À vérifier dans les boucles longues.
    pub fn cancelled(&self) -> bool {
        self.cancel.load(Ordering::SeqCst)
    }

    /// Émet un événement applicatif local, distribué aux listeners
    /// `App::on_event(nom)`. Une garde anti-boucle limite la profondeur.
    pub fn emit<T: Serialize>(&mut self, name: impl Into<String>, data: T) {
        self.emitted.push((name.into(), serde_json::to_value(data).unwrap_or(Value::Null)));
    }

    fn push_update(&mut self, id: String, patch: Value) {
        self.all.push((id.clone(), patch.clone()));
        // Livraison immédiate en temps réel, en plus de la réponse finale.
        self.send(json!({ "t": "update", "u": [json!({ "id": id, "p": patch })] }));
    }

    /// Garde commune aux écritures : hors flux déclaré (`flow_out`) ou marqué
    /// `skip` → la mise à jour est ignorée silencieusement.
    fn blocked_writes(&self, id: &str) -> bool {
        self.skipped.contains(id)
            || self.flow_out.as_ref().is_some_and(|s| !s.contains(id))
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