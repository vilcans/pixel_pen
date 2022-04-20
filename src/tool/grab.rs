use eframe::egui::{Color32, CursorIcon, Stroke};

use crate::{
    actions::{Action, UiAction},
    cell_image::CellCoordinates,
    coords::PixelPoint,
};

use super::{Tool, ToolUiContext};

const SELECTION_STROKE: Stroke = Stroke {
    width: 1.0,
    color: Color32::from_rgb(200, 200, 200),
};

#[derive(Default, Debug, Clone)]
pub struct GrabTool {
    selection_start: Option<PixelPoint>,
}

impl Tool for GrabTool {
    fn update_ui(&mut self, ui_ctx: &mut ToolUiContext<'_>, user_actions: &mut Vec<Action>) {
        let hover_pos = ui_ctx.hover_pos;
        let doc = ui_ctx.doc;

        let mut selection = None;
        match self.selection_start {
            None => {
                if let Some(hover_pos) = hover_pos {
                    *ui_ctx.cursor_icon = Some(CursorIcon::Crosshair);
                    let cell_rect = doc.image.cell_selection(hover_pos, hover_pos);
                    let cell = cell_rect.origin;
                    ui_ctx.draw_crosshair(doc.image.cell_coordinates_unclipped(&cell));
                    let response = ui_ctx.widget_response;
                    if response.drag_started() {
                        self.selection_start = Some(hover_pos);
                    } else if response.clicked() {
                        selection = Some((hover_pos, hover_pos));
                    }
                }
            }
            Some(selection_start) => {
                if let Some(hover_pos) = hover_pos {
                    *ui_ctx.cursor_icon = Some(CursorIcon::Crosshair);

                    let cell_rect = doc.image.cell_selection(selection_start, hover_pos);

                    let (top_left, bottom_right) = doc.image.cell_rectangle(&cell_rect);
                    ui_ctx.draw_rect(top_left, bottom_right, SELECTION_STROKE);
                }

                if ui_ctx.widget_response.drag_released() {
                    if let Some(hover_pos) = hover_pos {
                        selection = Some((selection_start, hover_pos));
                    } else {
                        self.selection_start = None;
                    }
                }
            }
        }
        if let Some(selection) = selection {
            self.selection_start = None;
            let rect = *doc.image.cell_selection(selection.0, selection.1);
            if rect.width() != 0 && rect.height() != 0 {
                user_actions.push(Action::Ui(UiAction::CreateCharBrush { rect }));
            }
        }
    }
}
