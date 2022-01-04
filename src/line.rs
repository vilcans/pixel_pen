use crate::coords::Point;

pub fn line(p0: Point, p1: Point) -> impl Iterator<Item = Point> {
    let delta_x = p1.x - p0.x;
    let delta_y = p1.y - p0.y;
    let steps = i32::max(delta_x.abs(), delta_y.abs()) + 1;
    let dx = delta_x as f32 / steps as f32;
    let dy = delta_y as f32 / steps as f32;
    (0..steps).map(move |step| Point {
        x: (p0.x as f32 + dx * step as f32) as i32,
        y: (p0.y as f32 + dy * step as f32) as i32,
    })
}
