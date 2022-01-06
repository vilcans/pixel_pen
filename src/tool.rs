mod grab;
mod paint;

use crate::{editing::Mode, import::Import};
pub use grab::GrabTool;
pub use paint::PaintTool;

#[derive(Debug)]
pub enum Tool {
    Import(Import),
    Paint(PaintTool),
    Grab(GrabTool),
    CharBrush,
}

impl Tool {
    pub fn instructions(&self, mode: &Mode) -> &'static str {
        match self {
            Tool::Import(_) => "Tweak settings and click Import.",
            Tool::Paint(_) => mode.instructions(),
            Tool::Grab(_) => "Click and drag to select an area to create a brush from.",
            Tool::CharBrush => "Click to draw with the character brush.",
        }
    }
}
