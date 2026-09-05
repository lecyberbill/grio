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
    Mock {
        /// Utilisateurs de test pré-configurés (username -> Profile).
        users: Vec<UserProfile>,
    },
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
}

fn uuid_placeholder() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("{nanos:x}")
}
