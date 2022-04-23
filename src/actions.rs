use std::fmt;

use image::RgbaImage;
use imgref::ImgVec;

use crate::{
    coords::{CellPos, CellRect, PixelPoint},
    error::{DisallowedAction, Severity},
    mode::Mode,
    tool::ToolType,
    ui::ViewSettings,
    update_area::UpdateArea,
    vic::{Char, ColorFormat, PixelColor, Register},
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

/// Some action the user performs.
pub enum Action {
    /// An action that changes the document
    Document(DocAction),
    /// An action that previews a change to the document, but does not actually change it.
    Preview(DocAction),
    /// Clears the preview, if any, and shows the actual document.
    ClearPreview,
    /// An action that changes the UI state. Not undoable.
    Ui(UiAction),
}

pub enum DocAction {
    /// Change one of the global colors.
    ChangeRegister {
        index: Register,
        value: u8,
    },
    /// Paste a true color image into the image
    PasteTrueColor {
        source: RgbaImage,
        target: PixelPoint,
        format: ColorFormat,
    },
    /// Change the color of single pixels
    Plot {
        area: UpdateArea,
        color: PixelColor,
    },
    /// Fill the whole character cell with a color
    Fill {
        area: UpdateArea,
        color: PixelColor,
    },
    /// Change the color of the cell
    CellColor {
        area: UpdateArea,
        color: PixelColor,
    },
    /// Make the cell high-res
    MakeHighRes {
        area: UpdateArea,
    },
    /// Make the cell multicolor
    MakeMulticolor {
        area: UpdateArea,
    },
    /// Replace one color with another.
    ReplaceColor {
        area: UpdateArea,
        to_replace: PixelColor,
        replacement: PixelColor,
    },
    /// Swap two colors
    SwapColors {
        area: UpdateArea,
        color_1: PixelColor,
        color_2: PixelColor,
    },
    CharBrushPaint {
        pos: CellPos,
        chars: ImgVec<Char>,
    },
}

/// An action that changes something in the user interface, not the document. Not undoable.
pub enum UiAction {
    Undo,
    Redo,
    NewDocument(Document),
    CloseEditor(usize),
    SelectTool(ToolType),
    SelectMode(Mode),
    CreateCharBrush { rect: CellRect },
    ZoomIn,
    ZoomOut,
    SetZoom(f32),
    ToggleGrid,
    ToggleRaw,
    ViewSettings(ViewSettings),
    MirrorBrushX,
    MirrorBrushY,
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

impl Action {
    /// Create an [`Action::Preview`] or [`Action::Document`] from a [`DocAction`].
    /// If `apply` is true, creates an action that affects the document, otherwise a preview.
    pub fn apply_or_preview(apply: bool, action: DocAction) -> Action {
        if apply {
            Action::Document(action)
        } else {
            Action::Preview(action)
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
