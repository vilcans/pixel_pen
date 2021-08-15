#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

#[cfg(not(target_arch = "wasm32"))]
mod cli;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use native::NativeSystemFunctions;

    match cli::main() {
        Ok(Some(mut app)) => {
            app.system = Box::new(NativeSystemFunctions::new());
            let options = eframe::NativeOptions::default();
            eframe::run_native(Box::new(app), options); // never returns
        }
        Ok(None) => {}
        Err(i) => std::process::exit(i),
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use native_dialog::{FileDialog, MessageDialog, MessageType};
    use pixel_pen::error::Error;
    use pixel_pen::system::SystemFunctions;
    use std::path::PathBuf;

    pub struct NativeSystemFunctions {}

    impl NativeSystemFunctions {
        pub fn new() -> Self {
            Self {}
        }
    }

    impl SystemFunctions for NativeSystemFunctions {
        fn has_open_file_dialog(&self) -> bool {
            true
        }

        fn has_save_file_dialog(&self) -> bool {
            true
        }

        fn open_file_dialog(&mut self) -> Result<Option<PathBuf>, Error> {
            let path = FileDialog::new()
                //.set_location("~/Desktop")
                .add_filter("Pixel Pen Image", &["pixelpen"])
                .add_filter("Turbo Rascal FLUFF", &["flf"])
                .add_filter("PNG Image", &["png"])
                .add_filter("JPEG Image", &["jpg", "jpeg"])
                .show_open_single_file()
                .map_err(|e| Error::FileDialogError(format!("File dialog failed: {0}", e)))?;
            Ok(path)
        }

        fn save_file_dialog(&mut self, default_extension: &str) -> Result<Option<PathBuf>, Error> {
            let path = FileDialog::new()
                //.set_location("~/Desktop")
                .add_filter("Pixel Pen Image", &["pixelpen"])
                .show_save_single_file()
                .map_err(|e| Error::FileDialogError(format!("File dialog failed: {0}", e)))?;

            match path {
                Some(filename) if filename.extension().is_none() => {
                    Ok(Some(filename.with_extension(default_extension)))
                }
                p => Ok(p),
            }
        }

        fn show_error(&self, message: &str) {
            match MessageDialog::new()
                .set_type(MessageType::Error)
                .set_title("Error")
                .set_text(message)
                .show_alert()
            {
                Err(e) => {
                    eprintln!("Failed to show error message \"{0}\": {1}", message, e);
                }
                Ok(()) => {}
            }
        }
    }
}
