use std::ops::Index;

use eframe::egui::Color32;

pub fn rgba_to_rgb<T>(rgba: &T) -> [u8; 3]
where
    T: Index<usize, Output = u8>,
{
    let r = ((rgba[0] as u32 * rgba[3] as u32) / 255) as u8;
    let g = ((rgba[1] as u32 * rgba[3] as u32) / 255) as u8;
    let b = ((rgba[2] as u32 * rgba[3] as u32) / 255) as u8;
    [r, g, b]
}

pub fn closest_palette_entry(rgb: &[u8; 3], palette: impl Iterator<Item = Color32>) -> usize {
    palette
        .enumerate()
        .min_by_key(|&(_palette_index, rgb_color)| {
            (0..2)
                .map(|i| {
                    let diff = rgb_color[i] as i32 - rgb[i] as i32;
                    diff * diff
                })
                .sum::<i32>()
        })
        .unwrap()
        .0
}
