use super::{char::Char, ColorFormat, DisallowedEdit, GlobalColors, PixelColor, VicPalette};
use crate::{
    colors::TrueColor,
    coords::{CellPos, CellRect, Point, SizeInCells, WithinBounds},
    error::{DisallowedAction, Error},
    image_operations,
    ui::ViewSettings,
    update_area::UpdateArea,
};
use bimap::BiMap;
use bit_vec::BitVec;
use image::{imageops::FilterType, GenericImage, GenericImageView, RgbaImage};
use imgref::{ImgRef, ImgVec};
use itertools::Itertools;
use std::collections::HashMap;

#[derive(Clone)]
pub struct VicImage {
    pub(super) colors: GlobalColors,

    /// The character at each position.
    /// Size: columns x rows.
    pub(super) video: ImgVec<Char>,

    /// Bitmap for each character
    bitmaps: BiMap<usize, [u8; 8]>,
}

impl Default for VicImage {
    fn default() -> Self {
        VicImage::new(22, 23)
    }
}

impl VicImage {
    pub const MAX_SIZE: SizeInCells = SizeInCells {
        width: 10000,
        height: 10000,
    };

    pub fn new(columns: usize, rows: usize) -> Self {
        let video = ImgVec::new(vec![Char::default(); columns * rows], columns, rows);
        Self::with_content(video)
    }

    /// Create an image from video data.
    /// ## Arguments
    /// `video_chars`:  The character at each position. Size: `size`.
    /// `video_colors`: The color and multicolor bit at each position. Size: `size`.
    /// `characters`: Bitmap for each character.
    pub fn from_data(
        size: SizeInCells,
        global_colors: GlobalColors,
        video_chars: Vec<usize>,
        video_colors: Vec<u8>,
        characters: HashMap<usize, [u8; Char::HEIGHT]>,
    ) -> Result<Self, Error> {
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
            .take(size.area() as usize)
            .collect();
        assert_eq!(size.area() as usize, raw_video.len());
        let video = ImgVec::new(raw_video, size.width as usize, size.height as usize);
        let mut bitmaps = BiMap::new();
        bitmaps.extend(characters);
        Ok(Self {
            colors: global_colors,
            video,
            bitmaps,
        })
    }

    pub fn with_content(video: ImgVec<Char>) -> Self {
        Self {
            colors: Default::default(),
            video,
            bitmaps: BiMap::new(),
        }
    }

    pub fn from_image(source_image: &RgbaImage) -> Result<VicImage, Error> {
        let columns = (source_image.width() as usize + Char::WIDTH - 1) / Char::WIDTH;
        let rows = (source_image.height() as usize + Char::HEIGHT - 1) / Char::HEIGHT;
        let mut image = VicImage::new(columns, rows);
        image.paste_image(source_image, Point { x: 0, y: 0 }, ColorFormat::Multicolor);
        Ok(image)
    }

    pub fn size_in_cells(&self) -> SizeInCells {
        SizeInCells {
            width: self.video.width() as u32,
            height: self.video.height() as u32,
        }
    }

    /// Get the width and height of the image in pixels.
    pub fn pixel_size(&self) -> (usize, usize) {
        let size = self.size_in_cells();
        (
            size.width as usize * Char::WIDTH,
            size.height as usize * Char::HEIGHT,
        )
    }

    /// Get one of the global colors.
    pub fn global_color(&self, index: u32) -> u8 {
        self.colors[index]
    }

    /// Set one of the global colors.
    /// Return true if the value actually changed.
    pub fn set_global_color(&mut self, index: u32, color: u8) -> bool {
        if self.colors[index] != color {
            self.colors[index] = color;
            true
        } else {
            false
        }
    }

    /// Paste a true color image into this image.
    pub fn paste_image(&mut self, source: &RgbaImage, target: Point, format: ColorFormat) {
        const CELL_W: i32 = Char::WIDTH as i32;
        const CELL_H: i32 = Char::HEIGHT as i32;
        let start_column = (target.x / CELL_W as i32).max(0);
        let end_column = ((target.x + source.width() as i32 + CELL_W - 1) / CELL_W)
            .min(self.size_in_cells().width as i32);
        let start_row = (target.y / CELL_H as i32).max(0);
        let end_row = ((target.y + source.height() as i32 + CELL_H - 1) / CELL_H)
            .min(self.size_in_cells().height as i32);

        let global_colors = &self.colors;

        for (r, c) in (start_row..end_row).cartesian_product(start_column..end_column) {
            let left = (c * CELL_W) - target.x;
            let top = (r * CELL_H) - target.y;
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

    pub fn color_index_from_paint_color(&self, c: &PixelColor) -> u8 {
        match c {
            PixelColor::Background => self.colors[GlobalColors::BACKGROUND],
            PixelColor::Border => self.colors[GlobalColors::BORDER],
            PixelColor::Aux => self.colors[GlobalColors::AUX],
            PixelColor::CharColor(index) => *index,
        }
    }

    pub fn true_color_from_paint_color(&self, c: &PixelColor) -> TrueColor {
        VicPalette::color(self.color_index_from_paint_color(c))
    }

    /// Paste characters into the image.
    /// `target_pos` is the top-left corner.
    /// The extents of the pasted chars may be outside the image (they are clipped).
    pub fn paste_chars(
        &mut self,
        target_pos: &CellPos,
        source: ImgRef<'_, Char>,
    ) -> Result<bool, Box<dyn DisallowedAction>> {
        let mut changed = false;
        let source_size = SizeInCells {
            width: source.width() as u32,
            height: source.height() as u32,
        };
        for (char, (r, c)) in source.pixels().zip(
            (target_pos.row..target_pos.row + source_size.height as i32)
                .cartesian_product(target_pos.column..target_pos.column + source_size.width as i32),
        ) {
            let p = CellPos { row: r, column: c };
            if let Some(p) = p.within_bounds(&self.size_in_cells()) {
                self.video[p.as_tuple()] = char;
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
        F: Fn(PixelColor) -> PixelColor,
    {
        let mut changed = false;
        for (cell, mask) in self.cells_and_pixels(target) {
            let char = &mut self.video[cell.as_tuple()];
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
        F: Fn(PixelColor) -> PixelColor,
    {
        let mut changed = false;
        let mask = BitVec::from_elem(Char::WIDTH * Char::HEIGHT, true);
        for cell in self.target_cells(target) {
            let char = &mut self.video[cell.as_tuple()];
            changed |= char.mutate_pixels(&mask, &operation)?;
        }
        Ok(changed)
    }

    /// Standard draw tool. Draw single pixels.
    pub fn plot(
        &mut self,
        target: &UpdateArea,
        color: PixelColor,
    ) -> Result<bool, Box<dyn DisallowedAction>> {
        self.apply_operation_to_pixels(target, |_| color)
    }

    /// Fill the whole cell with a given color
    pub fn fill_cells(
        &mut self,
        target: &UpdateArea,
        color: PixelColor,
    ) -> Result<bool, Box<dyn DisallowedAction>> {
        self.apply_operation_to_cells(target, |_| color)
    }

    /// Replace one color with another.
    pub fn replace_color(
        &mut self,
        target: &UpdateArea,
        to_replace: PixelColor,
        replacement: PixelColor,
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
        color_1: PixelColor,
        color_2: PixelColor,
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
        if !super::ALLOWED_CHAR_COLORS.contains(&color) {
            return Err(Box::new(DisallowedEdit::DisallowedCharacterColor));
        }
        let mut changed = false;
        for cell in self.target_cells(target) {
            let cell = &mut self.video[cell.as_tuple()];
            changed = cell.set_color(color);
        }
        Ok(changed)
    }

    pub fn make_high_res(
        &mut self,
        target: &UpdateArea,
    ) -> Result<bool, Box<dyn DisallowedAction>> {
        let mut changed = false;
        for cell in self.target_cells(target) {
            changed |= self.video[cell.as_tuple()].make_high_res()?;
        }
        Ok(changed)
    }

    pub fn make_multicolor(
        &mut self,
        target: &UpdateArea,
    ) -> Result<bool, Box<dyn DisallowedAction>> {
        let mut changed = false;
        for cell in self.target_cells(target) {
            changed |= self.video[cell.as_tuple()].make_multicolor()?;
        }
        Ok(changed)
    }

    /// Get the pixel coordinates of the top-left corner of a character cell.
    /// Accepts coordinates outside the image.
    pub fn cell_coordinates_unclipped(&self, cell: &CellPos) -> Point {
        Point {
            x: cell.column * Char::WIDTH as i32,
            y: cell.row * Char::HEIGHT as i32,
        }
    }

    /// Get the pixel coordinates of the top-left corner of a character cell.
    /// Returns None if the given cell coordinates are outside the image.
    pub fn cell_coordinates(&self, cell: &CellPos) -> Option<Point> {
        let cell = cell.within_bounds(&self.size_in_cells())?;
        Some(self.cell_coordinates_unclipped(&cell))
    }

    /// Get a rectangle in pixel coordinates from a rectangle in character cells.
    /// Returns the top left, and bottom right (exclusive) of the rectangle in image pixels.
    /// Accepts coordinates outside the image.
    pub fn cell_rectangle(&self, rect: &CellRect) -> (Point, Point) {
        (
            self.cell_coordinates_unclipped(&rect.top_left),
            self.cell_coordinates_unclipped(&(rect.top_left + rect.size)),
        )
    }

    /// Get at which pixel coordinates to dispay grid lines
    pub fn vertical_grid_lines(&self) -> impl Iterator<Item = i32> {
        (0..=self.size_in_cells().width).map(|c| (c * Char::WIDTH as u32) as i32)
    }

    /// Get at which pixel coordinates to dispay grid lines
    pub fn horizontal_grid_lines(&self) -> impl Iterator<Item = i32> {
        (0..=self.size_in_cells().height).map(|r| (r * Char::HEIGHT as u32) as i32)
    }

    /// General information about the image
    pub fn image_info(&self) -> String {
        format!("{} characters used", self.bitmaps.len())
    }

    /// Information about the given pixel in the image
    pub fn pixel_info(&self, position: Point) -> String {
        if let Some((cell, _cx, _cy)) = self.char_coordinates(position.x, position.y) {
            let char = &self.video[cell.as_tuple()];
            format!(
                "({}, {}): column {}, row {} {} color {}",
                position.x,
                position.y,
                cell.column,
                cell.row,
                if char.is_multicolor() {
                    "multicolor"
                } else {
                    "high-res"
                },
                char.color()
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
        VicPalette::color(i)
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
    pub fn grab_cells(&self, rect: &WithinBounds<CellRect>) -> ImgVec<Char> {
        let chars = self
            .video
            .sub_image(
                rect.left() as usize,
                rect.top() as usize,
                rect.width() as usize,
                rect.height() as usize,
            )
            .pixels()
            .collect();
        ImgVec::new(chars, rect.width() as usize, rect.height() as usize)
    }

    /// Given pixel coordinates, return column, row, and x and y inside the character.
    /// May return coordinates outside the image.
    pub fn char_coordinates_unclipped(&self, x: i32, y: i32) -> (CellPos, i32, i32) {
        let column = x.div_euclid(Char::WIDTH as i32);
        let cx = x.rem_euclid(Char::WIDTH as i32);
        let row = y.div_euclid(Char::HEIGHT as i32);
        let cy = y.rem_euclid(Char::HEIGHT as i32);
        (CellPos { column, row }, cx, cy)
    }

    /// Given pixel coordinates, return column, row, and x and y inside the character.
    /// Returns None if the coordinates are outside the image.
    pub fn char_coordinates(&self, x: i32, y: i32) -> Option<(WithinBounds<CellPos>, i32, i32)> {
        let (width, height) = self.pixel_size();
        if (0..width as i32).contains(&x) && (0..height as i32).contains(&y) {
            let (cell, cx, cy) = self.char_coordinates_unclipped(x, y);
            Some((cell.within_bounds(&self.size_in_cells()).unwrap(), cx, cy))
        } else {
            None
        }
    }

    /// Given pixel coordinates, return column, row, and x and y inside the character.
    /// If the arguments are outside the image, they are clamped to be inside it.
    pub fn char_coordinates_clamped(&self, x: i32, y: i32) -> (WithinBounds<CellPos>, i32, i32) {
        let (width, height) = self.pixel_size();
        let (cell, cx, cy) = self.char_coordinates_unclipped(
            x.clamp(0, width as i32 - 1),
            y.clamp(0, height as i32 - 1),
        );
        (cell.within_bounds(&self.size_in_cells()).unwrap(), cx, cy)
    }

    /// Get the character cells to update given an UpdateArea.
    /// Returns the columns and rows of the cells within this image's bounds.
    fn target_cells(&self, target: &UpdateArea) -> Vec<WithinBounds<CellPos>> {
        self.cells_and_pixels(target)
            .iter()
            .map(|(cell, _)| cell)
            .copied()
            .collect()
    }

    /// Get the character cells to update, and which pixel in each cell to update.
    fn cells_and_pixels(&self, target: &UpdateArea) -> HashMap<WithinBounds<CellPos>, BitVec> {
        target.cells_and_pixels(
            Char::WIDTH as u32,
            Char::HEIGHT as u32,
            &self.size_in_cells(),
        )
    }
}

/// Generates an optimized highres image using the given hardware palette colors.
/// Tries different colors and finds the one that gives the least quantization error.
/// Returns the resulting color numbers.
pub fn optimized_image_highres(original: &RgbaImage, global_colors: &GlobalColors) -> ImgVec<u8> {
    let fixed_colors = [global_colors[GlobalColors::BACKGROUND]];
    image_operations::optimized_image(
        original,
        &fixed_colors,
        super::ALLOWED_CHAR_COLORS,
        VicPalette::all_colors(),
    )
}

/// Generates an optimized multicolor image using the given hardware palette colors.
/// Tries different colors and finds the one that gives the least quantization error.
/// Returns the resulting color numbers.
pub fn optimized_image_multicolor(
    original: &RgbaImage,
    global_colors: &GlobalColors,
) -> ImgVec<u8> {
    let fixed_colors = [
        global_colors[GlobalColors::BACKGROUND],
        global_colors[GlobalColors::BORDER],
        global_colors[GlobalColors::AUX],
    ];
    image_operations::optimized_image(
        original,
        &fixed_colors,
        super::ALLOWED_CHAR_COLORS,
        VicPalette::all_colors(),
    )
}
