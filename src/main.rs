#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

#[cfg(not(target_arch = "wasm32"))]
mod cli;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    match cli::main() {
        Ok(Some(mut app)) => {
            app.file_dialog = Some(Box::new(native::file_dialog));
            eframe::run_native(Box::new(app)); // never returns
        }
        Ok(None) => {}
        Err(i) => std::process::exit(i),
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    pub fn file_dialog() -> Option<String> {
        use nfd::Response;
        let result = nfd::open_file_dialog(Some("png,flf"), None).ok()?;

        match result {
            Response::Okay(file_path) => Some(file_path),
            Response::OkayMultiple(files) => files.first().cloned(),
            Response::Cancel => None,
        }
    }
}
