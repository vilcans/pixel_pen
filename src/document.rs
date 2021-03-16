use std::path::Path;

/// Data for the "document" the user is working on.
/// The document is what is saved to file.
use crate::{error::Error, image_io, mutation_monitor::MutationMonitor, vic::VicImage};

/// A "document" the user is working on.
pub struct Document {
    pub paint_color: usize,
    pub image: MutationMonitor<VicImage>,
}

impl Document {
    pub fn load(filename: &Path) -> Result<Document, Error> {
        let image = image_io::load_file(filename)?;
        Ok(Document {
            paint_color: 1,
            image: MutationMonitor::new_dirty(image),
        })
    }
}

impl Default for Document {
    fn default() -> Self {
        let image = VicImage::default();
        Self {
            paint_color: 1,
            image: MutationMonitor::new_dirty(image),
        }
    }
}
