use eframe::egui::{self, Color32, CursorIcon, Painter, PointerButton, Stroke};
use euclid::Point2D;

use crate::{
    actions::Action,
    cell_image::CellImageSize,
    coords::{PixelPoint, PixelRect, PixelTransform},
    mode::Mode,
    ui,
    update_area::UpdateArea,
    vic::PixelColor,
    Document,
};

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

impl RectangleTool {
    #[allow(clippy::too_many_arguments)] // Shut it up for now
    pub fn update_ui(
        &mut self,
        pixel_pos: Option<PixelPoint>,
        ui: &mut egui::Ui,
        response: &egui::Response,
        painter: &Painter,
        pixel_transform: &PixelTransform,
        cursor_icon: &mut Option<CursorIcon>,
        mode: &Mode,
        colors: (PixelColor, PixelColor),
        doc: &Document,
        user_actions: &mut Vec<Action>,
    ) {
        let hover_pos = match pixel_pos {
            Some(p) => p,
            None => return,
        };
        *cursor_icon = Some(CursorIcon::Crosshair);

        let pressed = if response.secondary_clicked()
            || (response.dragged() && ui.input().pointer.button_down(PointerButton::Secondary))
        {
            Some(true)
        } else if response.clicked() || response.dragged() {
            Some(false)
        } else {
            None
        };

        let (image_w, image_h) = doc.image.size_in_pixels();
        let image_lower_right = Point2D::new(image_w as i32, image_h as i32);
        let cursor_position_clamped = hover_pos.clamp(PixelPoint::zero(), image_lower_right);
        if self.corner.is_none() && pressed.is_some() {
            self.corner = Some(cursor_position_clamped);
        }
        match self.corner {
            None => {
                *cursor_icon = Some(CursorIcon::Crosshair);
                ui::crosshair::draw_crosshair(painter, pixel_transform, hover_pos);
            }
            Some(corner) if pressed.is_some() => {
                // Dragging
                self.swap_colors = matches!(pressed, Some(true));
                painter.rect_stroke(
                    egui::Rect::from_points(&[
                        pixel_transform.screen_pos(corner),
                        pixel_transform.screen_pos(cursor_position_clamped),
                    ]),
                    0.0,
                    STROKE,
                );
            }
            Some(corner) => {
                // Released
                let selection = PixelRect::from_points(&[corner, cursor_position_clamped]);
                if selection.area() != 0 {
                    let area = UpdateArea::rectangle(selection);
                    user_actions.push(Action::Document(if self.swap_colors {
                        mode.paint_action(area, colors.1, colors.0)
                    } else {
                        mode.paint_action(area, colors.0, colors.1)
                    }));
                }
                self.corner = None;
            }
        }
    }
}
