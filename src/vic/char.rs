use super::ALLOWED_CHAR_COLORS;
use crate::{colors::TrueColor, error::DisallowedAction, ui::ViewSettings};
use bit_vec::BitVec;
use imgref::ImgRef;

use super::{DisallowedEdit, GlobalColors, PixelColor, VicPalette};

#[derive(Clone, Copy, Hash)]
pub struct Char {
    pub(super) bits: [u8; 8],
    pub(super) color: u8,
    pub(super) multicolor: bool,
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

    /// Gets the character color
    pub fn color(&self) -> u8 {
        self.color
    }

    /// Sets the character color. Returns true if it was changed, false if it was the same as before.
    pub fn set_color(&mut self, color: u8) -> bool {
        if self.color != color {
            self.color = color;
            true
        } else {
            false
        }
    }

    pub fn is_multicolor(&self) -> bool {
        self.multicolor
    }

    /// Return the 4 bit value as stored in color RAM.
    pub fn raw_nibble(&self) -> u8 {
        self.color + if self.multicolor { 8 } else { 0 }
    }

    pub fn render(
        &self,
        colors: &GlobalColors,
        settings: &ViewSettings,
    ) -> [TrueColor; Self::WIDTH * Self::HEIGHT] {
        if self.multicolor {
            let (background, border, aux, char_color) = match settings {
                ViewSettings::Normal => (
                    VicPalette::color(colors[GlobalColors::BACKGROUND]),
                    VicPalette::color(colors[GlobalColors::BORDER]),
                    VicPalette::color(colors[GlobalColors::AUX]),
                    VicPalette::color(self.color),
                ),
                ViewSettings::Raw => ViewSettings::raw_colors(),
            };
            Self::render_multicolor(&self.bits, background, border, aux, char_color)
        } else {
            let (background, char_color) = match settings {
                ViewSettings::Normal => (
                    VicPalette::color(colors[GlobalColors::BACKGROUND]),
                    VicPalette::color(self.color),
                ),
                ViewSettings::Raw => (
                    ViewSettings::raw_highres_background(),
                    ViewSettings::raw_hires_char_color(),
                ),
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
        F: Fn(PixelColor) -> PixelColor,
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
        F: Fn(PixelColor) -> PixelColor,
    {
        let mut changed = false;
        let mut new_color = self.color;

        for cy in 0..Self::HEIGHT {
            let mut new_bits = self.bits[cy];
            for cx in (0..Self::WIDTH).step_by(2) {
                let shift = 6 - cx;
                if mask[cx + cy * Self::WIDTH] || mask[cx + cy * Self::WIDTH + 1] {
                    let current = match (self.bits[cy] >> shift) & 0b11 {
                        0b00 => PixelColor::Background,
                        0b01 => PixelColor::Border,
                        0b10 => PixelColor::CharColor(self.color),
                        0b11 => PixelColor::Aux,
                        _ => unreachable!(),
                    };
                    let to_set = match operation(current) {
                        PixelColor::Background => 0b00,
                        PixelColor::Border => 0b01,
                        PixelColor::CharColor(c) => {
                            new_color = c;
                            0b10
                        }
                        PixelColor::Aux => 0b11,
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
        F: Fn(PixelColor) -> PixelColor,
    {
        let mut changed = false;
        let mut new_color = self.color;

        for cy in 0..Self::HEIGHT {
            let mut new_bits = self.bits[cy];
            for cx in 0..Self::WIDTH {
                let bytemask = 0x80 >> cx;
                if let Some(true) = mask.get(cx + cy * Self::WIDTH) {
                    let current = match self.bits[cy] & bytemask {
                        0 => PixelColor::Background,
                        _ => PixelColor::CharColor(self.color),
                    };
                    match operation(current) {
                        PixelColor::Background => new_bits &= !bytemask,
                        PixelColor::CharColor(c) => {
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
        changed |= self.set_color(new_color);
        Ok(changed)
    }

    pub fn make_high_res(&mut self) -> Result<bool, Box<dyn DisallowedAction>> {
        if !self.multicolor {
            return Ok(false);
        }
        self.multicolor = false;
        Ok(true)
    }

    pub fn make_multicolor(&mut self) -> Result<bool, Box<dyn DisallowedAction>> {
        if self.multicolor {
            return Ok(false);
        }
        self.multicolor = true;
        Ok(true)
    }

    pub fn mirror_x(&mut self) {
        if self.multicolor {
            for b in &mut self.bits {
                *b = (0..7)
                    .step_by(2)
                    .map(|bit| ((*b >> bit) & 0b11) << (6 - bit))
                    .sum()
            }
        } else {
            for b in &mut self.bits {
                *b = b.reverse_bits();
            }
        }
    }

    pub fn mirror_y(&mut self) {
        self.bits.reverse();
    }
}

impl Default for Char {
    fn default() -> Self {
        Self::new([0u8; 8], 1)
    }
}
