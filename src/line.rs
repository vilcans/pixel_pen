use euclid::Vector2D;

use crate::coords::PixelPoint;

pub fn line(p0: PixelPoint, p1: PixelPoint) -> impl Iterator<Item = PixelPoint> {
    let delta_x = p1.x - p0.x;
    let delta_y = p1.y - p0.y;
    let steps = i32::max(delta_x.abs(), delta_y.abs()) + 1;
    let d = Vector2D::new(delta_x as f32 / steps as f32, delta_y as f32 / steps as f32);
    (0..steps).map(move |step| p0 + (d * step as f32).cast())
}
