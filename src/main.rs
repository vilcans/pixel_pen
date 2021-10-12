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
    use directories::UserDirs;
    use eframe::epi::IconData;
    use image::{GenericImageView, ImageFormat};
    use native_dialog::{FileDialog, MessageDialog, MessageType};
    use pixel_pen::error::Error;
    use pixel_pen::storage;
    use pixel_pen::system::{OpenFileOptions, SaveFileOptions, SystemFunctions};
    use std::ffi::{OsStr, OsString};
    use std::path::{Path, PathBuf};

    const ICON_IMAGE: &[u8] =
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/assets/icon.png"));

    pub struct NativeSystemFunctions {
        location: PathBuf,
        filename: String,
    }

    impl NativeSystemFunctions {
        pub fn new() -> Self {
            Self {
                location: PathBuf::default(),
                filename: String::default(),
            }
        }

        fn create_file_dialog(
            &mut self,
            initial_path: Option<&Path>,
            include_native: bool,
            include_images: bool,
        ) -> FileDialog<'_> {
            let dialog = FileDialog::new();
            let (location, filename) = directory_and_file_or_default(initial_path);
            let mut dialog = self.set_default(dialog, location, filename);
            if include_native {
                dialog = dialog
                    .add_filter("Pixel Pen Image", &[storage::NATIVE_EXTENSION])
                    .add_filter("Turbo Rascal FLUFF", &["flf"]);
            }
            if include_images {
                dialog = dialog.add_filter(
                    "Image",
                    &["png", "jpg", "jpeg", "gif", "bmp", "tif", "tiff"],
                );
            }
            dialog
        }

        fn set_default<'a>(
            &'a mut self,
            mut dialog: FileDialog<'a>,
            location: Option<PathBuf>,
            filename: Option<OsString>,
        ) -> FileDialog<'a> {
            if let Some(location) = location {
                self.location = location;
                dialog = dialog.set_location(&self.location);
            }
            if let Some(filename) = filename {
                self.filename = filename.to_string_lossy().to_string();
                dialog = dialog.set_filename(&self.filename);
            }
            dialog
        }
    }

    impl SystemFunctions for NativeSystemFunctions {
        fn has_open_file_dialog(&self) -> bool {
            true
        }

        fn has_save_file_dialog(&self) -> bool {
            true
        }

        fn open_file_dialog(
            &mut self,
            options: OpenFileOptions<'_>,
        ) -> Result<Option<PathBuf>, Error> {
            let dialog = self.create_file_dialog(
                options.initial_path.as_deref(),
                options.include_native,
                options.include_images,
            );
            let path = dialog
                .show_open_single_file()
                .map_err(|e| Error::FileDialogError(format!("File dialog failed: {0}", e)))?;
            Ok(path)
        }

        fn save_file_dialog(
            &mut self,
            options: SaveFileOptions<'_>,
        ) -> Result<Option<PathBuf>, Error> {
            let dialog = self.create_file_dialog(
                options.initial_path,
                options.include_native,
                options.include_images,
            );
            let path = dialog
                .show_save_single_file()
                .map_err(|e| Error::FileDialogError(format!("File dialog failed: {0}", e)))?;

            match path {
                Some(filename) if filename.extension().is_none() => {
                    Ok(Some(filename.with_extension(options.default_extension)))
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

    /// Get directory and filename from the path `default`,
    /// or the user's Documents folder if `default` is `None`.
    fn directory_and_file_or_default(
        default: Option<&Path>,
    ) -> (Option<PathBuf>, Option<OsString>) {
        let (location, filename) = default
            .map(|path| {
                (
                    path.parent().map(Path::to_path_buf),
                    path.file_name().map(OsStr::to_os_string),
                )
            })
            .unwrap_or_else(|| {
                if let Some(user_dirs) = UserDirs::new() {
                    let dir = user_dirs.document_dir().map(|d| d.to_owned());
                    (dir, None)
                } else {
                    (None, None)
                }
            });
        (location, filename)
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
