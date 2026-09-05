# 💎 Chromatix Pixel Standard (CPS) Authentication in grio

This document details the **Chromatix Visual Passkey Authentication Protocol** integrated natively into `grio`. It provides a cryptographically secure, offline, air-gapped, and laboratory-friendly authentication mechanism using **signed PNG images**.

---

## 1. Overview & Use Cases

Traditional authentication methods (passwords, OAuth2, SSO) rely on identity providers or active network connections. In specialized contexts like **isolated R&D laboratories, defense facilities, air-gapped workstations, or IoT control planes**, external SSO servers are unavailable.

**Chromatix Pixel Standard (CPS) Passkeys** solve this by using an image file (PNG) as an immutable, tamper-evident physical badge:
- **No External Servers**: 100% offline verification using a shared or master secret.
- **Embedded Timestamping**: The exact creation date (`created_at`) and expiration date (`expires_at`) are sealed within the cryptographically signed payload.
- **Tamper & Re-save Proof**: Any attempt to open and re-save the badge in editing software (Paint, Photoshop, GIMP) invalidates the HMAC signature or alters the pixel matrix, immediately rejecting access.
- **Pure Rust**: Zero C-dependencies, built directly into `grio`.

---

## 2. Cryptographic Security Model

### Sealed Payload Structure (`ChromatixPasskeyPayload`)

Each badge contains a signed binary payload embedded into a dedicated chunk (`cpsP` / steganography matrix):

```rust
pub struct ChromatixPasskeyPayload {
    pub v: u8,                   // Protocol Version (CPS v1)
    pub user: UserProfile,       // User identity (id, username, email, roles, metadata)
    pub created_at: u64,         // Exact UNIX creation timestamp
    pub expires_at: u64,         // UNIX expiration timestamp (TTL)
    pub nonce: String,           // Unique entropy nonce (anti-replay)
    pub sig: String,             // HMAC-SHA256 signature (Hex encoded)
}
```

### Signature Calculation
$$\text{Signature} = \text{HMAC-SHA256}(\text{CanonicalData}, \text{MasterKey})$$

Where `CanonicalData` is:
```
CPS:<user_json>:<created_at>:<expires_at>:<nonce>:<master_key>
```

### Integrity Verification Workflow
1. **PNG Structure Check**: Ensures standard PNG magic headers (`\x89PNG\r\n\x1a\n`) and valid chunk ordering.
2. **Strict CRC32 Check**: Every chunk in the file is verified against its 32-bit CRC checksum. If any pixel or metadata byte was modified, the CRC check fails.
3. **HMAC Signature Check**: The signature is recalculated with the server's `master_key` and compared in constant time.
4. **Temporal Expiration Check**: Ensures `created_at <= now <= expires_at`.

---

## 3. How to Enable in `grio`

### Step 1: Configure `App` with `with_chromatix_pixel`

```rust
use grio::auth::AuthConfig;
use grio::App;

const LAB_MASTER_KEY: &str = "super_secret_lab_key_2026";

#[tokio::main]
async fn main() -> grio::Result<()> {
    App::new("🧬 Quantum Lab Workstation")
        .auth(
            AuthConfig::enabled()
                .with_chromatix_pixel(LAB_MASTER_KEY)
        )
        // Protected Admin Page
        .page("/admin", "Admin Panel", |p| {
            p.require_role("admin");
            // ...
        })
        .launch("0.0.0.0:7890")
}
```

### Step 2: Generating a Visual Passkey Badge (Admin Utility)

```rust
use grio::auth::{AuthConfig, AuthManager, UserProfile};

let auth_mgr = AuthManager::new(AuthConfig::enabled().with_chromatix_pixel(LAB_MASTER_KEY));

let alice = UserProfile::new("alice_lead", "Dr. Alice (Lead Scientist)")
    .email("alice@lab.org")
    .roles(&["researcher", "admin"]);

// Generate badge valid for 7 days (604800 seconds)
let badge_png_bytes: Vec<u8> = auth_mgr.create_chromatix_badge(alice, LAB_MASTER_KEY, 604800);

// Save to disk or send to user
std::fs::write("alice_badge.png", badge_png_bytes)?;
```

---

## 4. Interactive Showcase

Run the built-in laboratory kiosk demonstration:

```bash
cargo run -p grio --example chromatix_passkey
```

1. Navigate to `http://localhost:7890`.
2. Download one of the generated badges (**Dr. Alice (Admin)** or **Bob (Technician)**).
3. Click **Sign in** in the top header.
4. Drag and drop the `.png` badge file into the **Chromatix Visual Passkey** dropzone.
5. You are immediately authenticated with the appropriate RBAC roles!
