use eframe::egui::{Pos2, Vec2};

pub fn palette_patch(pos: Pos2, width: f32, height: f32) -> Vec<Pos2> {
    vec![
        pos,
        pos + Vec2::new(width, height),
        pos + Vec2::new(-width, height),
    ]
}
