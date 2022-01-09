use eframe::egui::{self, Color32, CursorIcon, Painter, Response, Stroke};

use crate::{
    actions::{Action, UiAction},
    coords::{CellPos, PixelTransform, Point},
    vic::VicImage,
    Document,
};

const SELECTION_STROKE: Stroke = Stroke {
    width: 1.0,
    color: Color32::from_rgb(200, 200, 200),
};

const CROSSHAIR_STROKE: Stroke = Stroke {
    width: 1.0,
    color: Color32::from_rgb(200, 200, 200),
};

#[derive(Default, Debug)]
pub struct GrabTool {
    selection_start: Option<Point>,
}

impl GrabTool {
    pub fn update_ui(
        &mut self,
        painter: &Painter,
        pixel_transform: &PixelTransform,
        cursor_icon: &mut Option<CursorIcon>,
        doc: &Document,
        hover_pos: Option<Point>,
        response: &Response,
    ) -> Option<Action> {
        let mut selection = None;
        match self.selection_start {
            None => {
                if let Some(hover_pos) = hover_pos {
                    *cursor_icon = Some(CursorIcon::Crosshair);
                    if let Some((cell, _, _)) = doc.image.char_coordinates(hover_pos.x, hover_pos.y)
                    {
                        draw_crosshair(
                            painter,
                            pixel_transform,
                            doc.image.cell_coordinates_unclipped(&cell),
                        )
                    }
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

                    let (cell, width, height) =
                        selection_to_cells(&doc.image, (selection_start, hover_pos));

                    let (top_left, bottom_right) =
                        doc.image.cell_rectangle(&cell, width as u32, height as u32);
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
            let (pos, width, height) = selection_to_cells(&doc.image, selection);
            Some(Action::Ui(UiAction::CreateCharBrush { pos, width, height }))
        } else {
            None
        }
    }
}

fn draw_crosshair(painter: &Painter, pixel_transform: &PixelTransform, pos: Point) {
    painter.line_segment(
        [
            pixel_transform.screen_pos(Point::new(pos.x, 0)),
            pixel_transform.screen_pos(Point::new(pos.x, pixel_transform.pixel_height)),
        ],
        CROSSHAIR_STROKE,
    );
    painter.line_segment(
        [
            pixel_transform.screen_pos(Point::new(0, pos.y)),
            pixel_transform.screen_pos(Point::new(pixel_transform.pixel_width, pos.y)),
        ],
        CROSSHAIR_STROKE,
    );
}

fn selection_to_cells(image: &VicImage, selection: (Point, Point)) -> (CellPos, usize, usize) {
    let (c0, _, _) = image.char_coordinates_clamped(selection.0.x, selection.0.y);
    let (c1, _, _) = image.char_coordinates_clamped(selection.1.x, selection.1.y);
    let (column, width) = if c1.column >= c0.column {
        (c0.column, c1.column - c0.column)
    } else {
        (c1.column, c0.column - c1.column)
    };
    let (row, height) = if c1.row >= c0.row {
        (c0.row, c1.row - c0.row)
    } else {
        (c1.row, c0.row - c1.row)
    };
    (
        CellPos { column, row },
        width as usize + 1,
        height as usize + 1,
    )
}
