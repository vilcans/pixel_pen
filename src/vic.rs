//! The Vic-20 platform.

use crate::error::DisallowedAction;
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;
use thiserror::Error;

mod char;
mod image;
mod palette;
mod registers;
mod serialization;

pub use self::{
    char::Char, image::VicImage, palette::VicPalette, registers::GlobalColors, registers::Register,
};

/// Which colors are allowed as the "character" color.
pub const ALLOWED_CHAR_COLORS: RangeInclusive<u8> = 0..=7;

/// A choice of color for an individual pixel.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum PixelColor {
    Background,
    Border,
    Aux,
    CharColor(u8),
}

impl Default for PixelColor {
    fn default() -> Self {
        Self::CharColor(2)
    }
}

impl PixelColor {
    /// Which colors from the palette are possible to choose for this color.
    pub fn selectable_colors(&self) -> impl Iterator<Item = u8> {
        match self {
            PixelColor::Background => 0..=15,
            PixelColor::Border => 0..=7,
            PixelColor::Aux => 0..=15,
            PixelColor::CharColor(index) => *index..=*index,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub enum ColorFormat {
    HighRes,
    Multicolor,
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
