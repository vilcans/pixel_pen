//! Data for the "document" the user is working on.
//! The document is what is saved to file.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    mutation_monitor::MutationMonitor,
    vic::{PaintColor, VicImage},
};

/// A "document" the user is working on.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Document {
    #[serde(skip)]
    pub filename: Option<PathBuf>,
    #[serde(default)]
    pub primary_color: PaintColor,
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
            filename: None,
            primary_color: PaintColor::CharColor(3),
            image: MutationMonitor::new_dirty(image),
        }
    }
}
