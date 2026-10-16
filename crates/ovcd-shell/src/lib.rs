//! The shell state and the hull views, shared by the desktop app (Tauri) and the web build
//! (WebAssembly). No Tauri types, no calculation: every answer comes from `ovcd-format`.

pub mod state;
pub mod views;

pub use state::AppState;
