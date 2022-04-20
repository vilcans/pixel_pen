use eframe::egui::{Color32, CursorIcon, PointerButton, Stroke};
use euclid::Point2D;

use crate::{
    actions::Action,
    cell_image::CellImageSize,
    coords::{PixelPoint, PixelRect},
    update_area::UpdateArea,
};

use super::{Tool, ToolUiContext};

const STROKE: Stroke = Stroke {
    width: 1.0,
    color: Color32::from_rgb(200, 200, 200),
};

#[derive(Debug, Default, Clone)]
pub struct RectangleTool {
    /// If dragging, the corner the user started dragging in
    corner: Option<PixelPoint>,
    /// When dragging, the secondary mouse button is used so should swap primary/secondary colors.
    swap_colors: bool,
}

impl Tool for RectangleTool {
    fn update_ui(&mut self, ui_ctx: &mut ToolUiContext<'_>, user_actions: &mut Vec<Action>) {
        let hover_pos = match ui_ctx.hover_pos {
            Some(p) => p,
            None => return,
        };
        *ui_ctx.cursor_icon = Some(CursorIcon::Crosshair);

        let response = ui_ctx.widget_response;
        let pressed = if response.secondary_clicked()
            || (response.dragged()
                && ui_ctx
                    .ui
                    .input()
                    .pointer
                    .button_down(PointerButton::Secondary))
        {
            Some(true)
        } else if response.clicked() || response.dragged() {
            Some(false)
        } else {
            None
        };

        let (image_w, image_h) = ui_ctx.doc.image.size_in_pixels();
        let image_lower_right = Point2D::new(image_w as i32, image_h as i32);
        let cursor_position_clamped = hover_pos.clamp(PixelPoint::zero(), image_lower_right);
        if self.corner.is_none() && pressed.is_some() {
            self.corner = Some(cursor_position_clamped);
        }
        match self.corner {
            None => {
                *ui_ctx.cursor_icon = Some(CursorIcon::Crosshair);
                ui_ctx.draw_crosshair(hover_pos);
            }
            Some(corner) if pressed.is_some() => {
                // Dragging
                self.swap_colors = matches!(pressed, Some(true));
                ui_ctx.draw_rect(corner, cursor_position_clamped, STROKE);
            }
            Some(corner) => {
                // Released
                let selection = PixelRect::from_points(&[corner, cursor_position_clamped]);
                if selection.area() != 0 {
                    let area = UpdateArea::rectangle(selection);
                    user_actions.push(Action::Document(
                        ui_ctx
                            .ui_state
                            .mode
                            .paint_action(area, ui_ctx.colors(self.swap_colors)),
                    ));
                }
                self.corner = None;
            }
        }
    }
}
