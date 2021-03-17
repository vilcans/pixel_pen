//! Data for the "document" the user is working on.
//! The document is what is saved to file.

use serde::{Deserialize, Serialize};

use crate::{mutation_monitor::MutationMonitor, vic::VicImage};

fn default_image() -> MutationMonitor<VicImage> {
    MutationMonitor::new_dirty(VicImage::default())
}

/// A "document" the user is working on.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct Document {
    pub paint_color: usize,
    #[serde(default = "default_image")]
    #[serde(skip)]
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
            paint_color: 1,
            image: MutationMonitor::new_dirty(image),
        }
    }
}
