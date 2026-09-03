//! Showcase example — One-line component showcase launcher
//!
//! Run: `cargo run -p grio --example showcase`

fn main() -> grio::Result<()> {
    grio::App::showcase().launch("0.0.0.0:7860")
}
