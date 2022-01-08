pub mod palette;

use std::time::Instant;

use crate::{
    colors::TrueColor,
    editing::Mode,
    tool::Tool,
    vic::{Char, PaintColor},
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

#[derive(Clone, PartialEq)]
pub enum ViewSettings {
    Normal,
    Raw,
}
impl Default for ViewSettings {
    fn default() -> Self {
        ViewSettings::Normal
    }
}
impl ViewSettings {
    /// Get the colors to use when displaying in raw mode.
    pub fn raw_colors() -> (TrueColor, TrueColor, TrueColor, TrueColor) {
        (
            Self::raw_multicolor_background(),
            Self::raw_multicolor_border(),
            Self::raw_multicolor_aux(),
            Self::raw_multicolor_char_color(),
        )
    }

    /// Get color to use when displaying in raw mode.
    pub fn raw_highres_background() -> TrueColor {
        TrueColor::from_u32(0x555555)
    }
    /// Get color to use when displaying in raw mode.
    pub fn raw_hires_char_color() -> TrueColor {
        TrueColor::from_u32(0xeeeeee)
    }
    /// Get color to use when displaying in raw mode.
    pub fn raw_multicolor_background() -> TrueColor {
        TrueColor::from_u32(0x000000)
    }
    /// Get color to use when displaying in raw mode.
    pub fn raw_multicolor_border() -> TrueColor {
        TrueColor::from_u32(0x0044ff)
    }
    /// Get color to use when displaying in raw mode.
    pub fn raw_multicolor_aux() -> TrueColor {
        TrueColor::from_u32(0xff0000)
    }
    /// Get color to use when displaying in raw mode.
    pub fn raw_multicolor_char_color() -> TrueColor {
        TrueColor::from_u32(0xffffff)
    }
}
