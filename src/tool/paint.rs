use eframe::egui::{self, CursorIcon, PointerButton};

use crate::{
    actions::Action, coords::Point, editing::Mode, update_area::UpdateArea, vic::PaintColor,
};

#[derive(Debug, Default, Clone)]
pub struct PaintTool {
    /// Where the user currently is painting
    pub paint_position: Option<Point>,
}

impl PaintTool {
    pub fn update_ui(
        &mut self,
        pixel_pos: Option<Point>,
        ui: &mut egui::Ui,
        response: &egui::Response,
        cursor_icon: &mut Option<CursorIcon>,
        mode: &Mode,
        primary_color: PaintColor,
        secondary_color: PaintColor,
    ) -> Option<Action> {
        if pixel_pos.is_none() {
            return None;
        }
        *cursor_icon = Some(CursorIcon::PointingHand);

        let hover_pos = pixel_pos.unwrap();

        let pressed = if response.secondary_clicked()
            || (response.dragged() && ui.input().pointer.button_down(PointerButton::Secondary))
        {
            Some(true)
        } else if response.clicked() || response.dragged() {
            Some(false)
        } else {
            None
        };

        let secondary = match pressed {
            None => {
                self.paint_position = None;
                return None;
            }
            Some(v) => v,
        };

        let area = match self.paint_position {
            Some(p) => {
                if p == hover_pos {
                    // Mouse is held and hasn't moved
                    return None;
                }
                UpdateArea::pixel_line(p, hover_pos)
            }
            None => UpdateArea::from_pixel(hover_pos),
        };
        self.paint_position = Some(hover_pos);

        let action = if secondary {
            // Used secondary mouse button - swap colors
            mode.paint_action(area, secondary_color, primary_color)
        } else {
            mode.paint_action(area, primary_color, secondary_color)
        };
        Some(action)
    }
}
