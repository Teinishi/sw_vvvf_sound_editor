#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod audio_player;
mod editable_function;
#[cfg(not(target_arch = "wasm32"))]
mod file_dialog;
mod player_state;
mod preference;
mod state;
mod ui;
pub use app::MainApp;
