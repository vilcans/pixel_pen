use eframe::egui::{self, Color32, CursorIcon, PointerButton, Stroke};

use crate::{
    actions::Action,
    cell_image::CellCoordinates,
    coords::{CellRect, PixelPoint, SizeInCells},
    mode::Mode,
    update_area::UpdateArea,
};

use super::{Tool, ToolUiContext};

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
    pub paint_position: Option<PixelPoint>,
}

impl Tool for PaintTool {
    fn update_ui(&mut self, ui_ctx: &mut ToolUiContext<'_>, user_actions: &mut Vec<Action>) {
        let hover_pos = match ui_ctx.hover_pos {
            Some(p) => p,
            None => return,
        };
        *ui_ctx.cursor_icon = Some(CursorIcon::PointingHand);

        let doc = ui_ctx.doc;

        // Highlight character
        if let Some((cell, _, _)) = doc.image.cell(hover_pos) {
            let (top_left, bottom_right) = doc
                .image
                .cell_rectangle(&CellRect::new(*cell, SizeInCells::new(1, 1)));
            if let Some(stroke) = match ui_ctx.ui_state.mode {
                Mode::FillCell | Mode::CellColor => Some(Stroke {
                    width: 1.0,
                    color: doc
                        .image
                        .true_color_from_paint_color(&ui_ctx.ui_state.primary_color)
                        .into(),
                }),
                Mode::MakeHiRes => Some(MAKE_HIRES_HIGHLIGHT),
                Mode::MakeMulticolor => Some(MAKE_MULTICOLOR_HIGHLIGHT),
                _ => None,
            } {
                ui_ctx.painter.rect_stroke(
                    egui::Rect::from_min_max(
                        ui_ctx.pixel_transform.screen_pos(top_left),
                        ui_ctx.pixel_transform.screen_pos(bottom_right),
                    ),
                    0.0,
                    stroke,
                );
            }
        }

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

        let secondary = match pressed {
            None => {
                self.paint_position = None;
                return;
            }
            Some(v) => v,
        };

        let area = match self.paint_position {
            Some(p) => {
                if p == hover_pos {
                    // Mouse is held and hasn't moved
                    return;
                }
                UpdateArea::pixel_line(p, hover_pos)
            }
            None => UpdateArea::from_pixel(hover_pos),
        };
        self.paint_position = Some(hover_pos);

        let ui_state = ui_ctx.ui_state;
        user_actions.push(Action::Document(
            ui_state.mode.paint_action(area, ui_ctx.colors(secondary)),
        ));
    }
}
