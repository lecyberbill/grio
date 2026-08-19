//! # grio
//!
//! Un équivalent minimal de *Gradio*, écrit en Rust : on déclare des
//! composants, on branche un handler, et un serveur web + une API REST
//! apparaissent automatiquement.
//!
//! ```no_run
//! use grio::*;
//!
//! # fn main() -> grio::Result<()> {
//! App::new("Ma démo")
//!     .item(Text::new("name").label("Name"))
//!     .item(Slider::new("n").min(0.0).max(10.0).step(1.0))
//!     .item(Output::new("greet").label("Sortie"))
//!     .on_submit(|ctx| {
//!         let name: String = ctx.get("name")?;
//!         let n: f64 = ctx.get("n")?;
//!         ctx.set("greet", format!("Hello {name} ×{n}"));
//!         Ok(())
//!     })
//!     .launch("127.0.0.1:7860")
//! }
//! ```
//!
//! ## Document de référence
//!
//! Le manuel d'utilisation complet se trouve à la racine de ce workspace :
//! [`README.md`](https://github.com/example/grio#readme).
#![warn(missing_docs)]

pub mod app;
pub mod components;
pub mod context;
pub mod events;
pub mod media;
pub mod server;

pub use app::{App, Theme, ThemeMode};
pub use components::{
    Accordion, Audio, Button, ChatMessage, Chatbot, Checkbox, Code, Column, Component, Dataframe,
    DatePicker, Dropdown, Explorer, Gallery, Grid, Image, ImageEditor, IntoBox, Layout, Markdown,
    Metric, Output, Panel, Plot, Progress, Role, Row, SectionBuilder, Slider, SortableList, Tabs,
    Text, TimePicker, Video, WithLayout,
};
pub use context::{AlertLevel, Context};
pub use events::EventName;
pub use media::{MediaInfo, StreamInfo};

/// Erreur levée par le moteur (contenu libre, généralement une chaîne).
pub type Error = Box<dyn std::error::Error + Send + Sync>;
/// Résultat standard du moteur.
pub type Result<T> = std::result::Result<T, Error>;