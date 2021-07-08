//! Data for the "document" the user is working on.
//! The document is what is saved to file.

use serde::{Deserialize, Serialize};

use crate::{import::ImportSettings, mutation_monitor::MutationMonitor, vic::VicImage};

/// A "document" the user is working on.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Document {
    #[serde(default)]
    pub import: ImportSettings,
    pub paint_color: usize,
    pub image: MutationMonitor<VicImage>,
}

impl Document {
    pub fn from_image(image: VicImage) -> Self {
        Self {
            image: MutationMonitor::new_dirty(image),
            ..Default::default()
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        let image = VicImage::default();
        Self {
            import: Default::default(),
            paint_color: 1,
            image: MutationMonitor::new_dirty(image),
        }
    }
}
