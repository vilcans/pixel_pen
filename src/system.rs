use crate::error::Error;
use std::path::PathBuf;

pub type OpenFileDialog = fn() -> Result<Option<PathBuf>, Error>;
pub type SaveFileDialog = fn(default_extension: &str) -> Result<Option<PathBuf>, Error>;
pub type ErrorDisplay = fn(&str);

pub struct SystemFunctions {
    pub open_file_dialog: Option<Box<OpenFileDialog>>,
    pub save_file_dialog: Option<Box<SaveFileDialog>>,
    pub show_error: Box<ErrorDisplay>,
}

impl Default for SystemFunctions {
    fn default() -> Self {
        Self {
            open_file_dialog: None,
            save_file_dialog: None,
            show_error: Box::new(|message| eprintln!("{}\n", message)),
        }
    }
}

impl SystemFunctions {
    pub fn open_file_dialog(&self) -> Result<Option<PathBuf>, Error> {
        (self.open_file_dialog.as_ref().expect("open_file_dialog"))()
    }
    pub fn save_file_dialog(&self, default_extension: &str) -> Result<Option<PathBuf>, Error> {
        (self.save_file_dialog.as_ref().expect("save_file_dialog"))(default_extension)
    }
    pub fn show_error(&self, message: &str) {
        (&self.show_error)(message)
    }
}
