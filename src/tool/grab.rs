use eframe::egui::Response;

use crate::{
    actions::{Action, UiAction},
    coords::Point,
    Document,
};

#[derive(Default, Debug)]
pub struct GrabTool {
    selection_start: Option<Point>,
}

impl GrabTool {
    pub fn update_ui(
        &mut self,
        doc: &Document,
        hover_pos: Option<Point>,
        response: &Response,
    ) -> Option<Action> {
        let mut selection = None;
        match self.selection_start {
            None => {
                if let Some(hover_pos) = hover_pos {
                    if response.drag_started() {
                        if response.drag_started() {
                            self.selection_start = Some(hover_pos);
                        }
                    } else if response.clicked() {
                        selection = Some((hover_pos, hover_pos));
                    }
                }
            }
            Some(selection_start) => {
                if response.drag_released() {
                    if let Some(hover_pos) = hover_pos {
                        selection = Some((selection_start, hover_pos));
                    } else {
                        self.selection_start = None;
                    }
                }
            }
        }
        if let Some(selection) = selection {
            let (col0, row0, _, _) = doc
                .image
                .char_coordinates_clamped(selection.0.x, selection.0.y);
            let (col1, row1, _, _) = doc
                .image
                .char_coordinates_clamped(selection.1.x, selection.1.y);
            let (column, width) = if col1 >= col0 {
                (col0, col1 - col0)
            } else {
                (col1, col0 - col1)
            };
            let (row, height) = if row1 >= row0 {
                (row0, row1 - row0)
            } else {
                (row1, row0 - row1)
            };
            Some(Action::Ui(UiAction::CreateCharBrush {
                column,
                row,
                width,
                height,
            }))
        } else {
            None
        }
    }
}
