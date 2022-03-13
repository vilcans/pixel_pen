mod brush;
mod grab;
mod import;
mod paint;
mod rectangle;

use crate::mode::Mode;
pub use brush::CharBrushTool;
pub use grab::GrabTool;
pub use import::ImportTool;
pub use paint::PaintTool;

use self::rectangle::RectangleTool;

#[derive(Clone)]
pub enum Tool {
    Import(ImportTool),
    Paint(PaintTool),
    Rectangle(RectangleTool),
    Grab(GrabTool),
    CharBrush(CharBrushTool),
}

impl Tool {
    pub fn instructions(&self, mode: &Mode) -> &'static str {
        match self {
            Tool::Import(_) => "Tweak settings and click Import.",
            Tool::Paint(_) | Tool::Rectangle(_) => mode.instructions(),
            Tool::Grab(_) => "Click and drag to select an area to create a brush from.",
            Tool::CharBrush(_) => "Click to draw with the character brush.",
        }
    }
}
