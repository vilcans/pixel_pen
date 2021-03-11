//! Screen and pixel coordinate systems.

use eframe::egui::{Pos2, Rect};

/// Integer point, e.g. pixel coordinates.
pub struct Point {
    pub x: i32,
    pub y: i32,
}

pub struct PixelTransform {
    pub screen_rect: Rect,
    pub pixel_width: i32,
    pub pixel_height: i32,
}

impl PixelTransform {
    /// Convert pixel coordinates to screen coordinates.
    pub fn screen_pos(&self, p: impl Into<Point>) -> Pos2 {
        let p = p.into();
        Pos2::new(
            self.screen_rect.left()
                + self.screen_rect.width() * p.x as f32 / self.pixel_width as f32,
            self.screen_rect.top()
                + self.screen_rect.height() * p.y as f32 / self.pixel_height as f32,
        )
    }

    /// Convert screen coordinates to pixel coordinates.
    /// Return None if the pixel coordinates are out of bounds.
    pub fn bounded_pixel_pos(&self, p: impl Into<Pos2>) -> Option<Point> {
        let p = p.into();
        let p = p - self.screen_rect.left_top();
        let fx = p.x / self.screen_rect.size().x;
        let fy = p.y / self.screen_rect.size().y;
        let x = (fx * self.pixel_width as f32).round() as i32;
        let y = (fy * self.pixel_height as f32).round() as i32;
        if x >= 0 && x < self.pixel_width && y >= 0 && y < self.pixel_height {
            Some(Point { x, y })
        } else {
            None
        }
    }
}
