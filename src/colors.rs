use eframe::egui::Color32;
use rgb::RGBA8;

#[derive(Clone, Copy)]
pub struct TrueColor(image::Rgba<u8>);

impl TrueColor {
    /// Get amount of red (0-255)
    pub fn r(&self) -> u8 {
        self.0[0]
    }
    /// Get amount of green (0-255)
    pub fn g(&self) -> u8 {
        self.0[1]
    }
    /// Get amount of blue (0-255)
    pub fn b(&self) -> u8 {
        self.0[2]
    }
    /// Get amount of alpha (0-255)
    pub fn a(&self) -> u8 {
        self.0[3]
    }
}

impl Default for TrueColor {
    fn default() -> Self {
        Self(image::Rgba::<u8>([0u8, 0, 0, 0]))
    }
}

impl From<TrueColor> for image::Rgba<u8> {
    fn from(c: TrueColor) -> Self {
        c.0
    }
}

impl From<image::Rgba<u8>> for TrueColor {
    fn from(p: image::Rgba<u8>) -> Self {
        Self(p)
    }
}

impl From<TrueColor> for Color32 {
    fn from(c: TrueColor) -> Self {
        let c = c.0;
        Color32::from_rgba_unmultiplied(c[0], c[1], c[2], c[3])
    }
}

impl From<TrueColor> for rgb::RGBA8 {
    fn from(c: TrueColor) -> rgb::RGBA8 {
        let c = c.0;
        RGBA8::new(c[0], c[1], c[2], c[3])
    }
}

impl TrueColor {
    pub const fn from_u32(rgb: u32) -> Self {
        Self(image::Rgba([
            (rgb >> 16) as u8,
            (rgb >> 8) as u8,
            rgb as u8,
            0xff,
        ]))
    }
}

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
    color: TrueColor,
    palette: impl Iterator<Item = &'a TrueColor>,
) -> (usize, i32) {
    palette
        .enumerate()
        .map(|(palette_index, candidate)| {
            let dr = candidate.r() as i32 - color.r() as i32;
            let dg = candidate.g() as i32 - color.g() as i32;
            let db = candidate.b() as i32 - color.b() as i32;
            let error = dr * dr + dg * dg + db * db;
            (palette_index, error)
        })
        .min_by(|(_, e0), (_, e1)| e0.cmp(e1))
        .unwrap()
}
