#![warn(clippy::all, rust_2018_idioms)]

mod app;
mod audio_player;
#[cfg(not(target_arch = "wasm32"))]
mod file_dialog;
mod func_edit;
mod player_state;
mod preference;
#[cfg(not(target_arch = "wasm32"))]
mod save_load;
mod state;
mod ui;
pub use app::MainApp;
