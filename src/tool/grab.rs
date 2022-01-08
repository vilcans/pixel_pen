use eframe::egui::{self, Color32, Painter, Response, Stroke};

use crate::{
    actions::{Action, UiAction},
    coords::{PixelTransform, Point},
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
        doc: &Document,
        hover_pos: Option<Point>,
        response: &Response,
    ) -> Option<Action> {
        let mut selection = None;
        match self.selection_start {
            None => {
                if let Some(hover_pos) = hover_pos {
                    if let Some((column, row, _, _)) =
                        doc.image.char_coordinates(hover_pos.x, hover_pos.y)
                    {
                        draw_crosshair(
                            painter,
                            pixel_transform,
                            doc.image
                                .cell_coordinates_unclipped(column as i32, row as i32),
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
                    let (column, row, width, height) =
                        selection_to_cells(&doc.image, (selection_start, hover_pos));

                    let (top_left, bottom_right) = doc.image.cell_rectangle(
                        column as i32,
                        row as i32,
                        width as u32,
                        height as u32,
                    );
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
            let (column, row, width, height) = selection_to_cells(&doc.image, selection);
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

fn selection_to_cells(image: &VicImage, selection: (Point, Point)) -> (usize, usize, usize, usize) {
    let (col0, row0, _, _) = image.char_coordinates_clamped(selection.0.x, selection.0.y);
    let (col1, row1, _, _) = image.char_coordinates_clamped(selection.1.x, selection.1.y);
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
    (column, row, width + 1, height + 1)
}
