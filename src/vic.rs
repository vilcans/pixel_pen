use bimap::BiMap;
use eframe::egui::Color32;
use image::{DynamicImage, GenericImage, GenericImageView, Pixel, RgbaImage};
use imgref::{ImgRefMut, ImgVec};
use itertools::Itertools;
use rgb::RGBA;
use serde::{Deserialize, Serialize};
use std::{
    cmp::Ordering,
    collections::HashMap,
    ops::{Index, IndexMut, RangeInclusive},
};

use crate::{color_operations, coords::Point, error::Error, image_operations};

mod serialization;

// From /usr/lib/vice/VIC20/vice.vpl
const PALETTE: [(u32, &str); 16] = [
    // 0xRRGGBB
    (0x000000, "Black"),
    (0xffffff, "White"),
    (0xf00000, "Red"),
    (0x00f0f0, "Cyan"),
    (0x600060, "Purple"),
    (0x00a000, "Green"),
    (0x0000f0, "Blue"),
    (0xd0d000, "Yellow"),
    (0xc0a000, "Orange"),
    (0xffa000, "Light Orange"),
    (0xf08080, "Pink"),
    (0x00ffff, "Light Cyan"),
    (0xff00ff, "Light Purple"),
    (0x00ff00, "Light Green"),
    (0x00a0ff, "Light Blue"),
    (0xffff00, "Light Yellow"),
];

/// Number of entries in the palette.
pub const PALETTE_SIZE: usize = 16;

/// Which colors are allowed as the "character" color.
pub const ALLOWED_CHAR_COLORS: RangeInclusive<u8> = 0..=7;

pub const GLOBAL_COLORS: [(usize, &str, RangeInclusive<u8>); 3] = [
    (0, "Background", 0..=15),
    (1, "Border", 0..=7),
    (2, "Aux", 0..=15),
];

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GlobalColors(pub [u8; 3]);

impl GlobalColors {
    pub const BACKGROUND: u32 = 0;
    pub const BORDER: u32 = 1;
    pub const AUX: u32 = 2;
}
impl Default for GlobalColors {
    fn default() -> Self {
        Self([0, 1, 2])
    }
}
impl Index<u32> for GlobalColors {
    type Output = u8;
    fn index(&self, index: u32) -> &Self::Output {
        &self.0[index as usize]
    }
}
impl IndexMut<u32> for GlobalColors {
    fn index_mut(&mut self, index: u32) -> &mut Self::Output {
        &mut self.0[index as usize]
    }
}

#[derive(Clone, Copy, Hash)]
pub struct Char {
    bits: [u8; 8],
    color: u8,
    multicolor: bool,
}

impl Char {
    pub const WIDTH: usize = 8;
    pub const HEIGHT: usize = 8;
    pub const EMPTY_BITMAP: [u8; Self::HEIGHT] = [0u8; Self::HEIGHT];

    pub fn new(bits: [u8; 8], color: u8) -> Self {
        assert!(ALLOWED_CHAR_COLORS.contains(&color));
        Self {
            bits,
            color,
            multicolor: true,
        }
    }

    pub fn from_image(image: &RgbaImage, left: u32, top: u32, colors: &GlobalColors) -> Self {
        let mut histogram = [0i32; PALETTE_SIZE];
        for (y, x) in (0..Self::WIDTH as u32).cartesian_product(0..Self::HEIGHT as u32) {
            let rgba = image.get_pixel(x + left, y + top).to_rgba();
            let rgb = color_operations::rgba_to_rgb(&rgba);
            let color = color_operations::closest_palette_entry(
                &rgb,
                ALLOWED_CHAR_COLORS.map(|i| palette_color(i)),
            );
            histogram[color] += 1;
        }
        let most_used = histogram
            .iter()
            .enumerate()
            .filter(|&(index, _count)| index as u32 != GlobalColors::BACKGROUND)
            .filter(|&(_index, count)| *count != 0)
            .sorted_by(|(_, count1), (_, count2)| count2.cmp(count1))
            .map(|(index, _)| index)
            .next();

        let mut char = Self::new(Self::EMPTY_BITMAP, 1);
        char.multicolor = false;

        if let Some(most_used) = most_used {
            for (y, x) in (0..Self::WIDTH as u32).cartesian_product(0..Self::HEIGHT as u32) {
                let rgba = image.get_pixel(x + left, y + top).to_rgba();
                let rgb = color_operations::rgba_to_rgb(&rgba);
                if color_operations::closest_palette_entry(
                    &rgb,
                    [
                        palette_color(colors[GlobalColors::BACKGROUND]),
                        palette_color(most_used),
                    ]
                    .iter()
                    .cloned(),
                ) == 1
                {
                    char.set_pixel(x as i32, y as i32, most_used as u8, &colors)
                };
            }
        }
        char
    }

    /// Return the 4 bit value as stored in color RAM.
    pub fn raw_nibble(&self) -> u8 {
        self.color + if self.multicolor { 8 } else { 0 }
    }

    fn render_to(&self, pixels: ImgRefMut<'_, Color32>, colors: &GlobalColors) {
        debug_assert_eq!(Self::WIDTH, pixels.width());
        debug_assert_eq!(Self::HEIGHT, pixels.height());
        if self.multicolor {
            self.render_multicolor(pixels, colors);
        } else {
            self.render_hires(pixels, colors);
        }
    }

    /// Render high resolution character (not multicolor).
    fn render_hires(&self, mut pixels: ImgRefMut<'_, Color32>, colors: &GlobalColors) {
        for (bits, pixel_row) in self.bits.iter().zip(pixels.rows_mut()) {
            for (b, p) in pixel_row.iter_mut().enumerate() {
                let index = if (bits & (0x80 >> b)) == 0 {
                    colors[GlobalColors::BACKGROUND]
                } else {
                    self.color
                };
                *p = palette_color(index);
            }
        }
    }

    /// Render multicolor character (low resolution).
    fn render_multicolor(&self, mut pixels: ImgRefMut<'_, Color32>, colors: &GlobalColors) {
        for (bits, pixel_row) in self.bits.iter().zip(pixels.rows_mut()) {
            let mut pixels = pixel_row.iter_mut();
            for b in (0..8).step_by(2) {
                let v = (bits >> (6 - b)) & 0b11;
                let index = match v {
                    0b00 => colors[GlobalColors::BACKGROUND],
                    0b01 => colors[GlobalColors::BORDER],
                    0b10 => self.color,
                    0b11 => colors[GlobalColors::AUX],
                    _ => unreachable!(),
                };
                let color = palette_color(index);
                *pixels.next().unwrap() = color;
                *pixels.next().unwrap() = color;
            }
        }
    }

    fn set_pixel(&mut self, x: i32, y: i32, color: u8, colors: &GlobalColors) {
        debug_assert!((0..Self::WIDTH).contains(&(x as usize)));
        debug_assert!((0..Self::HEIGHT).contains(&(y as usize)));
        if self.multicolor {
            self.set_pixel_multicolor(x, y, color, colors)
        } else {
            self.set_pixel_hires(x, y, color, colors)
        }
    }

    fn set_pixel_hires(&mut self, x: i32, y: i32, color: u8, colors: &GlobalColors) {
        let bit = 0x80u8 >> x;
        if color == colors[GlobalColors::BACKGROUND] {
            self.bits[y as usize] &= !bit;
        } else {
            self.bits[y as usize] |= bit;
            self.color = color;
        }
    }

    fn set_pixel_multicolor(&mut self, x: i32, y: i32, color: u8, colors: &GlobalColors) {
        let x = x & !1;
        let mask = 0xc0u8 >> x;
        let old = &self.bits[y as usize];
        self.bits[y as usize] = match () {
            _ if color == colors[GlobalColors::BACKGROUND] => old & !mask,
            _ if color == colors[GlobalColors::BORDER] => (old & !mask) | (mask & 0b01010101),
            _ if color == colors[GlobalColors::AUX] => old | mask,
            _ => {
                self.color = color;
                (old & !mask) | (mask & 0b10101010)
            }
        }
    }
}

impl Default for Char {
    fn default() -> Self {
        Self::new([0u8; 8], 1)
    }
}

pub struct VicImage {
    columns: usize,
    rows: usize,

    pub colors: GlobalColors,

    /// The character at each position.
    /// Size: columns x rows.
    video: ImgVec<Char>,

    /// Bitmap for each character
    bitmaps: BiMap<usize, [u8; 8]>,
}

impl Default for VicImage {
    fn default() -> Self {
        VicImage::new(22, 23)
    }
}

impl VicImage {
    pub fn new(columns: usize, rows: usize) -> Self {
        let video = ImgVec::new(vec![Char::default(); columns * rows], columns, rows);
        Self::with_content(video)
    }

    /// Create an image from video data.
    /// ## Arguments
    /// `video_chars`:  The character at each position. Size: columns x rows.
    /// `video_colors`: The color and multicolor bit at each position. Size: columns x rows.
    /// `characters`: Bitmap for each character.
    pub fn from_data(
        columns: usize,
        rows: usize,
        global_colors: GlobalColors,
        video_chars: Vec<usize>,
        video_colors: Vec<u8>,
        characters: HashMap<usize, [u8; Char::HEIGHT]>,
    ) -> Result<Self, Error> {
        let size = columns * rows;
        let raw_video: Vec<Char> = video_chars
            .iter()
            .zip(video_colors)
            .map(|(charnum, color)| {
                let bits = *characters.get(charnum).unwrap_or(&Char::EMPTY_BITMAP);
                Char {
                    bits,
                    color: color & 7,
                    multicolor: (color & 8) == 8,
                }
            })
            // Add padding in case video_colors is too short
            .chain(std::iter::repeat(Char::default()))
            .take(size)
            .collect();
        assert_eq!(size, raw_video.len());
        let video = ImgVec::new(raw_video, columns, rows);
        let mut bitmaps = BiMap::new();
        bitmaps.extend(characters);
        Ok(Self {
            columns,
            rows,
            colors: global_colors,
            video,
            bitmaps,
        })
    }

    pub fn with_content(video: ImgVec<Char>) -> Self {
        let columns = video.width();
        let rows = video.height();
        Self {
            columns,
            rows,
            colors: Default::default(),
            video,
            bitmaps: BiMap::new(),
        }
    }

    pub fn from_image(image: DynamicImage) -> Result<VicImage, Error> {
        let columns = (image.width() as usize + Char::WIDTH - 1) / Char::WIDTH;
        let rows = (image.height() as usize + Char::HEIGHT - 1) / Char::HEIGHT;

        // Extend to whole character cells and convert to RGBA
        let image = {
            let mut extended = RgbaImage::new(
                columns as u32 * Char::WIDTH as u32,
                rows as u32 * Char::HEIGHT as u32,
            );
            extended.copy_from(&image, 0, 0)?;
            extended
        };

        let global_colors = GlobalColors([0, 1, 2]);

        // Reused for each character
        let mut char_image = RgbaImage::new(Char::WIDTH as u32, Char::HEIGHT as u32);

        let video = (0..rows)
            .cartesian_product(0..columns)
            .map(|(row, column)| {
                char_image
                    .copy_from(
                        &image.view(
                            (column * Char::WIDTH) as u32,
                            (row * Char::HEIGHT) as u32,
                            Char::WIDTH as u32,
                            Char::HEIGHT as u32,
                        ),
                        0,
                        0,
                    )
                    .unwrap();
                let image = optimized_image(&char_image, &global_colors);
                Char::from_image(&image, 0, 0, &global_colors)
            })
            .collect();

        Ok(Self::with_content(ImgVec::new(video, columns, rows)))
    }

    /// Get the width and height of the image in pixels.
    pub fn pixel_size(&self) -> (usize, usize) {
        (self.columns * Char::WIDTH, self.rows * Char::HEIGHT)
    }

    /// Set a pixel at the given coordinates to a given color.
    pub fn set_pixel(&mut self, x: i32, y: i32, color: u8) -> Option<()> {
        let (column, row, cx, cy) = self.char_coordinates(x, y)?;
        self.video[(column, row)].set_pixel(cx, cy, color, &self.colors);
        Some(())
    }

    /// Change the character color at given pixel coordinates
    pub fn set_color(&mut self, x: i32, y: i32, color: u8) -> Option<()> {
        let (column, row, _, _) = self.char_coordinates(x, y)?;
        self.video[(column, row)].color = color;
        Some(())
    }

    /// Get the rectangle of the character at the given pixel coordinates.
    /// Returns (top left, width, height), or None if the coordinate is outside the image.
    pub fn character_box(&self, p: Point) -> Option<(Point, i32, i32)> {
        let (column, row, _, _) = self.char_coordinates(p.x, p.y)?;
        Some((
            Point {
                x: (column * Char::WIDTH) as i32,
                y: (row * Char::HEIGHT) as i32,
            },
            Char::WIDTH as i32,
            Char::HEIGHT as i32,
        ))
    }

    /// General information about the image
    pub fn image_info(&self) -> String {
        format!("{} characters used", self.bitmaps.len())
    }

    /// Information about the given pixel in the image
    pub fn pixel_info(&self, position: Point) -> String {
        if let Some((column, row, _cx, _cy)) = self.char_coordinates(position.x, position.y) {
            let char = &self.video[(column, row)];
            format!(
                "({}, {}): column {}, row {} {} color {}",
                position.x,
                position.y,
                column,
                row,
                if char.multicolor {
                    "multicolor"
                } else {
                    "high-res"
                },
                char.color
            )
        } else {
            String::new()
        }
    }

    /// Width of one pixel compared to its height.
    pub fn pixel_aspect_ratio(&self) -> f32 {
        2.0
    }

    pub fn update(&mut self) {
        self.bitmaps = self.map_characters();
    }

    /// Generate a mapping between character bitmaps and character numbers.
    pub fn map_characters(&self) -> BiMap<usize, [u8; 8]> {
        let mut map = BiMap::new();
        for char in self.video.pixels() {
            if map.get_by_right(&char.bits).is_some() {
                // Existing bitmap
            } else {
                let num = map.len();
                map.insert(num, char.bits);
            }
        }
        map
    }

    /// Render true color pixels for this image.
    pub fn render(&mut self, mut pixels: ImgRefMut<'_, Color32>) {
        assert_eq!(self.pixel_size(), (pixels.width(), pixels.height()));
        for (row, chars) in self.video.rows().enumerate() {
            for (column, char) in chars.iter().enumerate() {
                let left = column * Char::WIDTH;
                let top = row * Char::HEIGHT;
                char.render_to(
                    pixels.sub_image_mut(left, top, Char::WIDTH, Char::HEIGHT),
                    &self.colors,
                );
            }
        }
    }

    /// Given pixel coordinates, return column, row, and x and y inside the character.
    /// Returns None if the coordinates are outside the image.
    fn char_coordinates(&self, x: i32, y: i32) -> Option<(usize, usize, i32, i32)> {
        let column = x / Char::WIDTH as i32;
        let row = y / Char::WIDTH as i32;
        if !(0..self.columns as i32).contains(&column) || !(0..self.rows as i32).contains(&row) {
            return None;
        }
        let cx = x % Char::WIDTH as i32;
        let cy = y % Char::WIDTH as i32;
        Some((column as usize, row as usize, cx, cy))
    }
}

/// Get a color from the palette.
/// `index` must be in the range `0..PALETTE_SIZE`.
pub fn palette_color<T>(index: T) -> Color32
where
    T: Into<usize>,
{
    let rgb = PALETTE[index.into()].0;
    Color32::from_rgb((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
}

/// Get a color from the palette.
/// `index` must be in the range `0..PALETTE_SIZE`.
pub fn palette_color_rgba<T>(index: T) -> RGBA<u8>
where
    T: Into<usize>,
{
    let rgb = PALETTE[index.into()].0;
    RGBA::new((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 0xff)
}

/// Get the name of a color from the palette.
/// `index` must be in the range `0..PALETTE_SIZE`.
pub fn palette_entry_name<T>(index: T) -> &'static str
where
    T: Into<usize>,
{
    PALETTE[index.into()].1
}

fn optimized_image(original: &RgbaImage, global_colors: &GlobalColors) -> RgbaImage {
    let fixed_colors = [palette_color_rgba(global_colors[GlobalColors::BACKGROUND])];
    let (pixels, palette, _error) = ALLOWED_CHAR_COLORS
        .filter(|attempted_color| *attempted_color != global_colors[GlobalColors::BACKGROUND])
        .map(|attempted_color| {
            let mut palette = Vec::with_capacity(fixed_colors.len() + 1);
            palette.extend_from_slice(&fixed_colors);
            palette.push(palette_color_rgba(attempted_color));
            image_operations::palettize(original, &palette)
        })
        .min_by(|(_, _, error0), (_, _, error1)| {
            error0.partial_cmp(error1).unwrap_or(Ordering::Equal)
        })
        .unwrap();

    image_operations::depalettize(original.width(), original.height(), &pixels, &palette)
}
