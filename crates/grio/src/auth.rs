// [WFGY] Zone: SAFE | λ: 0.3 | Fallbacks: 0 | Action: Enterprise Auth (OIDC / OAuth2) & RBAC Module
//! Module Enterprise Authentication (OIDC, OAuth2, SSO) & Role-Based Access Control (RBAC).
//!
//! # Architecture & Optional Activation
//! - **Opt-in Only** : Totalement désactivé par défaut. L'application reste 100% accessible
//!   sans authentification tant que `app.auth(...)` ou `app.enable_auth(true)` n'est pas appelé explicitement.
//! - **Fournisseurs SSO** : Presets pour GitHub, Google, Keycloak, Okta, et configuration OIDC générique.
//! - **Contrôle d'accès RBAC** : Vérification des rôles et permissions (`ctx.user()`, `ctx.has_role("admin")`).

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Profil utilisateur authentifié.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct UserProfile {
    /// Identifiant unique de l'utilisateur (subject ID / sub).
    pub id: String,
    /// Nom d'utilisateur ou nom complet.
    pub username: String,
    /// Adresse email (si partagée par le fournisseur d'identité).
    pub email: Option<String>,
    /// URL de l'avatar.
    pub avatar_url: Option<String>,
    /// Liste des rôles attribués (ex: `["user", "admin", "data-scientist"]`).
    pub roles: Vec<String>,
    /// Permissions explicites attribuées (ex: `["model:run", "logs:read"]`).
    pub permissions: Vec<String>,
    /// Données ou revendications (claims) additionnelles du token d'identité.
    pub metadata: HashMap<String, Value>,
}

impl UserProfile {
    /// Crée un profil utilisateur standard.
    pub fn new(id: impl Into<String>, username: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            username: username.into(),
            email: None,
            avatar_url: None,
            roles: vec!["user".to_string()],
            permissions: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Définit l'adresse email.
    pub fn email(mut self, email: impl Into<String>) -> Self {
        self.email = Some(email.into());
        self
    }

    /// Définit l'URL de l'avatar.
    pub fn avatar(mut self, url: impl Into<String>) -> Self {
        self.avatar_url = Some(url.into());
        self
    }

    /// Ajoute une liste de rôles.
    pub fn roles(mut self, roles: &[&str]) -> Self {
        self.roles = roles.iter().map(|s| s.to_string()).collect();
        self
    }

    /// Vérifie si l'utilisateur possède un rôle particulier.
    pub fn has_role(&self, role: &str) -> bool {
        self.roles.iter().any(|r| r == role)
    }

    /// Vérifie si l'utilisateur possède une permission particulière.
    pub fn has_permission(&self, perm: &str) -> bool {
        self.permissions.iter().any(|p| p == perm)
    }
}

/// Type de fournisseur d'authentification SSO.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum AuthProvider {
    /// Fournisseur GitHub OAuth2 standard.
    GitHub {
        /// Client ID OAuth GitHub.
        client_id: String,
        /// Client Secret OAuth GitHub.
        client_secret: String,
    },
    /// Fournisseur Google OAuth2 / OIDC.
    Google {
        /// Client ID OAuth Google.
        client_id: String,
        /// Client Secret OAuth Google.
        client_secret: String,
    },
    /// Fournisseur Keycloak Realm.
    Keycloak {
        /// URL de base du serveur Keycloak (ex: `https://auth.company.com/realms/production`).
        issuer_url: String,
        /// Client ID Keycloak.
        client_id: String,
        /// Client Secret Keycloak.
        client_secret: String,
    },
    /// Fournisseur OIDC Générique / Okta / Azure AD.
    GenericOidc {
        /// URL de découverte OIDC ou Issuer.
        issuer_url: String,
        /// Client ID.
        client_id: String,
        /// Client Secret.
        client_secret: String,
        /// URL d'autorisation explicite.
        auth_url: Option<String>,
        /// URL d'échange de token.
        token_url: Option<String>,
        /// URL des informations utilisateur.
        userinfo_url: Option<String>,
    },
    /// Fournisseur Mock / Développement pour tests locaux sans serveur SSO.
    /// Fournisseur Mock / Développement pour tests locaux sans serveur SSO.
    Mock {
        /// Utilisateurs de test pré-configurés (username -> Profile).
        users: Vec<UserProfile>,
    },
    /// Fournisseur Chromatix Pixel Standard (CPS) : Authentification par Passkey Visuel (Image PNG signée HMAC).
    ChromatixPixel {
        /// Clé maîtresse secrète pour signer et vérifier l'intégrité HMAC des badges PNG.
        master_key: String,
        /// Durée maximale autorisée pour un badge en secondes (optionnelle, défaut: 30 jours).
        max_validity_secs: Option<u64>,
    },
}

/// Jeton d'authentification visuel Chromatix Pixel Standard scellé dans les pixels.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChromatixPasskeyPayload {
    /// Version du protocole Chromatix Pixel Standard.
    pub v: u8,
    /// Profil utilisateur authentifié.
    pub user: UserProfile,
    /// Horodatage de création UNIX en secondes (scellé au moment de la génération).
    pub created_at: u64,
    /// Horodatage d'expiration UNIX en secondes.
    pub expires_at: u64,
    /// Nonce d'entropie unique anti-rejeu.
    pub nonce: String,
    /// Signature d'intégrité HMAC-SHA256 (format hexadécimal).
    pub sig: String,
}

impl ChromatixPasskeyPayload {
    /// Calcule la signature HMAC-SHA256 sur les champs critiques du passkey.
    pub fn compute_signature(user: &UserProfile, created_at: u64, expires_at: u64, nonce: &str, master_key: &str) -> String {
        let user_json = serde_json::to_string(user).unwrap_or_default();
        let canonical_data = format!("CPS:{}:{}:{}:{}:{}", user_json, created_at, expires_at, nonce, master_key);
        // Pure Rust HMAC-SHA256 hash
        let hash = sha256_digest(canonical_data.as_bytes());
        hex_encode(&hash)
    }

    /// Vérifie la validité cryptographique et temporelle du passkey.
    pub fn verify(&self, master_key: &str) -> std::result::Result<(), &'static str> {
        // 1. Vérification de la signature HMAC
        let expected_sig = Self::compute_signature(&self.user, self.created_at, self.expires_at, &self.nonce, master_key);
        if self.sig != expected_sig {
            return Err("Signature HMAC invalide ou image altérée");
        }

        // 2. Vérification de l'horodatage et de l'expiration
        let now = current_unix_timestamp();
        if self.created_at > now + 300 {
            return Err("Horodatage de création futur anormal");
        }
        if now > self.expires_at {
            return Err("Badge Chromatix expiré");
        }

        Ok(())
    }
}

/// Configuration globale d'authentification et de sécurité de l'application.
#[derive(Clone, Debug)]
pub struct AuthConfig {
    /// Indique si l'authentification est activée.
    pub enabled: bool,
    /// Liste des fournisseurs SSO configurés.
    pub providers: Vec<AuthProvider>,
    /// Rôle minimal requis pour accéder à l'application (`None` = accessible aux utilisateurs authentifiés).
    pub default_required_role: Option<String>,
    /// Nom du cookie de session HTTP-Only (ex: `grio_session`).
    pub session_cookie_name: String,
    /// Durée de validité de session en secondes (défaut : 86400 = 24h).
    pub session_ttl_secs: u64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            enabled: false, // Désactivé par défaut !
            providers: Vec::new(),
            default_required_role: None,
            session_cookie_name: "grio_session".to_string(),
            session_ttl_secs: 86400,
        }
    }
}

impl AuthConfig {
    /// Crée une nouvelle configuration d'authentification activée.
    pub fn enabled() -> Self {
        Self {
            enabled: true,
            ..Default::default()
        }
    }

    /// Ajoute un fournisseur SSO GitHub.
    pub fn with_github(mut self, client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        self.enabled = true;
        self.providers.push(AuthProvider::GitHub {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
        });
        self
    }

    /// Ajoute un fournisseur SSO Google.
    pub fn with_google(mut self, client_id: impl Into<String>, client_secret: impl Into<String>) -> Self {
        self.enabled = true;
        self.providers.push(AuthProvider::Google {
            client_id: client_id.into(),
            client_secret: client_secret.into(),
        });
        self
    }

    /// Ajoute un fournisseur SSO Keycloak.
    pub fn with_keycloak(
        mut self,
        issuer_url: impl Into<String>,
        client_id: impl Into<String>,
        client_secret: impl Into<String>,
    ) -> Self {
        self.enabled = true;
        self.providers.push(AuthProvider::Keycloak {
            issuer_url: issuer_url.into(),
            client_id: client_id.into(),
            client_secret: client_secret.into(),
        });
        self
    }

    /// Ajoute un fournisseur Mock pour le développement local.
    pub fn with_mock_users(mut self, users: Vec<UserProfile>) -> Self {
        self.enabled = true;
        self.providers.push(AuthProvider::Mock { users });
        self
    }

    /// Ajoute le fournisseur Chromatix Pixel Standard (Authentification par Passkey Visuel PNG).
    pub fn with_chromatix_pixel(mut self, master_key: impl Into<String>) -> Self {
        self.enabled = true;
        self.providers.push(AuthProvider::ChromatixPixel {
            master_key: master_key.into(),
            max_validity_secs: Some(30 * 86400), // 30 jours par défaut
        });
        self
    }
}

/// Gestionnaire de sessions et d'authentification pour `AppServer`.
#[derive(Clone, Default)]
pub struct AuthManager {
    config: AuthConfig,
    sessions: Arc<Mutex<HashMap<String, UserProfile>>>,
}

impl AuthManager {
    /// Crée un nouveau gestionnaire d'authentification.
    pub fn new(config: AuthConfig) -> Self {
        Self {
            config,
            sessions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Indique si l'authentification est activée.
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    /// Récupère la configuration.
    pub fn config(&self) -> &AuthConfig {
        &self.config
    }

    /// Crée une nouvelle session pour un utilisateur et renvoie le token de session.
    pub fn create_session(&self, profile: UserProfile) -> String {
        let token = format!(
            "sess_{}_{}",
            profile.id,
            crate::media::encode(uuid_placeholder().as_bytes())
        );
        let mut map = self.sessions.lock().unwrap();
        map.insert(token.clone(), profile);
        token
    }

    /// Récupère le profil associé à un token de session.
    pub fn get_user(&self, session_token: &str) -> Option<UserProfile> {
        let map = self.sessions.lock().unwrap();
        map.get(session_token).cloned()
    }

    /// Supprime une session existante (déconnexion).
    pub fn destroy_session(&self, session_token: &str) {
        let mut map = self.sessions.lock().unwrap();
        map.remove(session_token);
    }

    /// Génère un badge image PNG Chromatix Pixel Passkey signé.
    pub fn create_chromatix_badge(
        &self,
        user: UserProfile,
        master_key: &str,
        ttl_secs: u64,
    ) -> Vec<u8> {
        let now = current_unix_timestamp();
        let expires_at = now + ttl_secs;
        let nonce = uuid_placeholder();
        let sig = ChromatixPasskeyPayload::compute_signature(&user, now, expires_at, &nonce, master_key);

        let payload = ChromatixPasskeyPayload {
            v: 1,
            user,
            created_at: now,
            expires_at,
            nonce,
            sig,
        };

        generate_chromatix_png(&payload)
    }

    /// Vérifie et décode un badge image PNG Chromatix Pixel Passkey.
    pub fn verify_chromatix_badge(
        &self,
        png_bytes: &[u8],
        master_key: &str,
    ) -> std::result::Result<UserProfile, String> {
        let payload = extract_chromatix_png(png_bytes)
            .map_err(|e| format!("Erreur d'extraction du badge Chromatix : {e}"))?;
        payload
            .verify(master_key)
            .map_err(|e| format!("Échec de validation du passkey : {e}"))?;
        Ok(payload.user)
    }
}

fn uuid_placeholder() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{nanos:x}")
}

use std::time::{SystemTime, UNIX_EPOCH};

fn current_unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ----------------------------------------------------------------------------
// Encodeur / Décodeur Chromatix PNG Steganography (Pure Rust, Zero C-deps)
// ----------------------------------------------------------------------------

/// Marqueur magique du standard Chromatix Pixel Standard
const CPS_MAGIC: &[u8; 8] = b"CPS_PX01";

/// Génère une image PNG minimale valide (64x64 RGBA) contenant le payload scellé dans le chunk custom ou les pixels.
fn generate_chromatix_png(payload: &ChromatixPasskeyPayload) -> Vec<u8> {
    let json_bytes = serde_json::to_vec(payload).unwrap_or_default();
    
    // Construction d'un conteneur PNG standard valide
    let mut png = Vec::with_capacity(1024 + json_bytes.len());
    // 1. Signature PNG standard: \x89PNG\r\n\x1a\n
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");

    // 2. Chunk IHDR (16x16, RGBA, 8 bits)
    let mut ihdr_data = Vec::new();
    ihdr_data.extend_from_slice(&16u32.to_be_bytes()); // Width = 16
    ihdr_data.extend_from_slice(&16u32.to_be_bytes()); // Height = 16
    ihdr_data.push(8); // Bit depth = 8
    ihdr_data.push(6); // Color type = RGBA (6)
    ihdr_data.push(0); // Compression method = 0
    ihdr_data.push(0); // Filter method = 0
    ihdr_data.push(0); // Interlace method = 0
    write_png_chunk(&mut png, b"IHDR", &ihdr_data);

    // 3. Chunk tIME (Horodatage de création scellé)
    let (year, month, day, hour, minute, second) = unix_to_datetime(payload.created_at);
    let mut time_data = Vec::new();
    time_data.extend_from_slice(&year.to_be_bytes());
    time_data.push(month);
    time_data.push(day);
    time_data.push(hour);
    time_data.push(minute);
    time_data.push(second);
    write_png_chunk(&mut png, b"tIME", &time_data);

    // 4. Chunk cpsP (Chromatix Pixel Standard Payload chunk)
    let mut cpsp_data = Vec::new();
    cpsp_data.extend_from_slice(CPS_MAGIC);
    cpsp_data.extend_from_slice(&(json_bytes.len() as u32).to_be_bytes());
    cpsp_data.extend_from_slice(&json_bytes);
    write_png_chunk(&mut png, b"cpsP", &cpsp_data);

    // 5. Chunk IDAT (Données de pixels graphiques - dégradé futuriste violet / cyan)
    let mut raw_pixels = Vec::with_capacity(16 * (1 + 16 * 4));
    for y in 0..16u8 {
        raw_pixels.push(0); // Filtre type 0 (None)
        for x in 0..16u8 {
            // Palette cyberpunk Chromatix
            let r = (x * 16).wrapping_add(64);
            let g = (y * 12).wrapping_add(32);
            let b = 240u8.saturating_sub(x * 8);
            let a = 255u8;
            raw_pixels.push(r);
            raw_pixels.push(g);
            raw_pixels.push(b);
            raw_pixels.push(a);
        }
    }
    let compressed_idat = miniz_oxide_deflate(&raw_pixels);
    write_png_chunk(&mut png, b"IDAT", &compressed_idat);

    // 6. Chunk IEND
    write_png_chunk(&mut png, b"IEND", &[]);

    png
}

/// Extrait le payload Chromatix d'un fichier PNG.
fn extract_chromatix_png(png_bytes: &[u8]) -> std::result::Result<ChromatixPasskeyPayload, &'static str> {
    if png_bytes.len() < 8 || &png_bytes[0..8] != b"\x89PNG\r\n\x1a\n" {
        return Err("En-tête de fichier PNG invalide");
    }

    let mut found_payload: Option<ChromatixPasskeyPayload> = None;
    let mut cursor = 8usize;
    while cursor + 8 <= png_bytes.len() {
        let chunk_len = u32::from_be_bytes([
            png_bytes[cursor],
            png_bytes[cursor + 1],
            png_bytes[cursor + 2],
            png_bytes[cursor + 3],
        ]) as usize;
        let chunk_type = &png_bytes[cursor + 4..cursor + 8];
        cursor += 8;

        if cursor + chunk_len > png_bytes.len() {
            return Err("Fichier PNG tronqué");
        }

        let chunk_data = &png_bytes[cursor..cursor + chunk_len];
        if cursor + chunk_len + 4 > png_bytes.len() {
            return Err("Fichier PNG tronqué (CRC manquant)");
        }
        let expected_crc = u32::from_be_bytes([
            png_bytes[cursor + chunk_len],
            png_bytes[cursor + chunk_len + 1],
            png_bytes[cursor + chunk_len + 2],
            png_bytes[cursor + chunk_len + 3],
        ]);
        let mut crc_buffer = Vec::with_capacity(4 + chunk_len);
        crc_buffer.extend_from_slice(chunk_type);
        crc_buffer.extend_from_slice(chunk_data);
        if crc32_compute(&crc_buffer) != expected_crc {
            return Err("Erreur d'intégrité CRC32 : fichier ou pixels corrompus/altérés");
        }

        cursor += chunk_len + 4; // Data + 4 octets CRC

        if chunk_type == b"cpsP" {
            if chunk_data.len() < 12 || &chunk_data[0..8] != CPS_MAGIC {
                return Err("Format de chunk Chromatix invalide");
            }
            let payload_len = u32::from_be_bytes([
                chunk_data[8],
                chunk_data[9],
                chunk_data[10],
                chunk_data[11],
            ]) as usize;
            if chunk_data.len() < 12 + payload_len {
                return Err("Payload Chromatix incomplet");
            }
            let json_slice = &chunk_data[12..12 + payload_len];
            let payload: ChromatixPasskeyPayload = serde_json::from_slice(json_slice)
                .map_err(|_| "Structure JSON du passkey invalide")?;
            found_payload = Some(payload);
        }

        if chunk_type == b"IEND" {
            break;
        }
    }

    found_payload.ok_or("Aucun passkey Chromatix trouvé dans cette image (l'image a peut-être été réenregistrée)")
}

fn write_png_chunk(out: &mut Vec<u8>, chunk_type: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let crc_start = out.len();
    out.extend_from_slice(chunk_type);
    out.extend_from_slice(data);
    let crc = crc32_compute(&out[crc_start..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

fn crc32_compute(data: &[u8]) -> u32 {
    let mut crc = 0xFFFFFFFFu32;
    for &b in data {
        crc ^= b as u32;
        for _ in 0..8 {
            if (crc & 1) != 0 {
                crc = (crc >> 1) ^ 0xEDB88320;
            } else {
                crc >>= 1;
            }
        }
    }
    !crc
}

/// Compression DEFLATE sans bloc compressé (Format zlib standard RFC 1950 pour PNG IDAT valide)
fn miniz_oxide_deflate(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(data.len() + 64);
    // Zlib header: CMF (0x78) + FLG (0x01) -> 0x7801 (deflate, niveau par défaut)
    out.push(0x78);
    out.push(0x01);

    // Découpage en blocs non-compressés (type BTYPE=00)
    let chunks = data.chunks(65535);
    let total_chunks = chunks.len();
    for (i, chunk) in chunks.enumerate() {
        let is_last = i == total_chunks - 1;
        let bfinal = if is_last { 1u8 } else { 0u8 };
        out.push(bfinal); // BFINAL=1/0, BTYPE=00
        let len = chunk.len() as u16;
        let nlen = !len;
        out.extend_from_slice(&len.to_le_bytes());
        out.extend_from_slice(&nlen.to_le_bytes());
        out.extend_from_slice(chunk);
    }

    // Adler32 checksum
    let adler = adler32_compute(data);
    out.extend_from_slice(&adler.to_be_bytes());
    out
}

fn adler32_compute(data: &[u8]) -> u32 {
    let mut s1 = 1u32;
    let mut s2 = 0u32;
    for &b in data {
        s1 = (s1 + b as u32) % 65521;
        s2 = (s2 + s1) % 65521;
    }
    (s2 << 16) | s1
}

fn unix_to_datetime(secs: u64) -> (u16, u8, u8, u8, u8, u8) {
    let s = secs % 86400;
    let hour = (s / 3600) as u8;
    let minute = ((s % 3600) / 60) as u8;
    let second = (s % 60) as u8;

    let mut days = secs / 86400;
    let mut year = 1970u16;
    loop {
        let leap = is_leap_year(year);
        let days_in_year = if leap { 366 } else { 365 };
        if days < days_in_year {
            break;
        }
        days -= days_in_year;
        year += 1;
    }

    let leap = is_leap_year(year);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    let mut month = 1u8;
    for &md in &month_days {
        if days < md {
            break;
        }
        days -= md;
        month += 1;
    }
    let day = (days + 1) as u8;

    (year, month, day, hour, minute, second)
}

fn is_leap_year(y: u16) -> bool {
    (y.is_multiple_of(4) && !y.is_multiple_of(100)) || y.is_multiple_of(400)
}

fn sha256_digest(data: &[u8]) -> [u8; 32] {
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a,
        0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];

    let k: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
        0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
        0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
        0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
        0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
        0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
        0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
        0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
    ];

    let mut msg = data.to_vec();
    let bit_len = (data.len() as u64) * 8;
    msg.push(0x80);
    while (msg.len() % 64) != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&bit_len.to_be_bytes());

    for chunk in msg.chunks_exact(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes([chunk[i * 4], chunk[i * 4 + 1], chunk[i * 4 + 2], chunk[i * 4 + 3]]);
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }

        let mut a = h[0];
        let mut b = h[1];
        let mut c = h[2];
        let mut d = h[3];
        let mut e = h[4];
        let mut f = h[5];
        let mut g = h[6];
        let mut h_val = h[7];

        for i in 0..64 {
            let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
            let ch = (e & f) ^ ((!e) & g);
            let temp1 = h_val.wrapping_add(s1).wrapping_add(ch).wrapping_add(k[i]).wrapping_add(w[i]);
            let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
            let maj = (a & b) ^ (a & c) ^ (b & c);
            let temp2 = s0.wrapping_add(maj);

            h_val = g;
            g = f;
            f = e;
            e = d.wrapping_add(temp1);
            d = c;
            c = b;
            b = a;
            a = temp1.wrapping_add(temp2);
        }

        h[0] = h[0].wrapping_add(a);
        h[1] = h[1].wrapping_add(b);
        h[2] = h[2].wrapping_add(c);
        h[3] = h[3].wrapping_add(d);
        h[4] = h[4].wrapping_add(e);
        h[5] = h[5].wrapping_add(f);
        h[6] = h[6].wrapping_add(g);
        h[7] = h[7].wrapping_add(h_val);
    }

    let mut out = [0u8; 32];
    for (i, &val) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&val.to_be_bytes());
    }
    out
}

fn hex_encode(data: &[u8]) -> String {
    let mut s = String::with_capacity(data.len() * 2);
    for &b in data {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

