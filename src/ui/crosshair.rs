use eframe::egui::{Color32, Painter, Stroke};

use crate::coords::{PixelPoint, PixelTransform};

const STROKE: Stroke = Stroke {
    width: 1.0,
    color: Color32::from_rgb(200, 200, 200),
};

/// Draw a crosshair on the given pixel point.
/// The point is clamped to inside the image, or on the rightmost/bottommost edge.
/// (Because user may want to select everything starting from the right/bottom.)
pub fn draw_crosshair(painter: &Painter, pixel_transform: &PixelTransform, pos: PixelPoint) {
    let pos = pos.clamp(
        PixelPoint::origin(),
        PixelPoint::new(pixel_transform.pixel_width, pixel_transform.pixel_height),
    );
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
