use bimap::BiMap;
use bit_vec::BitVec;
use image::{imageops::FilterType, GenericImage, GenericImageView, RgbaImage};
use imgref::{ImgRef, ImgVec};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    ops::{Index, IndexMut, RangeInclusive},
};
use thiserror::Error;

use crate::{
    colors::TrueColor,
    coords::Point,
    error::{DisallowedAction, Error},
    image_operations,
    update_area::UpdateArea,
};

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

const RAW_HIRES_BACKGROUND: TrueColor = TrueColor::from_u32(0x555555);
const RAW_HIRES_CHAR_COLOR: TrueColor = TrueColor::from_u32(0xeeeeee);

const RAW_MULTICOLOR_BACKGROUND: TrueColor = TrueColor::from_u32(0x000000);
const RAW_MULTICOLOR_BORDER: TrueColor = TrueColor::from_u32(0x0044ff);
const RAW_MULTICOLOR_AUX: TrueColor = TrueColor::from_u32(0xff0000);
const RAW_MULTICOLOR_CHAR_COLOR: TrueColor = TrueColor::from_u32(0xffffff);

/// Which colors are allowed as the "character" color.
pub const ALLOWED_CHAR_COLORS: RangeInclusive<u8> = 0..=7;

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

#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PaintColor {
    Background,
    Border,
    Aux,
    CharColor(u8),
}

impl Default for PaintColor {
    fn default() -> Self {
        Self::CharColor(2)
    }
}

impl PaintColor {
    pub fn selectable_colors(&self) -> impl Iterator<Item = u8> {
        match self {
            PaintColor::Background => 0..=15,
            PaintColor::Border => 0..=7,
            PaintColor::Aux => 0..=15,
            PaintColor::CharColor(index) => *index..=*index,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum ColorFormat {
    HighRes,
    Multicolor,
}

#[derive(Clone, PartialEq)]
pub enum ViewSettings {
    Normal,
    Raw,
}
impl Default for ViewSettings {
    fn default() -> Self {
        ViewSettings::Normal
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
    pub const DEFAULT_BRUSH: Char = Char {
        bits: [0xff; 8],
        color: 1,
        multicolor: false,
    };

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

    fn render(
        &self,
        colors: &GlobalColors,
        settings: &ViewSettings,
    ) -> [TrueColor; Self::WIDTH * Self::HEIGHT] {
        if self.multicolor {
            let (background, border, aux, char_color) = match settings {
                ViewSettings::Normal => (
                    palette_color(colors[GlobalColors::BACKGROUND]),
                    palette_color(colors[GlobalColors::BORDER]),
                    palette_color(colors[GlobalColors::AUX]),
                    palette_color(self.color),
                ),
                ViewSettings::Raw => (
                    RAW_MULTICOLOR_BACKGROUND,
                    RAW_MULTICOLOR_BORDER,
                    RAW_MULTICOLOR_AUX,
                    RAW_MULTICOLOR_CHAR_COLOR,
                ),
            };
            Self::render_multicolor(&self.bits, background, border, aux, char_color)
        } else {
            let (background, char_color) = match settings {
                ViewSettings::Normal => (
                    palette_color(colors[GlobalColors::BACKGROUND]),
                    palette_color(self.color),
                ),
                ViewSettings::Raw => (RAW_HIRES_BACKGROUND, RAW_HIRES_CHAR_COLOR),
            };
            Self::render_hires(&self.bits, background, char_color)
        }
    }

    /// Render high resolution character (not multicolor).
    fn render_hires<Pixel>(
        bitmap: &[u8; Self::HEIGHT],
        background: Pixel,
        char_color: Pixel,
    ) -> [Pixel; Self::WIDTH * Self::HEIGHT]
    where
        Pixel: Copy + Default,
    {
        let mut pixels = [Pixel::default(); Self::WIDTH * Self::HEIGHT];
        let mut pixel_iter = pixels.iter_mut();
        for bits in bitmap.iter() {
            for b in 0..Self::WIDTH {
                *pixel_iter.next().unwrap() = if (bits & (0x80 >> b)) == 0 {
                    background
                } else {
                    char_color
                };
            }
        }
        pixels
    }

    /// Render multicolor character (low resolution).
    fn render_multicolor<Pixel>(
        bitmap: &[u8; Self::HEIGHT],
        background: Pixel,
        border: Pixel,
        aux: Pixel,
        char_color: Pixel,
    ) -> [Pixel; Self::WIDTH * Self::HEIGHT]
    where
        Pixel: Copy + Default,
    {
        let mut pixels = [Pixel::default(); Self::WIDTH * Self::HEIGHT];
        let mut pixel_iter = pixels.iter_mut();
        for bits in bitmap.iter() {
            for b in (0..8).step_by(2) {
                let v = (bits >> (6 - b)) & 0b11;
                let index = match v {
                    0b00 => background,
                    0b01 => border,
                    0b10 => char_color,
                    0b11 => aux,
                    _ => unreachable!(),
                };
                *pixel_iter.next().unwrap() = index;
                *pixel_iter.next().unwrap() = index;
            }
        }
        pixels
    }

    pub fn mutate_pixels<F>(
        &mut self,
        mask: &BitVec,
        operation: F,
    ) -> Result<bool, Box<dyn DisallowedAction>>
    where
        F: Fn(PaintColor) -> PaintColor,
    {
        if self.multicolor {
            self.mutate_pixels_multicolor(mask, operation)
        } else {
            self.mutate_pixels_hires(mask, operation)
        }
    }

    fn mutate_pixels_multicolor<F>(
        &mut self,
        mask: &BitVec,
        operation: F,
    ) -> Result<bool, Box<dyn DisallowedAction>>
    where
        F: Fn(PaintColor) -> PaintColor,
    {
        let mut changed = false;
        let mut new_color = self.color;

        for cy in 0..Self::HEIGHT {
            let mut new_bits = self.bits[cy];
            for cx in (0..Self::WIDTH).step_by(2) {
                let shift = 6 - cx;
                if mask[cx + cy * Self::WIDTH] || mask[cx + cy * Self::WIDTH + 1] {
                    let current = match (self.bits[cy] >> shift) & 0b11 {
                        0b00 => PaintColor::Background,
                        0b01 => PaintColor::Border,
                        0b10 => PaintColor::CharColor(self.color),
                        0b11 => PaintColor::Aux,
                        _ => unreachable!(),
                    };
                    let to_set = match operation(current) {
                        PaintColor::Background => 0b00,
                        PaintColor::Border => 0b01,
                        PaintColor::CharColor(c) => {
                            new_color = c;
                            0b10
                        }
                        PaintColor::Aux => 0b11,
                    };
                    new_bits &= !(0b11 << shift);
                    new_bits |= to_set << shift;
                }
            }
            if new_bits != self.bits[cy] {
                self.bits[cy] = new_bits;
                changed = true;
            }
        }
        if new_color != self.color {
            self.color = new_color;
            changed = true;
        }
        Ok(changed)
    }

    fn mutate_pixels_hires<F>(
        &mut self,
        mask: &BitVec,
        operation: F,
    ) -> Result<bool, Box<dyn DisallowedAction>>
    where
        F: Fn(PaintColor) -> PaintColor,
    {
        let mut changed = false;
        let mut new_color = self.color;

        for cy in 0..Self::HEIGHT {
            let mut new_bits = self.bits[cy];
            for cx in 0..Self::WIDTH {
                let bytemask = 0x80 >> cx;
                if let Some(true) = mask.get(cx + cy * Self::WIDTH) {
                    let current = match self.bits[cy] & bytemask {
                        0 => PaintColor::Background,
                        _ => PaintColor::CharColor(self.color),
                    };
                    match operation(current) {
                        PaintColor::Background => new_bits &= !bytemask,
                        PaintColor::CharColor(c) => {
                            new_color = c;
                            new_bits |= bytemask;
                        }
                        _ => return Err(Box::new(DisallowedEdit::DisallowedHiresColor)),
                    }
                }
            }
            if new_bits != self.bits[cy] {
                self.bits[cy] = new_bits;
                changed = true;
            }
        }
        if new_color != self.color {
            self.color = new_color;
            changed = true;
        }
        Ok(changed)
    }

    fn make_high_res(&mut self) -> Result<bool, Box<dyn DisallowedAction>> {
        if !self.multicolor {
            return Ok(false);
        }
        self.multicolor = false;
        Ok(true)
    }

    fn make_multicolor(&mut self) -> Result<bool, Box<dyn DisallowedAction>> {
        if self.multicolor {
            return Ok(false);
        }
        self.multicolor = true;
        Ok(true)
    }
}

impl Default for Char {
    fn default() -> Self {
        Self::new([0u8; 8], 1)
    }
}

#[derive(Clone)]
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

    pub fn color_index_from_paint_color(&self, c: &PaintColor) -> u8 {
        match c {
            PaintColor::Background => self.colors[GlobalColors::BACKGROUND],
            PaintColor::Border => self.colors[GlobalColors::BORDER],
            PaintColor::Aux => self.colors[GlobalColors::AUX],
            PaintColor::CharColor(index) => *index,
        }
    }

    pub fn true_color_from_paint_color(&self, c: &PaintColor) -> TrueColor {
        palette_color(self.color_index_from_paint_color(c))
    }

    /// Get the width and height of the image in pixels.
    pub fn pixel_size(&self) -> (usize, usize) {
        (self.columns * Char::WIDTH, self.rows * Char::HEIGHT)
    }

    /// Paste characters into the image.
    /// `target_column` and `target_row` is the top-left corner.
    /// The extents of the pasted chars may be outside the image (they are clipped).
    pub fn paste_chars(
        &mut self,
        target_column: i32,
        target_row: i32,
        source: ImgRef<'_, Char>,
    ) -> Result<bool, Box<dyn DisallowedAction>> {
        let mut changed = false;
        for (char, (r, c)) in source.pixels().zip(
            (target_row..target_row + source.height() as i32)
                .cartesian_product(target_column..target_column + source.width() as i32),
        ) {
            if (0..self.columns as i32).contains(&c) && (0..self.rows as i32).contains(&r) {
                self.video[(c as usize, r as usize)] = char;
                changed = true;
            }
        }
        Ok(changed)
    }

    fn apply_operation_to_pixels<F>(
        &mut self,
        target: &UpdateArea,
        operation: F,
    ) -> Result<bool, Box<dyn DisallowedAction>>
    where
        F: Fn(PaintColor) -> PaintColor,
    {
        let mut changed = false;
        for ((column, row), mask) in self.cells_and_pixels(target) {
            let char = &mut self.video[(column, row)];
            changed |= char.mutate_pixels(&mask, &operation)?;
        }
        Ok(changed)
    }

    fn apply_operation_to_cells<F>(
        &mut self,
        target: &UpdateArea,
        operation: F,
    ) -> Result<bool, Box<dyn DisallowedAction>>
    where
        F: Fn(PaintColor) -> PaintColor,
    {
        let mut changed = false;
        let mask = BitVec::from_elem(Char::WIDTH * Char::HEIGHT, true);
        for (column, row) in self.target_cells(target) {
            let char = &mut self.video[(column, row)];
            changed |= char.mutate_pixels(&mask, &operation)?;
        }
        Ok(changed)
    }

    /// Standard draw tool. Draw single pixels.
    pub fn plot(
        &mut self,
        target: &UpdateArea,
        color: PaintColor,
    ) -> Result<bool, Box<dyn DisallowedAction>> {
        self.apply_operation_to_pixels(target, |_| color)
    }

    /// Fill the whole cell with a given color
    pub fn fill_cells(
        &mut self,
        target: &UpdateArea,
        color: PaintColor,
    ) -> Result<bool, Box<dyn DisallowedAction>> {
        self.apply_operation_to_cells(target, |_| color)
    }

    /// Replace one color with another.
    pub fn replace_color(
        &mut self,
        target: &UpdateArea,
        to_replace: PaintColor,
        replacement: PaintColor,
    ) -> Result<bool, Box<dyn DisallowedAction>> {
        self.apply_operation_to_pixels(
            target,
            |old| if old == to_replace { replacement } else { old },
        )
    }

    /// Swap two colors
    pub fn swap_colors(
        &mut self,
        target: &UpdateArea,
        color_1: PaintColor,
        color_2: PaintColor,
    ) -> Result<bool, Box<dyn DisallowedAction>> {
        self.apply_operation_to_pixels(target, |old| {
            if old == color_1 {
                color_2
            } else if old == color_2 {
                color_1
            } else {
                old
            }
        })
    }

    /// Change the character color of cells
    pub fn set_color(
        &mut self,
        target: &UpdateArea,
        color: u8,
    ) -> Result<bool, Box<dyn DisallowedAction>> {
        if !ALLOWED_CHAR_COLORS.contains(&color) {
            return Err(Box::new(DisallowedEdit::DisallowedCharacterColor));
        }
        let mut changed = false;
        for (col, row) in self.target_cells(target) {
            let cell = &mut self.video[(col, row)];
            if cell.color != color {
                cell.color = color;
                changed = true;
            }
        }
        Ok(changed)
    }

    pub fn make_high_res(
        &mut self,
        target: &UpdateArea,
    ) -> Result<bool, Box<dyn DisallowedAction>> {
        let mut changed = false;
        for (column, row) in self.target_cells(target) {
            changed |= self.video[(column, row)].make_high_res()?;
        }
        Ok(changed)
    }

    pub fn make_multicolor(
        &mut self,
        target: &UpdateArea,
    ) -> Result<bool, Box<dyn DisallowedAction>> {
        let mut changed = false;
        for (column, row) in self.target_cells(target) {
            changed |= self.video[(column, row)].make_multicolor()?;
        }
        Ok(changed)
    }

    /// Get a rectangle in pixel coordinates from a rectangle in character cells.
    /// Returns the top left, and bottom right (exclusive) of the rectangle in image pixels.
    /// Accepts coordinates outside the image.
    pub fn cell_rectangle(&self, column: i32, row: i32, width: u32, height: u32) -> (Point, Point) {
        let x = column * Char::WIDTH as i32;
        let y = row * Char::HEIGHT as i32;
        (
            Point { x, y },
            Point {
                x: x + Char::WIDTH as i32 * width as i32,
                y: y + Char::HEIGHT as i32 * height as i32,
            },
        )
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
        self.render_with_settings(&ViewSettings::default())
    }

    pub fn render_with_settings(&self, settings: &ViewSettings) -> RgbaImage {
        let (source_width, source_height) = self.pixel_size();
        let mut image = RgbaImage::new(source_width as u32, source_height as u32);
        for (row, chars) in self.video.rows().enumerate() {
            for (column, char) in chars.iter().enumerate() {
                let char_pixels = char.render(&self.colors, settings);
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

    /// Get a copy of the characters in a rectangular area.
    pub fn grab_cells(
        &self,
        column: usize,
        row: usize,
        width: usize,
        height: usize,
    ) -> ImgVec<Char> {
        let chars = self
            .video
            .sub_image(column, row, width, height)
            .pixels()
            .collect();
        ImgVec::new(chars, width, height)
    }

    /// Given pixel coordinates, return column, row, and x and y inside the character.
    /// May return coordinates outside the image.
    pub fn char_coordinates_unclipped(&self, x: i32, y: i32) -> (i32, i32, i32, i32) {
        let column = x.div_euclid(Char::WIDTH as i32);
        let cx = x.rem_euclid(Char::WIDTH as i32);
        let row = y.div_euclid(Char::HEIGHT as i32);
        let cy = y.rem_euclid(Char::HEIGHT as i32);
        (column, row, cx, cy)
    }

    /// Given pixel coordinates, return column, row, and x and y inside the character.
    /// Returns None if the coordinates are outside the image.
    pub fn char_coordinates(&self, x: i32, y: i32) -> Option<(usize, usize, i32, i32)> {
        let (width, height) = self.pixel_size();
        if (0..width as i32).contains(&x) && (0..height as i32).contains(&y) {
            let (column, row, cx, cy) = self.char_coordinates_unclipped(x, y);
            Some((column as usize, row as usize, cx, cy))
        } else {
            None
        }
    }

    /// Given pixel coordinates, return column, row, and x and y inside the character.
    /// If the arguments are outside the image, they are clamped to be inside it.
    pub fn char_coordinates_clamped(&self, x: i32, y: i32) -> (usize, usize, i32, i32) {
        let (width, height) = self.pixel_size();
        let (column, row, cx, cy) = self.char_coordinates_unclipped(
            x.clamp(0, width as i32 - 1),
            y.clamp(0, height as i32 - 1),
        );
        (column as usize, row as usize, cx, cy)
    }

    /// Get the character cells to update given an UpdateArea.
    /// Returns the columns and rows of the cells within this image's bounds.
    fn target_cells(&self, target: &UpdateArea) -> Vec<(u32, u32)> {
        self.cells_and_pixels(target)
            .iter()
            .map(|(cell, _)| cell)
            .copied()
            .collect()
    }

    /// Get the character cells to update, and which pixel in each cell to update.
    fn cells_and_pixels(&self, target: &UpdateArea) -> HashMap<(u32, u32), BitVec> {
        target.cells_and_pixels(
            Char::WIDTH as u32,
            Char::HEIGHT as u32,
            self.columns as u32,
            self.rows as u32,
        )
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
    #[error("Character color must be between 0 and 7")]
    DisallowedCharacterColor,
}

impl DisallowedAction for DisallowedEdit {}
