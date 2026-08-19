//! Utilitaires média : inspection des data URLs (type, taille, dimensions) et
//! statistiques des flux de streaming (micro/caméra).
//!
//! Les composants `Image`/`Audio`/`Video` transportent leurs données comme
//! **data URLs** (`data:<mime>;base64,…`), pageable par `ctx.get::<String>`.
//! `inspect` décode l'en-tête pour en extraire le type, la taille en octets
//! et les dimensions (PNG, JPEG, GIF — les autres: `None`).

use base64::Engine;
use serde::{Deserialize, Serialize};

/// Informations extraites d'une data URL média.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MediaInfo {
    /// Type MIME (ex. `image/png`).
    pub mime: String,
    /// Taille du contenu décodé, en octets.
    pub size_bytes: usize,
    /// Largeur en pixels, si détectable (PNG/JPEG/GIF).
    pub width: Option<u32>,
    /// Hauteur en pixels, si détectable (PNG/JPEG/GIF).
    pub height: Option<u32>,
}

impl MediaInfo {
    /// Famille de contenu (`image`, `audio`, `video`).
    pub fn kind(&self) -> &str {
        self.mime.split('/').next().unwrap_or("")
    }

    /// Judgée affichable sous forme d'un résumé court et lisible.
    pub fn description(&self) -> String {
        let dims = match (self.width, self.height) {
            (Some(w), Some(h)) => format!("{w}×{h} px"),
            _ => "dimensions inconnues".to_string(),
        };
        format!("{} · {} · {} o", self.mime, dims, self.size_bytes)
    }
}

/// Statistiques d'un flux reçu en streaming (`{t:"stream"}`) pour un composant.
///
/// Sérialisé dans les valeurs partagées du serveur : un handler branché sur
/// l'événement `"stream"` peut le lire via `ctx.get::<StreamInfo>(id)`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct StreamInfo {
    /// Type MIME du flux (ex. `audio/webm`).
    pub mime: String,
    /// Octets reçus au total.
    pub bytes: u64,
    /// Fragments reçus.
    pub chunks: u64,
}

impl StreamInfo {
    /// Octets reçus, arrondis en kilo-octets.
    pub fn kb(&self) -> f64 {
        self.bytes as f64 / 1024.0
    }
}

/// Sépare une data URL en `(mime, données_base64)`.
pub fn split_data_url(data_url: &str) -> Option<(String, String)> {
    let rest = data_url.strip_prefix("data:")?;
    let (meta, data) = rest.split_once(',')?;
    let mime = meta.split(';').next().unwrap_or("application/octet-stream").to_string();
    Some((mime, data.to_string()))
}

/// Inspecte une data URL : type, taille et dimensions (quand détectable).
pub fn inspect(data_url: &str) -> Option<MediaInfo> {
    let (mime, data) = split_data_url(data_url)?;
    let bytes = decode(&data)?;
    let size = bytes.len();
    if size == 0 {
        return None;
    }
    let (width, height) = dimensions(&mime, &bytes);
    Some(MediaInfo { mime, size_bytes: size, width, height })
}

/// Décode des données base64 (URL-safe accepté).
pub fn decode(b64: &str) -> Option<Vec<u8>> {
    base64::engine::general_purpose::STANDARD
        .decode(b64)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(b64))
        .ok()
}

/// Dimensions détectées dans l'en-tête. Non-détectable → `(None, None)`.
fn dimensions(mime: &str, b: &[u8]) -> (Option<u32>, Option<u32>) {
    match mime {
        "image/png" if b.len() >= 24 && &b[0..8] == b"\x89PNG\r\n\x1a\n" => (
            Some(u32::from_be_bytes([b[16], b[17], b[18], b[19]])),
            Some(u32::from_be_bytes([b[20], b[21], b[22], b[23]])),
        ),
        "image/gif" if b.len() >= 10 && &b[0..3] == b"GIF" => (
            Some(u16::from_le_bytes([b[6], b[7]]) as u32),
            Some(u16::from_le_bytes([b[8], b[9]]) as u32),
        ),
        "image/jpeg" => jpeg_dimensions(b),
        _ => (None, None),
    }
}

/// Parcourt les segments JPEG à la recherche d'un cadre (SOF) pour les
/// dimensions.
fn jpeg_dimensions(b: &[u8]) -> (Option<u32>, Option<u32>) {
    if b.len() < 4 || b[0] != 0xFF || b[1] != 0xD8 {
        return (None, None);
    }
    let mut i = 2usize;
    while i + 9 < b.len() {
        if b[i] != 0xFF {
            i += 1;
            continue;
        }
        let marker = b[i + 1];
        let sof = marker >= 0xC0
            && marker <= 0xCF
            && !matches!(marker, 0xC4 | 0xC8 | 0xCC);
        if sof {
            let h = u16::from_be_bytes([b[i + 5], b[i + 6]]);
            let w = u16::from_be_bytes([b[i + 7], b[i + 8]]);
            return (Some(w as u32), Some(h as u32));
        }
        let len = u16::from_be_bytes([b[i + 2], b[i + 3]]);
        if len < 2 {
            break;
        }
        i += 2 + len as usize;
    }
    (None, None)
}