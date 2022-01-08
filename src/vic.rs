use crate::{colors::TrueColor, error::DisallowedAction};
use serde::{Deserialize, Serialize};
use std::ops::{Index, IndexMut, RangeInclusive};
use thiserror::Error;

mod char;
mod image;
mod serialization;

pub use self::{char::Char, image::VicImage};

const PALETTE_SIZE: usize = 16;

// From /usr/lib/vice/VIC20/colodore_vic.vpl
const PALETTE: [(u32, &str); PALETTE_SIZE] = [
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

pub fn true_color_palette() -> [TrueColor; PALETTE_SIZE] {
    array_init::array_init(|i| palette_color(i))
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

#[allow(clippy::enum_variant_names)] // All variants have the same prefix (Disallowed)
#[derive(Error, Debug)]
pub enum DisallowedEdit {
    #[error("High resolution characters can be painted with color 0-7, or background")]
    DisallowedHiresColor,
    #[error("Character color must be between 0 and 7")]
    DisallowedCharacterColor,
}

impl DisallowedAction for DisallowedEdit {}
