use eframe::egui::Response;

use crate::coords::Point;

#[derive(Default, Debug)]
pub struct GrabTool {
    selection_start: Option<Point>,
}

impl GrabTool {
    pub fn update_ui(
        &mut self,
        hover_pos: Option<Point>,
        response: &Response,
    ) -> Option<(Point, Point)> {
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
        selection
    }
}
