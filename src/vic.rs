use bimap::BiMap;
use image::{imageops::FilterType, GenericImage, GenericImageView, RgbaImage};
use imgref::{ImgRef, ImgVec};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ops::{Index, IndexMut, RangeInclusive},
};
use thiserror::Error;

use crate::{colors::TrueColor, coords::Point, error::Error, image_operations};

mod serialization;

// From /usr/lib/vice/VIC20/vice.vpl
const PALETTE: [(u32, &str); 16] = [
    // 0xRRGGBB
    (0x000000, "Black"),
    (0xffffff, "White"),
    (0x6d2327, "Red"),
    (0xa0fef8, "Cyan"),
    (0x8e3c97, "Purple"),
    (0x7eda75, "Green"),
    (0x252390, "Blue"),
    (0xffff86, "Yellow"),
    (0xa4643b, "Orange"),
    (0xffc8a1, "Light Orange"),
    (0xf2a7ab, "Pink"),
    (0xdbffff, "Light Cyan"),
    (0xffb4ff, "Light Purple"),
    (0xd7ffce, "Light Green"),
    (0x9d9aff, "Light Blue"),
    (0xffffc9, "Light Yellow"),
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
impl IntoIterator for GlobalColors {
    type Item = u8;
    type IntoIter = GlobalColorsIntoIterator;

    fn into_iter(self) -> Self::IntoIter {
        Self::IntoIter {
            value: self,
            index: 0,
        }
    }
}
pub struct GlobalColorsIntoIterator {
    value: GlobalColors,
    index: u32,
}
impl Iterator for GlobalColorsIntoIterator {
    type Item = u8;
    fn next(&mut self) -> Option<Self::Item> {
        if (self.index as usize) < self.value.0.len() {
            let c = self.value[self.index];
            self.index += 1;
            Some(c)
        } else {
            None
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum ColorFormat {
    HighRes,
    Multicolor,
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

    /// Create a new multicolor character
    pub fn new(bits: [u8; Self::HEIGHT], color: u8) -> Self {
        assert!(ALLOWED_CHAR_COLORS.contains(&color));
        Self {
            bits,
            color,
            multicolor: true,
        }
    }

    /// Create a new high resolution character
    pub fn new_highres(bits: [u8; Self::HEIGHT], color: u8) -> Self {
        assert!(ALLOWED_CHAR_COLORS.contains(&color));
        Self {
            bits,
            color,
            multicolor: false,
        }
    }

    pub fn highres_from_colors(colors: ImgRef<'_, u8>, global_colors: &GlobalColors) -> Self {
        assert_eq!(colors.width(), Self::WIDTH);
        assert_eq!(colors.height(), Self::HEIGHT);

        let mut cell_color = 1u8; // the final color of the cell
        let bg_color = global_colors[GlobalColors::BACKGROUND] as u8;
        let mut bitmap = [0u8; Self::HEIGHT];

        for (pixel_row, bits) in colors.rows().zip(bitmap.iter_mut()) {
            *bits = pixel_row
                .iter()
                .copied()
                .enumerate()
                .map(|(x, pixel_color)| {
                    if pixel_color == bg_color {
                        0
                    } else {
                        cell_color = pixel_color; // Use the last found color as the color for the cell
                        0x80u8 >> x
                    }
                })
                .sum();
        }

        Self::new_highres(bitmap, cell_color)
    }

    pub fn multicolor_from_colors(colors: ImgRef<'_, u8>, global_colors: &GlobalColors) -> Self {
        assert_eq!(colors.width(), Self::WIDTH / 2);
        assert_eq!(colors.height(), Self::HEIGHT);

        let mut cell_color = 1u8; // the final color of the cell
        let bg_color = global_colors[GlobalColors::BACKGROUND] as u8;
        let aux_color = global_colors[GlobalColors::AUX] as u8;
        let border_color = global_colors[GlobalColors::BORDER] as u8;
        let mut bitmap = [0u8; Self::HEIGHT];

        for (pixel_row, bits) in colors.rows().zip(bitmap.iter_mut()) {
            *bits = pixel_row
                .iter()
                .copied()
                .enumerate()
                .map(|(x, pixel_color)| {
                    if pixel_color == bg_color {
                        0
                    } else if pixel_color == border_color {
                        0x40u8 >> (x * 2)
                    } else if pixel_color == aux_color {
                        0xc0u8 >> (x * 2)
                    } else {
                        cell_color = pixel_color; // Use the last found color as the color for the cell
                        0x80u8 >> (x * 2)
                    }
                })
                .sum();
        }
        Self::new(bitmap, cell_color)
    }

    /// Return the 4 bit value as stored in color RAM.
    pub fn raw_nibble(&self) -> u8 {
        self.color + if self.multicolor { 8 } else { 0 }
    }

    fn render(&self, colors: &GlobalColors) -> [TrueColor; Self::WIDTH * Self::HEIGHT] {
        if self.multicolor {
            self.render_multicolor(colors)
        } else {
            self.render_hires(colors)
        }
    }

    /// Render high resolution character (not multicolor).
    fn render_hires(&self, colors: &GlobalColors) -> [TrueColor; Self::WIDTH * Self::HEIGHT] {
        let mut pixels = [TrueColor::default(); Self::WIDTH * Self::HEIGHT];
        let mut pixel_iter = pixels.iter_mut();
        for bits in self.bits.iter() {
            for b in 0..Self::WIDTH {
                let index = if (bits & (0x80 >> b)) == 0 {
                    colors[GlobalColors::BACKGROUND]
                } else {
                    self.color
                };
                *pixel_iter.next().unwrap() = palette_color(index);
            }
        }
        pixels
    }

    /// Render multicolor character (low resolution).
    fn render_multicolor(&self, colors: &GlobalColors) -> [TrueColor; Self::WIDTH * Self::HEIGHT] {
        let mut pixels = [TrueColor::default(); Self::WIDTH * Self::HEIGHT];
        let mut pixel_iter = pixels.iter_mut();
        for bits in self.bits.iter() {
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
                *pixel_iter.next().unwrap() = color;
                *pixel_iter.next().unwrap() = color;
            }
        }
        pixels
    }

    fn set_pixel(
        &mut self,
        x: i32,
        y: i32,
        color: u8,
        colors: &GlobalColors,
    ) -> Result<bool, DisallowedEdit> {
        debug_assert!((0..Self::WIDTH).contains(&(x as usize)));
        debug_assert!((0..Self::HEIGHT).contains(&(y as usize)));
        let old_bits = self.bits[y as usize];
        let new_bits;
        let mut new_color = self.color;
        if self.multicolor {
            let mask = 0xc0u8 >> (x & !1);
            match () {
                _ if color == colors[GlobalColors::BACKGROUND] => {
                    new_bits = old_bits & !mask;
                }
                _ if color == colors[GlobalColors::BORDER] => {
                    new_bits = (old_bits & !mask) | (mask & 0b01010101)
                }
                _ if color == colors[GlobalColors::AUX] => new_bits = old_bits | mask,
                _ if ALLOWED_CHAR_COLORS.contains(&color) => {
                    new_bits = (old_bits & !mask) | (mask & 0b10101010);
                    new_color = color;
                }
                _ => return Err(DisallowedEdit::DisallowedMulticolorColor),
            }
        } else {
            let bit = 0x80u8 >> x;
            if color == colors[GlobalColors::BACKGROUND] {
                new_bits = old_bits & !bit;
            } else if ALLOWED_CHAR_COLORS.contains(&color) {
                new_bits = old_bits | bit;
                new_color = color;
            } else {
                return Err(DisallowedEdit::DisallowedHiresColor);
            }
        }
        if new_bits != old_bits || new_color != self.color {
            self.bits[y as usize] = new_bits;
            self.color = new_color;
            Ok(true)
        } else {
            Ok(false)
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

    pub fn from_image(source_image: &RgbaImage) -> Result<VicImage, Error> {
        let columns = (source_image.width() as usize + Char::WIDTH - 1) / Char::WIDTH;
        let rows = (source_image.height() as usize + Char::HEIGHT - 1) / Char::HEIGHT;
        let mut image = VicImage::new(columns, rows);
        image.paste_image(source_image, 0, 0, ColorFormat::Multicolor);
        Ok(image)
    }

    /// Paste a true color image into this image.
    pub fn paste_image(
        &mut self,
        source: &RgbaImage,
        target_x: i32,
        target_y: i32,
        format: ColorFormat,
    ) {
        const CELL_W: i32 = Char::WIDTH as i32;
        const CELL_H: i32 = Char::HEIGHT as i32;
        let start_column = (target_x / CELL_W as i32).max(0);
        let end_column =
            ((target_x + source.width() as i32 + CELL_W - 1) / CELL_W).min(self.columns as i32);
        let start_row = (target_y / CELL_H as i32).max(0);
        let end_row =
            ((target_y + source.height() as i32 + CELL_H - 1) / CELL_H).min(self.rows as i32);

        let global_colors = &self.colors;

        for (r, c) in (start_row..end_row).cartesian_product(start_column..end_column) {
            let left = (c * CELL_W) - target_x;
            let top = (r * CELL_H) - target_y;
            let right = left + CELL_W;
            let bottom = top + CELL_H;
            let clamped_left = i32::max(0, left);
            let clamped_top = i32::max(0, top);
            let clamped_right = i32::min(source.width() as i32, right);
            let clamped_bottom = i32::min(source.height() as i32, bottom);

            let mut char_image = RgbaImage::new(Char::WIDTH as u32, Char::HEIGHT as u32);
            char_image
                .copy_from(
                    &source.view(
                        clamped_left as u32,
                        clamped_top as u32,
                        (clamped_right - clamped_left) as u32,
                        (clamped_bottom - clamped_top) as u32,
                    ),
                    (clamped_left - left) as u32,
                    (clamped_top - top) as u32,
                )
                .unwrap();

            self.video[(c as usize, r as usize)] = match format {
                ColorFormat::HighRes => {
                    let colors = optimized_image_highres(&char_image, global_colors);
                    Char::highres_from_colors(colors.as_ref(), global_colors)
                }
                ColorFormat::Multicolor => {
                    let half_width = image::imageops::resize(
                        &char_image,
                        Char::WIDTH as u32 / 2,
                        Char::HEIGHT as u32,
                        FilterType::Triangle,
                    );
                    let colors = optimized_image_multicolor(&half_width, global_colors);
                    Char::multicolor_from_colors(colors.as_ref(), global_colors)
                }
            }
        }
    }

    /// Get the width and height of the image in pixels.
    pub fn pixel_size(&self) -> (usize, usize) {
        (self.columns * Char::WIDTH, self.rows * Char::HEIGHT)
    }

    /// Set a pixel at the given coordinates to a given color.
    pub fn set_pixel(&mut self, x: i32, y: i32, color: u8) -> Result<bool, DisallowedEdit> {
        if let Some((column, row, cx, cy)) = self.char_coordinates(x, y) {
            self.video[(column, row)].set_pixel(cx, cy, color, &self.colors)
        } else {
            Ok(false)
        }
    }

    /// Change the character color at given pixel coordinates
    pub fn set_color(&mut self, x: i32, y: i32, color: u8) -> Result<bool, DisallowedEdit> {
        if let Some((column, row, _cx, _cy)) = self.char_coordinates(x, y) {
            if !ALLOWED_CHAR_COLORS.contains(&color) {
                return Err(DisallowedEdit::DisallowedCharacterColor);
            }
            let cell = &mut self.video[(column, row)];
            if cell.color == color {
                Ok(false)
            } else {
                cell.color = color;
                Ok(true)
            }
        } else {
            Ok(false)
        }
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

    /// Get at which pixel coordinates to dispay grid lines
    pub fn vertical_grid_lines(&self) -> impl Iterator<Item = i32> {
        (0..=self.columns).map(|c| (c * Char::WIDTH) as i32)
    }

    /// Get at which pixel coordinates to dispay grid lines
    pub fn horizontal_grid_lines(&self) -> impl Iterator<Item = i32> {
        (0..=self.rows).map(|r| (r * Char::HEIGHT) as i32)
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
        // I measured the 176x184 pixels of the Vic-20 screen,
        // which was 573x362 mm on my TV, giving this ratio:
        1.654822
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

    pub fn border(&self) -> TrueColor {
        let i = self.colors[GlobalColors::BORDER];
        palette_color(i)
    }

    /// Render true color pixels for this image.
    pub fn render(&self) -> RgbaImage {
        let (source_width, source_height) = self.pixel_size();
        let mut image = RgbaImage::new(source_width as u32, source_height as u32);
        for (row, chars) in self.video.rows().enumerate() {
            for (column, char) in chars.iter().enumerate() {
                let char_pixels = char.render(&self.colors);
                let left = column as u32 * Char::WIDTH as u32;
                let top = row as u32 * Char::HEIGHT as u32;
                for ((y, x), s) in ((0..Char::HEIGHT as u32)
                    .cartesian_product(0..Char::WIDTH as u32))
                .zip(char_pixels.iter())
                {
                    image.put_pixel(x + left, y + top, (*s).into());
                }
            }
        }
        image
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
pub fn palette_color<T>(index: T) -> TrueColor
where
    T: Into<usize>,
{
    let rgb = PALETTE[index.into()].0;
    TrueColor::from_u32(rgb)
}

/// Get the name of a color from the palette.
/// `index` must be in the range `0..PALETTE_SIZE`.
pub fn palette_entry_name<T>(index: T) -> &'static str
where
    T: Into<usize>,
{
    PALETTE[index.into()].1
}

/// Generates an optimized highres image using the given hardware palette colors.
/// Tries different colors and finds the one that gives the least quantization error.
/// Returns the resulting color numbers.
fn optimized_image_highres(original: &RgbaImage, global_colors: &GlobalColors) -> ImgVec<u8> {
    let fixed_colors = [global_colors[GlobalColors::BACKGROUND]];
    optimized_image(original, &fixed_colors)
}

/// Generates an optimized multicolor image using the given hardware palette colors.
/// Tries different colors and finds the one that gives the least quantization error.
/// Returns the resulting color numbers.
fn optimized_image_multicolor(original: &RgbaImage, global_colors: &GlobalColors) -> ImgVec<u8> {
    let fixed_colors = [
        global_colors[GlobalColors::BACKGROUND],
        global_colors[GlobalColors::BORDER],
        global_colors[GlobalColors::AUX],
    ];
    optimized_image(original, &fixed_colors)
}

/// Generate an image by attempting different color settings and finding the one that gives the least error.
/// Tries different character colors and finds the one that gives the least quantization error.
/// The colors in `fixed_colors` will be used in every attempt, in addition to the varying character color.
fn optimized_image(
    original: &image::ImageBuffer<image::Rgba<u8>, Vec<u8>>,
    fixed_colors: &[u8],
) -> imgref::Img<Vec<u8>> {
    let (pixels, colors, _error) = ALLOWED_CHAR_COLORS
        .filter(|attempted_color| !fixed_colors.contains(attempted_color))
        .map(|attempted_color| {
            // Generate a list of the color combinations to try
            let mut colors = Vec::with_capacity(fixed_colors.len() + 1);
            colors.extend_from_slice(fixed_colors);
            colors.push(attempted_color);
            // Generate RGBA palette from those colors.
            let palette = colors
                .iter()
                .map(|&c| palette_color(c as usize))
                .collect::<Vec<_>>();
            let (pixels, error) = image_operations::palettize(original, &palette);
            (pixels, colors, error)
        })
        .min_by(|(_, _, error0), (_, _, error1)| error0.partial_cmp(error1).unwrap())
        .unwrap();

    ImgVec::new(
        pixels.iter().map(|&c| colors[c as usize]).collect(),
        original.width() as usize,
        original.height() as usize,
    )
}

#[allow(clippy::enum_variant_names)] // All variants have the same prefix (Disallowed)
#[derive(Error, Debug)]
pub enum DisallowedEdit {
    #[error("High resolution characters can be painted with color 0-7, or background")]
    DisallowedHiresColor,
    #[error("Multicolor characters can be painted with color 0-7, background, border, or aux")]
    DisallowedMulticolorColor,
    #[error("Character color must be between 0 and 7")]
    DisallowedCharacterColor,
}
