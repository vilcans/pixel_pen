mod grab;

use crate::{editing::Mode, import::Import};
use grab::GrabTool;

#[derive(Debug)]
pub enum Tool {
    Import(Import),
    Paint,
    Grab(GrabTool),
    CharBrush,
}

impl Tool {
    pub fn instructions(&self, mode: &Mode) -> &'static str {
        match self {
            Tool::Import(_) => "Tweak settings and click Import.",
            Tool::Paint => mode.instructions(),
            Tool::Grab(_) => "Click and drag to select an area to create a brush from.",
            Tool::CharBrush => "Click to draw with the character brush.",
        }
    }
}
