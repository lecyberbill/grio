//! Modèle de données du cycle d'événements client ↔ serveur.

use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

/// Nom logique d'un événement.
///
/// * `Change` — modification d'une entrée (texte tapé, curseur bougé).
/// * `Click` — clic sur un composant (principalement un bouton).
/// * `Submit` — soumission globale (bouton Run, ou appel `/api/predict`).
/// * `Load` — montage de la page (émis par le client à l'ouverture du WS).
/// * `Play` / `Pause` / `Stop` — transport média (lecteur audio/vidéo).
/// * `Stream` — fragment d'un flux live (micro/caméra, message `{t:"stream"}`).
/// * `Custom` — événement applicatif distribué en interne via `ctx.emit`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventName {
    /// Valeur d'une entrée modifiée.
    Change,
    /// Clic utilisateur sur un composant.
    Click,
    /// Soumission de l'application (Run / API).
    Submit,
    /// Montage de la page cliente.
    Load,
    /// Lecture démarrée sur un média.
    Play,
    /// Lecture en pause sur un média.
    Pause,
    /// Lecture arrêtée sur un média.
    Stop,
    /// Fragment de flux streaming reçu.
    Stream,
    /// Événement applicatif arbitraire (bus interne).
    Custom(String),
}

impl EventName {
    /// Parse un nom d'événement venant du client (fil d'événements WebSocket).
    pub fn parse(s: &str) -> EventName {
        match s {
            "change" => EventName::Change,
            "click" => EventName::Click,
            "submit" => EventName::Submit,
            "load" => EventName::Load,
            "play" => EventName::Play,
            "pause" => EventName::Pause,
            "stop" => EventName::Stop,
            "stream" => EventName::Stream,
            other => EventName::Custom(other.to_string()),
        }
    }
}

/// Message brut reçu depuis le client (WebSocket).
///
/// Champs : `t` (type), `c` (composant cible), `e` (événement), `d` (données
/// d'événement), `v` (instantané des valeurs d'entrées).
#[derive(Debug, Clone, Deserialize)]
pub struct WireEvent {
    /// Type de message (`"event"` pour un événement applicatif).
    pub t: String,
    /// Identifiant du composant émetteur.
    pub c: String,
    /// Nom d'événement brut (`change`, `click`, …).
    pub e: String,
    /// Données d'événement optionnelles.
    pub d: Option<Value>,
    /// Instantané des valeurs des composants d'entrée.
    pub v: HashMap<String, Value>,
}