use crate::error::Error;
use std::path::PathBuf;

pub struct OpenFileOptions {
    pub include_native: bool,
    pub include_images: bool,
}
impl OpenFileOptions {
    pub fn for_open() -> Self {
        Self {
            include_native: true,
            include_images: true,
        }
    }
    pub fn for_import() -> Self {
        Self {
            include_native: false,
            include_images: true,
        }
    }
}

pub trait SystemFunctions {
    fn has_open_file_dialog(&self) -> bool;
    fn has_save_file_dialog(&self) -> bool;
    fn open_file_dialog(&mut self, options: OpenFileOptions) -> Result<Option<PathBuf>, Error>;
    fn save_file_dialog(&mut self, default_extension: &str) -> Result<Option<PathBuf>, Error>;
    fn show_error(&self, message: &str) {
        eprintln!("{}\n", message);
    }
}

pub struct DummySystemFunctions;

impl SystemFunctions for DummySystemFunctions {
    fn has_open_file_dialog(&self) -> bool {
        false
    }
    fn has_save_file_dialog(&self) -> bool {
        false
    }
    fn open_file_dialog(&mut self, _options: OpenFileOptions) -> Result<Option<PathBuf>, Error> {
        panic!("No open_file_dialog");
    }
    fn save_file_dialog(&mut self, _default_extension: &str) -> Result<Option<PathBuf>, Error> {
        panic!("No save_file_dialog");
    }
}
