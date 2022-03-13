use eframe::egui::{self, Color32, CursorIcon, Painter, Response, Stroke};

use crate::{
    actions::{Action, UiAction},
    cell_image::CellCoordinates,
    coords::{PixelPoint, PixelTransform},
    ui, Document,
};

const SELECTION_STROKE: Stroke = Stroke {
    width: 1.0,
    color: Color32::from_rgb(200, 200, 200),
};

#[derive(Default, Debug, Clone)]
pub struct GrabTool {
    selection_start: Option<PixelPoint>,
}

impl GrabTool {
    #[allow(clippy::too_many_arguments)]
    pub fn update_ui(
        &mut self,
        painter: &Painter,
        pixel_transform: &PixelTransform,
        cursor_icon: &mut Option<CursorIcon>,
        doc: &Document,
        hover_pos: Option<PixelPoint>,
        response: &Response,
        user_actions: &mut Vec<Action>,
    ) {
        let mut selection = None;
        match self.selection_start {
            None => {
                if let Some(hover_pos) = hover_pos {
                    *cursor_icon = Some(CursorIcon::Crosshair);
                    let cell_rect = doc.image.cell_selection(hover_pos, hover_pos);
                    let cell = cell_rect.origin;
                    ui::crosshair::draw_crosshair(
                        painter,
                        pixel_transform,
                        doc.image.cell_coordinates_unclipped(&cell),
                    );
                    if response.drag_started() {
                        self.selection_start = Some(hover_pos);
                    } else if response.clicked() {
                        selection = Some((hover_pos, hover_pos));
                    }
                }
            }
            Some(selection_start) => {
                if let Some(hover_pos) = hover_pos {
                    *cursor_icon = Some(CursorIcon::Crosshair);

                    let cell_rect = doc.image.cell_selection(selection_start, hover_pos);

                    let (top_left, bottom_right) = doc.image.cell_rectangle(&cell_rect);
                    painter.rect_stroke(
                        egui::Rect::from_min_max(
                            pixel_transform.screen_pos(top_left),
                            pixel_transform.screen_pos(bottom_right),
                        ),
                        0.0,
                        SELECTION_STROKE,
                    );
                }

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
            self.selection_start = None;
            let rect = *doc.image.cell_selection(selection.0, selection.1);
            if rect.width() != 0 && rect.height() != 0 {
                user_actions.push(Action::Ui(UiAction::CreateCharBrush { rect }));
            }
        }
    }
}
