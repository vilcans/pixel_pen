mod brush;
mod grab;
mod import;
mod paint;
mod rectangle;
mod ui;

use crate::{actions::Action, mode::Mode};
pub use brush::CharBrushTool;
pub use grab::GrabTool;
pub use import::ImportTool;
pub use paint::PaintTool;
pub use rectangle::RectangleTool;
pub use ui::ToolUiContext;

#[derive(Copy, Clone)]
pub enum ToolType {
    Import,
    Paint,
    Rectangle,
    Grab,
    CharBrush,
}

impl ToolType {
    pub fn instructions(&self, mode: &Mode) -> &'static str {
        match self {
            ToolType::Import => "Tweak settings and click Import.",
            ToolType::Paint | ToolType::Rectangle => mode.instructions(),
            ToolType::Grab => "Click and drag to select an area to create a brush from.",
            ToolType::CharBrush => "Click to draw with the character brush.",
        }
    }
}

pub trait Tool {
    fn update_ui(&mut self, ui_ctx: &mut ToolUiContext<'_>, user_actions: &mut Vec<Action>);
}

#[derive(Default)]
pub struct Toolbox {
    pub import: ImportTool,
    pub paint: PaintTool,
    pub grab: GrabTool,
    pub rectangle: RectangleTool,
    pub char_brush: CharBrushTool,
}

impl Toolbox {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn get_mut(&mut self, t: ToolType) -> &mut dyn Tool {
        match t {
            ToolType::Import => &mut self.import,
            ToolType::Paint => &mut self.paint,
            ToolType::Rectangle => &mut self.rectangle,
            ToolType::Grab => &mut self.grab,
            ToolType::CharBrush => &mut self.char_brush,
        }
    }
}
