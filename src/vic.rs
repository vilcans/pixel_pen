use eframe::egui::Color32;
use imgref::{ImgRefMut, ImgVec};
use itertools::Itertools;

// From /usr/lib/vice/VIC20/vice.vpl
const PALETTE: [u32; 16] = [
    // 0xRRGGBB
    0x000000, // Black
    0xffffff, // White
    0xf00000, // Red
    0x00f0f0, // Cyan
    0x600060, // Purple
    0x00a000, // Green
    0x0000f0, // Blue
    0xd0d000, // Yellow
    0xc0a000, // Orange
    0xffa000, // Light Orange
    0xf08080, // Pink
    0x00ffff, // Light Cyan
    0xff00ff, // Light Purple
    0x00ff00, // Light Green
    0x00a0ff, // Light Blue
    0xffff00, // Light Yellow
];

/// Number of entries in the palette.
pub const PALETTE_SIZE: usize = 16;

#[derive(Clone)]
pub struct Char {
    pixels: ImgVec<u8>,
}

impl Char {
    const WIDTH: usize = 8;
    const HEIGHT: usize = 8;

    pub fn render_to(&self, mut pixels: ImgRefMut<'_, Color32>) {
        assert_eq!(Self::WIDTH, pixels.width());
        assert_eq!(Self::HEIGHT, pixels.height());
        for (index, color) in self.pixels.pixels().zip(pixels.pixels_mut()) {
            *color = palette_color(index);
        }
    }
    pub fn set_pixel(&mut self, x: i32, y: i32, color: u8) {
        assert!((0..Self::WIDTH).contains(&(x as usize)));
        assert!((0..Self::HEIGHT).contains(&(y as usize)));
        self.pixels[(x as usize, y as usize)] = color;
    }
}

impl Default for Char {
    fn default() -> Self {
        let pixels = vec![0u8; Self::WIDTH * Self::HEIGHT];
        Self {
            pixels: ImgVec::new(pixels, Self::WIDTH, Self::HEIGHT),
        }
    }
}

pub struct VicImage {
    columns: usize,
    rows: usize,

    /// Character bitmaps and settings
    chars: Vec<Char>,

    /// The character at each position.
    /// Size: columns x rows.
    video: ImgVec<u16>,

    /// The true-color pixels from rendering `video` and `chars`.
    pixels: ImgVec<Color32>,
}

impl Default for VicImage {
    fn default() -> Self {
        let columns = 22;
        let rows = 23;
        let pixel_width = columns * Char::WIDTH;
        let pixel_height = rows * Char::HEIGHT;
        Self {
            columns,
            rows,
            chars: vec![Char::default()],
            video: ImgVec::new(vec![0u16; columns * rows], columns, rows),
            pixels: ImgVec::new(
                vec![Color32::BLACK; pixel_width * pixel_height],
                pixel_width,
                pixel_height,
            ),
        }
    }
}

impl VicImage {
    /// Get the width and height of the image in pixels.
    pub fn pixel_size(&self) -> (usize, usize) {
        (self.columns * Char::WIDTH, self.rows * Char::HEIGHT)
    }

    pub fn set_pixel(&mut self, x: i32, y: i32, color: u8) {
        let column = x / Char::WIDTH as i32;
        let row = y / Char::WIDTH as i32;
        if !(0..self.columns as i32).contains(&column) || !(0..self.rows as i32).contains(&row) {
            return;
        }
        let cx = x % Char::WIDTH as i32;
        let cy = y % Char::WIDTH as i32;
        let char_num = self.video[(column as usize, row as usize)];
        let mut char = self.chars[char_num as usize].clone();
        char.set_pixel(cx, cy, color);
        let new_char_num = self.chars.len();
        self.chars.push(char);
        self.video[(column as usize, row as usize)] = new_char_num as u16;
    }

    pub fn info(&self) -> String {
        let min = self.video.pixels().min().unwrap();
        let max = self.video.pixels().max().unwrap();
        let used = self.video.pixels().unique().count();
        format!("{} to {}, {} characters used", min, max, used)
    }

    /// Width of one pixel compared to its height.
    pub fn pixel_aspect_ratio(&self) -> f32 {
        2.0
    }

    /// Get the image as true-color pixels for rendering to screen.
    pub fn pixels<'a>(&'a mut self) -> &[Color32] {
        self.render();
        assert_eq!(self.pixels.stride(), self.pixel_size().0);
        self.pixels.buf()
    }

    fn render(&mut self) {
        let pixels = &mut self.pixels;
        for (row, chars) in self.video.rows().enumerate() {
            for (column, char_index) in chars.iter().enumerate() {
                let left = column * Char::WIDTH;
                let top = row * Char::HEIGHT;
                let char = &self.chars[*char_index as usize];
                char.render_to(pixels.sub_image_mut(left, top, Char::WIDTH, Char::HEIGHT));
            }
        }
    }
}

/// Get a color from the palette.
/// `index` must be in the range `0..PALETTE_SIZE`.
pub fn palette_color<T>(index: T) -> Color32
where
    T: Into<usize>,
{
    let rgb = PALETTE[index.into()];
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}
