#![forbid(unsafe_code)]
#![cfg_attr(not(debug_assertions), deny(warnings))] // Forbid warnings in release builds
#![warn(clippy::all, rust_2018_idioms)]

#[cfg(not(target_arch = "wasm32"))]
mod cli;

// When compiling natively:
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    use eframe::egui::Vec2;
    use native::NativeSystemFunctions;

    match cli::main() {
        Ok(Some(mut app)) => {
            app.system = Box::new(NativeSystemFunctions::new());
            let options = eframe::NativeOptions {
                icon_data: Some(native::load_icon()),
                initial_window_size: Some(Vec2::new(1280.0, 920.0)),
                ..Default::default()
            };
            eframe::run_native(Box::new(app), options); // never returns
        }
        Ok(None) => {}
        Err(i) => std::process::exit(i),
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use eframe::epi::IconData;
    use image::{GenericImageView, ImageFormat};
    use native_dialog::{FileDialog, MessageDialog, MessageType};
    use pixel_pen::error::Error;
    use pixel_pen::system::{OpenFileOptions, SystemFunctions};
    use std::path::PathBuf;

    const ICON_IMAGE: &[u8] =
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon.png"));

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

        fn open_file_dialog(&mut self, options: OpenFileOptions) -> Result<Option<PathBuf>, Error> {
            let mut dialog = FileDialog::new();
            if options.include_native {
                dialog = dialog
                    .add_filter("Pixel Pen Image", &["pixelpen"])
                    .add_filter("Turbo Rascal FLUFF", &["flf"]);
            }
            if options.include_images {
                dialog = dialog.add_filter(
                    "Image",
                    &["png", "jpg", "jpeg", "gif", "bmp", "tif", "tiff"],
                );
            }
            let path = dialog
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

    pub fn load_icon() -> IconData {
        let image = image::load_from_memory_with_format(ICON_IMAGE, ImageFormat::Png).unwrap();
        let pixels = image.to_rgba8().to_vec();
        IconData {
            rgba: pixels,
            width: image.width(),
            height: image.height(),
        }
    }
}
