use eframe::egui::Color32;
use rgb::RGBA8;

/// Convert from [`eframe::egui::Color32`] to [`rgb::RGBA8`].
#[allow(dead_code)]
pub fn rgba_from_color32(color: Color32) -> RGBA8 {
    RGBA8::new(color.r(), color.g(), color.b(), color.a())
}

/// Convert from [`image::Rgba`] to [`rgb::RGBA8`].
#[allow(dead_code)]
pub fn rgba_from_image(color: image::Rgba<u8>) -> RGBA8 {
    RGBA8::new(color.0[0], color.0[1], color.0[2], color.0[3])
}

/// Convert from [`image::Rgba`] to [`Color32`].
#[allow(dead_code)]
pub fn color32_from_image(color: image::Rgba<u8>) -> Color32 {
    Color32::from_rgba_unmultiplied(color.0[0], color.0[1], color.0[2], color.0[3])
}

/// Find the color in the given palette that best matches the given color.
/// Returns the index of the best palette entry and the amount of error compared to the color.
#[allow(dead_code)]
pub fn closest_palette_entry<'a>(
    color: RGBA8,
    palette: impl Iterator<Item = &'a RGBA8>,
) -> (usize, i32) {
    palette
        .enumerate()
        .map(|(palette_index, candidate)| {
            let dr = candidate.r as i32 - color.r as i32;
            let dg = candidate.g as i32 - color.g as i32;
            let db = candidate.b as i32 - color.b as i32;
            let error = dr * dr + dg * dg + db * db;
            (palette_index, error)
        })
        .min_by(|(_, e0), (_, e1)| e0.cmp(e1))
        .unwrap()
}
