pub mod palette;

use std::time::Instant;

use crate::{
    editing::Mode,
    tool::Tool,
    vic::{Char, PaintColor, ViewSettings},
};
use eframe::egui::Vec2;
use imgref::ImgVec;

pub struct UiState {
    pub tool: Tool,
    pub mode: Mode,
    pub zoom: f32,
    pub image_view_settings: ViewSettings,
    /// Primary selected color. Typically used when using the left mouse button.
    pub primary_color: PaintColor,
    /// Secondary selected color. Typically used when using the right mouse button.
    pub secondary_color: PaintColor,
    /// Characters to use as a brush
    pub char_brush: ImgVec<Char>,
    /// Enable showing the character grid
    pub grid: bool,
    /// Whether user is currently panning
    pub panning: bool,
    pub pan: Vec2,

    pub message: Option<(Instant, String)>,
}
impl UiState {
    pub fn show_warning(&mut self, message: String) {
        self.message = Some((Instant::now(), message));
    }
}
