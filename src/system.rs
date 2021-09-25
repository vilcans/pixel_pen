use crate::error::Error;
use std::path::{Path, PathBuf};

pub struct OpenFileOptions<'a> {
    pub include_native: bool,
    pub include_images: bool,
    pub initial_path: Option<&'a Path>,
}
impl<'a> OpenFileOptions<'a> {
    pub fn for_open(initial_path: Option<&'a Path>) -> Self {
        Self {
            include_native: true,
            include_images: true,
            initial_path,
        }
    }
    pub fn for_import(initial_path: Option<&'a Path>) -> Self {
        Self {
            include_native: false,
            include_images: true,
            initial_path,
        }
    }
}

pub trait SystemFunctions {
    fn has_open_file_dialog(&self) -> bool;
    fn has_save_file_dialog(&self) -> bool;
    fn open_file_dialog(&mut self, options: OpenFileOptions<'_>) -> Result<Option<PathBuf>, Error>;
    fn save_file_dialog(
        &mut self,
        default: Option<&Path>,
        default_extension: &str,
    ) -> Result<Option<PathBuf>, Error>;
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
    fn open_file_dialog(
        &mut self,
        _options: OpenFileOptions<'_>,
    ) -> Result<Option<PathBuf>, Error> {
        panic!("No open_file_dialog");
    }
    fn save_file_dialog(
        &mut self,
        _default: Option<&Path>,
        _default_extension: &str,
    ) -> Result<Option<PathBuf>, Error> {
        panic!("No save_file_dialog");
    }
}
