//! Data for the "document" the user is working on.
//! The document is what is saved to file.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use crate::{
    actions::DocAction, error::DisallowedAction, mutation_monitor::MutationMonitor, vic::VicImage,
};

const ERROR_FILENAME: &str = "INVALID FILENAME";

/// A "document" the user is working on.
#[derive(Serialize, Deserialize, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Document {
    #[serde(skip)]
    pub filename: Option<PathBuf>,
    /// Number for the document. For generating "Untitled-X" temporary name for unsaved files.
    #[serde(skip)]
    pub index_number: u32,
    pub image: MutationMonitor<VicImage>,
}

impl Default for Document {
    fn default() -> Self {
        Self::new()
    }
}

impl Document {
    pub fn new() -> Self {
        Self {
            filename: None,
            index_number: 0,
            image: MutationMonitor::new_dirty(VicImage::default()),
        }
    }

    pub fn from_image(image: VicImage) -> Self {
        Self {
            filename: None,
            index_number: 0,
            image: MutationMonitor::new_dirty(image),
        }
    }

    /// A name for this document.
    /// If it has a file name, only return the file name part of it, not the complete path.
    pub fn short_name(&self) -> String {
        match &self.filename {
            None => format!("Untitled-{}", self.index_number),
            Some(path) => path
                .file_name()
                .map(|f| f.to_string_lossy().to_string())
                .unwrap_or_else(|| ERROR_FILENAME.to_string()),
        }
    }

    /// Full name for this document to show where there is space for the full path.
    pub fn visible_name(&self) -> String {
        match &self.filename {
            None => format!("Untitled-{}", self.index_number),
            Some(path) => path.to_string_lossy().to_string(),
        }
    }

    /// Execute an action on this document
    pub fn apply(&mut self, action: &DocAction) -> Result<bool, Box<dyn DisallowedAction>> {
        let image = &mut self.image;
        match action {
            DocAction::GlobalColor { index, value } => Ok(image.set_global_color(*index, *value)),
            DocAction::PasteTrueColor {
                source,
                target,
                format,
            } => {
                image.paste_image(source, *target, *format);
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
            DocAction::CharBrushPaint { pos, chars } => image.paste_chars(pos, chars.as_ref()),
        }
    }
}
