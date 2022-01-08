//! Data for the "document" the user is working on.
//! The document is what is saved to file.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    actions::DocAction, error::DisallowedAction, mutation_monitor::MutationMonitor, vic::VicImage,
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
    pub fn apply(&mut self, action: &DocAction) -> Result<bool, Box<dyn DisallowedAction>> {
        let image = &mut self.image;
        match action {
            DocAction::GlobalColor { index, value } => Ok(image.set_global_color(*index, *value)),
            DocAction::PasteTrueColor {
                source,
                target_x,
                target_y,
                format,
            } => {
                image.paste_image(source, *target_x, *target_y, *format);
                Ok(true)
            }
            DocAction::Plot { area, color } => image.plot(area, *color),
            DocAction::Fill { area, color } => image.fill_cells(area, *color),
            DocAction::CellColor { area, color } => {
                let c = image.color_index_from_paint_color(color);
                image.set_color(area, c)
            }
            DocAction::MakeHighRes { area } => image.make_high_res(area),
            DocAction::MakeMulticolor { area } => image.make_multicolor(area),
            DocAction::ReplaceColor {
                area,
                to_replace,
                replacement,
            } => image.replace_color(area, *to_replace, *replacement),
            DocAction::SwapColors {
                area,
                color_1,
                color_2,
            } => image.swap_colors(area, *color_1, *color_2),
            DocAction::CharBrushPaint { column, row, chars } => {
                image.paste_chars(*column, *row, chars.as_ref())
            }
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
