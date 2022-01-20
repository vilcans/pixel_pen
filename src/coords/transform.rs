//! Handles transforms between coordinate systems.

use eframe::egui::{Pos2, Rect};

use super::PixelPoint;

pub struct PixelTransform {
    pub screen_rect: Rect,
    pub pixel_width: i32,
    pub pixel_height: i32,
}

impl PixelTransform {
    /// Convert pixel coordinates to screen coordinates.
    pub fn screen_pos(&self, p: impl Into<PixelPoint>) -> Pos2 {
        let p = p.into();
        Pos2::new(
            self.screen_rect.left()
                + self.screen_rect.width() * p.x as f32 / self.pixel_width as f32,
            self.screen_rect.top()
                + self.screen_rect.height() * p.y as f32 / self.pixel_height as f32,
        )
    }

    /// Convert screen coordinates to pixel coordinates.
    /// May return pixel coordinates outside the image.
    pub fn pixel_pos(&self, p: impl Into<Pos2>) -> PixelPoint {
        let p = p.into();
        let p = p - self.screen_rect.left_top();
        let fx = p.x / self.screen_rect.size().x;
        let fy = p.y / self.screen_rect.size().y;
        let x = (fx * self.pixel_width as f32) as i32;
        let y = (fy * self.pixel_height as f32) as i32;
        PixelPoint { x, y }
    }

    /// Convert screen coordinates to pixel coordinates.
    /// Returns None if the pixel coordinates are out of bounds.
    #[allow(dead_code)]
    pub fn bounded_pixel_pos(&self, p: impl Into<Pos2>) -> Option<PixelPoint> {
        let pix = self.pixel_pos(p);
        if (0..self.pixel_width).contains(&pix.x) && (0..self.pixel_height).contains(&pix.y) {
            Some(pix)
        } else {
            None
        }
    }
}
