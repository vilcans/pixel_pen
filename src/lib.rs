#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

mod actions;
mod app;
mod brush;
mod cell_image;
mod colors;
mod coords;
mod document;
mod editor;
mod egui_extensions;
pub mod error;
mod image_io;
mod image_operations;
mod import;
mod line;
mod mode;
mod mutation_monitor;
pub mod storage;
pub mod system;
mod texture;
mod tool;
mod ui;
mod update_area;
mod vic;
mod widgets;
pub use app::Application;
pub use document::Document;

// ----------------------------------------------------------------------------
// When compiling for web:

#[cfg(target_arch = "wasm32")]
use eframe::wasm_bindgen::{self, prelude::*};

/// This is the entry-point for all the web-assembly.
/// This is called once from the HTML.
/// It loads the app, installs some callbacks, then returns.
/// You can add more callbacks like this if you want to call in to your code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn start(canvas_id: &str) -> Result<(), eframe::wasm_bindgen::JsValue> {
    let app = Application::default();
    eframe::start_web(canvas_id, Box::new(app))
}
