use eframe::egui::{Color32, Painter, Stroke};

use crate::coords::{PixelPoint, PixelTransform};

const STROKE: Stroke = Stroke {
    width: 1.0,
    color: Color32::from_rgb(200, 200, 200),
};

pub fn draw_crosshair(painter: &Painter, pixel_transform: &PixelTransform, pos: PixelPoint) {
    painter.line_segment(
        [
            pixel_transform.screen_pos(PixelPoint::new(pos.x, 0)),
            pixel_transform.screen_pos(PixelPoint::new(pos.x, pixel_transform.pixel_height)),
        ],
        STROKE,
    );
    painter.line_segment(
        [
            pixel_transform.screen_pos(PixelPoint::new(0, pos.y)),
            pixel_transform.screen_pos(PixelPoint::new(pixel_transform.pixel_width, pos.y)),
        ],
        STROKE,
    );
}
