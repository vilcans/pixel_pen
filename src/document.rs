//! Data for the "document" the user is working on.
//! The document is what is saved to file.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    actions::Action, error::DisallowedAction, mutation_monitor::MutationMonitor, vic::VicImage,
};

/// A "document" the user is working on.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Document {
    #[serde(skip)]
    pub filename: Option<PathBuf>,
    pub image: MutationMonitor<VicImage>,
}

impl Document {
    pub fn from_image(image: VicImage) -> Self {
        Self {
            image: MutationMonitor::new_dirty(image),
            ..Default::default()
        }
    }

    /// Execute an action on this document
    pub fn apply(&mut self, action: &Action) -> Result<bool, Box<dyn DisallowedAction>> {
        let image = &mut self.image;
        match action {
            Action::Plot { area, color } => image.plot(area, *color),
            Action::Fill { area, color } => image.fill_cells(area, *color),
            Action::CellColor { area, color } => {
                let c = image.color_index_from_paint_color(color);
                image.set_color(area, c)
            }
            Action::MakeHighRes { area } => image.make_high_res(area),
            Action::MakeMulticolor { area } => image.make_multicolor(area),
            Action::ReplaceColor {
                area,
                to_replace,
                replacement,
            } => image.replace_color(area, *to_replace, *replacement),
            Action::SwapColors {
                area,
                color_1,
                color_2,
            } => image.swap_colors(area, *color_1, *color_2),
        }
    }
}

impl Default for Document {
    fn default() -> Self {
        let image = VicImage::default();
        Self {
            filename: None,
            image: MutationMonitor::new_dirty(image),
        }
    }
}
