use eframe::egui::{self, CursorIcon, PointerButton};

use crate::{
    actions::Action, coords::Point, editing::Mode, update_area::UpdateArea, vic::PixelColor,
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
        colors: (PixelColor, PixelColor),
    ) -> Option<Action> {
        let hover_pos = pixel_pos?;

        *cursor_icon = Some(CursorIcon::PointingHand);

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
            mode.paint_action(area, colors.1, colors.0)
        } else {
            mode.paint_action(area, colors.0, colors.1)
        };
        Some(Action::Document(action))
    }
}
