//! The Vic-20 platform.

use crate::{colors::TrueColor, error::DisallowedAction};
use serde::{Deserialize, Serialize};
use std::ops::RangeInclusive;
use thiserror::Error;

mod char;
mod image;
mod palette;
mod registers;
mod serialization;

pub use self::{char::Char, image::VicImage, palette::VicPalette, registers::GlobalColors};

const RAW_HIRES_BACKGROUND: TrueColor = TrueColor::from_u32(0x555555);
const RAW_HIRES_CHAR_COLOR: TrueColor = TrueColor::from_u32(0xeeeeee);

const RAW_MULTICOLOR_BACKGROUND: TrueColor = TrueColor::from_u32(0x000000);
const RAW_MULTICOLOR_BORDER: TrueColor = TrueColor::from_u32(0x0044ff);
const RAW_MULTICOLOR_AUX: TrueColor = TrueColor::from_u32(0xff0000);
const RAW_MULTICOLOR_CHAR_COLOR: TrueColor = TrueColor::from_u32(0xffffff);

/// Which colors are allowed as the "character" color.
pub const ALLOWED_CHAR_COLORS: RangeInclusive<u8> = 0..=7;

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

#[allow(clippy::enum_variant_names)] // All variants have the same prefix (Disallowed)
#[derive(Error, Debug)]
pub enum DisallowedEdit {
    #[error("High resolution characters can be painted with color 0-7, or background")]
    DisallowedHiresColor,
    #[error("Character color must be between 0 and 7")]
    DisallowedCharacterColor,
}

impl DisallowedAction for DisallowedEdit {}
