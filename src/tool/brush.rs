use eframe::egui::Response;
use imgref::ImgVec;

use crate::{
    actions::{Action, DocAction},
    coords::Point,
    vic::Char,
    Document,
};

#[derive(Debug, Default, Clone)]
pub struct CharBrushTool {}

impl CharBrushTool {
    pub fn update_ui(
        &mut self,
        response: &Response,
        brush: &ImgVec<Char>,
        cursor_pos: Option<Point>,
        doc: &Document,
    ) -> Option<Action> {
        if response.clicked() {
            if let Some((column, row, _, _)) = cursor_pos.map(|p| {
                doc.image.char_coordinates_unclipped(
                    p.x - (brush.width() * Char::WIDTH / 2) as i32,
                    p.y - (brush.height() * Char::HEIGHT / 2) as i32,
                )
            }) {
                Some(Action::Document(DocAction::CharBrushPaint {
                    column,
                    row,
                    chars: brush.clone(),
                }))
            } else {
                None
            }
        } else {
            None
        }
    }
}
