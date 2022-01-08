use eframe::egui::{self, Color32, Painter, Response, Stroke};
use imgref::ImgVec;

use crate::{
    actions::{Action, DocAction},
    coords::{PixelTransform, Point},
    vic::Char,
    Document,
};

const OUTLINE_STROKE: Stroke = Stroke {
    width: 1.0,
    color: Color32::from_rgb(200, 200, 200),
};

#[derive(Debug, Default, Clone)]
pub struct CharBrushTool {}

impl CharBrushTool {
    pub fn update_ui(
        &mut self,
        response: &Response,
        painter: &Painter,
        pixel_transform: &PixelTransform,
        brush: &ImgVec<Char>,
        cursor_pos: Option<Point>,
        doc: &Document,
    ) -> Option<Action> {
        let cursor_pos = cursor_pos?;
        let (column, row, _, _) = doc.image.char_coordinates_unclipped(
            cursor_pos.x - (brush.width() * Char::WIDTH / 2) as i32,
            cursor_pos.y - (brush.height() * Char::HEIGHT / 2) as i32,
        );

        let (top_left, bottom_right) = doc.image.cell_rectangle(
            column as i32,
            row as i32,
            brush.width() as u32,
            brush.height() as u32,
        );

        painter.rect_stroke(
            egui::Rect::from_min_max(
                pixel_transform.screen_pos(top_left),
                pixel_transform.screen_pos(bottom_right),
            ),
            0.0,
            OUTLINE_STROKE,
        );

        if response.clicked() {
            Some(Action::Document(DocAction::CharBrushPaint {
                column,
                row,
                chars: brush.clone(),
            }))
        } else {
            None
        }
    }
}
