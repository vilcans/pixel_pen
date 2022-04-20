use eframe::egui::{self, CtxRef, CursorIcon, Painter, Response, Stroke, Ui};
use imgref::ImgRef;

use crate::{
    coords::{PixelPoint, PixelTransform},
    ui::{self, UiState},
    vic::{Char, PixelColor},
    Document,
};

/// Used by Tool implementations to draw their UI.
pub struct ToolUiContext<'a> {
    pub ctx: CtxRef,
    pub ui: &'a mut Ui,
    pub painter: &'a Painter,
    pub widget_response: &'a Response,
    pub hover_pos: Option<PixelPoint>,
    pub pixel_transform: PixelTransform,
    pub cursor_icon: &'a mut Option<CursorIcon>,
    pub ui_state: &'a UiState,
    pub doc: &'a Document,
    pub brush: ImgRef<'a, Char>,
}

impl<'a> ToolUiContext<'a> {
    pub fn draw_crosshair(&self, pos: PixelPoint) {
        ui::crosshair::draw_crosshair(self.painter, &self.pixel_transform, pos);
    }

    pub fn draw_rect(&self, corner0: PixelPoint, corner1: PixelPoint, stroke: Stroke) {
        self.painter.rect_stroke(
            egui::Rect::from_points(&[
                self.pixel_transform.screen_pos(corner0),
                self.pixel_transform.screen_pos(corner1),
            ]),
            0.0,
            stroke,
        );
    }

    pub fn colors(&self, swapped: bool) -> (PixelColor, PixelColor) {
        match swapped {
            false => (self.ui_state.primary_color, self.ui_state.secondary_color),
            true => (self.ui_state.secondary_color, self.ui_state.primary_color),
        }
    }
}
