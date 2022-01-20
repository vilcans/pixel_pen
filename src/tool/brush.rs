use eframe::egui::{self, Color32, CursorIcon, Painter, Response, Stroke};
use imgref::ImgVec;

use crate::{
    actions::{Action, DocAction},
    coords::{CellRect, PixelTransform, Point},
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
    #[allow(clippy::too_many_arguments)] // Shut it up for now
    pub fn update_ui(
        &mut self,
        response: &Response,
        painter: &Painter,
        pixel_transform: &PixelTransform,
        cursor_icon: &mut Option<CursorIcon>,
        brush: &ImgVec<Char>,
        cursor_pos: Option<Point>,
        doc: &Document,
    ) -> Option<Action> {
        let cursor_pos = cursor_pos?;
        *cursor_icon = Some(CursorIcon::PointingHand);

        let (cell, _, _) = doc.image.char_coordinates_unclipped(Point {
            x: cursor_pos.x - brush.width() as i32 / 2 * Char::WIDTH as i32
                + if brush.width() % 2 == 1 {
                    0
                } else {
                    Char::WIDTH as i32 / 2
                },
            y: cursor_pos.y - brush.height() as i32 / 2 * Char::HEIGHT as i32
                + if brush.height() % 2 == 1 {
                    0
                } else {
                    Char::HEIGHT as i32 / 2
                },
        });

        let (top_left, bottom_right) = doc.image.cell_rectangle(&CellRect::from_cell_width_height(
            cell,
            brush.width() as u32,
            brush.height() as u32,
        ));

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
                pos: cell,
                chars: brush.clone(),
            }))
        } else {
            None
        }
    }
}
