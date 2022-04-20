use eframe::egui::{Color32, CursorIcon, Stroke};
use imgref::ImgVec;

use crate::{
    actions::{Action, DocAction},
    cell_image::CellCoordinates,
    coords::{CellRect, PixelPoint},
    vic::Char,
};

use super::{Tool, ToolUiContext};

const OUTLINE_STROKE: Stroke = Stroke {
    width: 1.0,
    color: Color32::from_rgb(200, 200, 200),
};

#[derive(Debug, Default, Clone)]
pub struct CharBrushTool {}

impl Tool for CharBrushTool {
    fn update_ui(&mut self, ui_ctx: &mut ToolUiContext<'_>, user_actions: &mut Vec<Action>) {
        let cursor_pos = match ui_ctx.hover_pos {
            None => return,
            Some(p) => p,
        };
        *ui_ctx.cursor_icon = Some(CursorIcon::PointingHand);

        let brush = ui_ctx.brush;
        let (cell, _, _) = ui_ctx.doc.image.cell_unclipped(PixelPoint::new(
            cursor_pos.x - brush.width() as i32 / 2 * Char::WIDTH as i32
                + if brush.width() % 2 == 1 {
                    0
                } else {
                    Char::WIDTH as i32 / 2
                },
            cursor_pos.y - brush.height() as i32 / 2 * Char::HEIGHT as i32
                + if brush.height() % 2 == 1 {
                    0
                } else {
                    Char::HEIGHT as i32 / 2
                },
        ));

        let (top_left, bottom_right) = ui_ctx.doc.image.cell_rectangle(&CellRect::new(
            cell,
            (brush.width() as i32, brush.height() as i32).into(),
        ));
        ui_ctx.draw_rect(top_left, bottom_right, OUTLINE_STROKE);

        if ui_ctx.widget_response.clicked() {
            let (buf, w, h) = brush.to_contiguous_buf().to_owned();
            user_actions.push(Action::Document(DocAction::CharBrushPaint {
                pos: cell,
                chars: ImgVec::new(buf.to_vec(), w, h),
            }));
        }
    }
}
