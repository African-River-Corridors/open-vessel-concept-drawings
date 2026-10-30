//! The shell state and the hull views, shared by the desktop app (Tauri) and the web build
//! (WebAssembly). No Tauri types, no calculation: every answer comes from `ovd-format`.

pub mod state;
mod vessel;
pub mod views;

pub use state::AppState;
