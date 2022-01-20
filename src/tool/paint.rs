use eframe::egui::{self, Color32, CursorIcon, Painter, PointerButton, Stroke};

use crate::{
    actions::Action,
    coords::{CellRect, PixelTransform, Point, SizeInCells},
    editing::Mode,
    update_area::UpdateArea,
    vic::PixelColor,
    Document,
};

const MAKE_HIRES_HIGHLIGHT: Stroke = Stroke {
    width: 2.0,
    color: Color32::from_rgb(200, 200, 200),
};
const MAKE_MULTICOLOR_HIGHLIGHT: Stroke = Stroke {
    width: 2.0,
    color: Color32::from_rgb(255, 255, 255),
};

#[derive(Debug, Default, Clone)]
pub struct PaintTool {
    /// Where the user currently is painting
    pub paint_position: Option<Point>,
}

impl PaintTool {
    #[allow(clippy::too_many_arguments)] // Shut it up for now
    pub fn update_ui(
        &mut self,
        pixel_pos: Option<Point>,
        ui: &mut egui::Ui,
        response: &egui::Response,
        painter: &Painter,
        pixel_transform: &PixelTransform,
        cursor_icon: &mut Option<CursorIcon>,
        mode: &Mode,
        colors: (PixelColor, PixelColor),
        doc: &Document,
    ) -> Option<Action> {
        let hover_pos = pixel_pos?;
        *cursor_icon = Some(CursorIcon::PointingHand);

        // Highlight character
        if let Some((cell, _, _)) = doc.image.char_coordinates(hover_pos) {
            let (top_left, bottom_right) = doc.image.cell_rectangle(&CellRect {
                top_left: *cell,
                size: SizeInCells::ONE,
            });
            if let Some(stroke) = match mode {
                Mode::FillCell | Mode::CellColor => Some(Stroke {
                    width: 1.0,
                    color: doc.image.true_color_from_paint_color(&colors.0).into(),
                }),
                Mode::MakeHiRes => Some(MAKE_HIRES_HIGHLIGHT),
                Mode::MakeMulticolor => Some(MAKE_MULTICOLOR_HIGHLIGHT),
                _ => None,
            } {
                painter.rect_stroke(
                    egui::Rect::from_min_max(
                        pixel_transform.screen_pos(top_left),
                        pixel_transform.screen_pos(bottom_right),
                    ),
                    0.0,
                    stroke,
                );
            }
        }

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
