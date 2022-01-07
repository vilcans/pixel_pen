use std::fmt;

use image::RgbaImage;
use imgref::ImgVec;

use crate::{
    error::{DisallowedAction, Severity},
    tool::Tool,
    update_area::UpdateArea,
    vic::{Char, ColorFormat, PaintColor},
    Document,
};

pub struct Undoable {
    pub action: DocAction,
    previous: Option<Document>,
}

impl Undoable {
    pub fn new(action: DocAction) -> Self {
        Self {
            action,
            previous: None,
        }
    }
}

pub enum Action {
    Document(DocAction),
    Ui(UiAction),
}

pub enum DocAction {
    /// Paste a true color image into the image
    PasteTrueColor {
        source: RgbaImage,
        target_x: i32,
        target_y: i32,
        format: ColorFormat,
    },
    /// Change the color of single pixels
    Plot { area: UpdateArea, color: PaintColor },
    /// Fill the whole character cell with a color
    Fill { area: UpdateArea, color: PaintColor },
    /// Change the color of the cell
    CellColor { area: UpdateArea, color: PaintColor },
    /// Make the cell high-res
    MakeHighRes { area: UpdateArea },
    /// Make the cell multicolor
    MakeMulticolor { area: UpdateArea },
    /// Replace one color with another.
    ReplaceColor {
        area: UpdateArea,
        to_replace: PaintColor,
        replacement: PaintColor,
    },
    /// Swap two colors
    SwapColors {
        area: UpdateArea,
        color_1: PaintColor,
        color_2: PaintColor,
    },
    CharBrushPaint {
        column: i32,
        row: i32,
        chars: ImgVec<Char>,
    },
}

/// An action that changes something in the user interface, not the document. Not undoable.
pub enum UiAction {
    SelectTool(Tool),
    CreateCharBrush {
        column: usize,
        row: usize,
        width: usize,
        height: usize,
    },
}

impl undo::Action for Undoable {
    type Target = Document;
    type Output = bool;
    type Error = Box<dyn DisallowedAction>;

    fn apply(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        let previous = target.clone();
        match target.apply(&self.action) {
            Ok(true) => {
                self.previous = Some(previous);
                Ok(true)
            }
            Ok(false) => Err(Box::new(NoChange)),
            other => other,
        }
    }

    fn undo(&mut self, target: &mut Self::Target) -> undo::Result<Self> {
        match self.previous.take() {
            Some(previous) => {
                *target = previous;
                Ok(true)
            }
            None => Ok(false),
        }
    }
}

#[derive(Debug)]
struct NoChange;

impl fmt::Display for NoChange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "No change")
    }
}

impl DisallowedAction for NoChange {
    fn severity(&self) -> crate::error::Severity {
        Severity::Silent
    }
}
