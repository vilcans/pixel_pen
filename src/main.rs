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
            app.open_file_dialog = Some(Box::new(native::open_file_dialog));
            app.save_file_dialog = Some(Box::new(native::save_file_dialog));
            eframe::run_native(Box::new(app)); // never returns
        }
        Ok(None) => {}
        Err(i) => std::process::exit(i),
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use pixel_pen::error::Error;
    const OPEN_FILE_TYPES: &str = "pixelpen,png,flf";
    const SAVE_FILE_TYPES: &str = "pixelpen";

    pub fn open_file_dialog() -> Result<Option<String>, Error> {
        let res = nfd::open_file_dialog(Some(OPEN_FILE_TYPES), None);
        map_result(&res)
    }

    pub fn save_file_dialog() -> Result<Option<String>, Error> {
        let res = nfd::open_save_dialog(Some(SAVE_FILE_TYPES), None);
        map_result(&res)
    }

    fn map_result(result: &nfd::Result<nfd::Response>) -> Result<Option<String>, Error> {
        use nfd::Response;
        match result {
            Ok(r) => Ok(match r {
                Response::Okay(file_path) => Some(file_path.clone()),
                Response::OkayMultiple(files) => files.first().cloned(),
                Response::Cancel => None,
            }),
            Err(e) => Err(Error::InternalError(format!(
                "Failed to open dialog: {}",
                e
            ))),
        }
    }
}
