//! Vic-20 palette.

use crate::colors::TrueColor;

/// Functions for getting the true colors to display for different color indices.
pub struct VicPalette;

impl VicPalette {
    /// Get the color to display for a given palette index.
    pub fn color<T>(index: T) -> TrueColor
    where
        T: Into<usize>,
    {
        COLORS[index.into()]
    }

    /// Get the name of a color from the palette.
    /// `index` must be in the range `0..PALETTE_SIZE`.
    pub fn name<T>(index: T) -> &'static str
    where
        T: Into<usize>,
    {
        NAMES[index.into()]
    }

    pub fn all_colors() -> &'static [TrueColor] {
        &COLORS
    }
}

const PALETTE_SIZE: usize = 16;

const COLORS: [TrueColor; PALETTE_SIZE] = [
    //                      0xRRGGBB
    TrueColor::from_u32(0x000000), // Black
    TrueColor::from_u32(0xffffff), // White
    TrueColor::from_u32(0x6d2327), // Red
    TrueColor::from_u32(0xa0fef8), // Cyan
    TrueColor::from_u32(0x8e3c97), // Purple
    TrueColor::from_u32(0x7eda75), // Green
    TrueColor::from_u32(0x252390), // Blue
    TrueColor::from_u32(0xffff86), // Yellow
    TrueColor::from_u32(0xa4643b), // Orange
    TrueColor::from_u32(0xffc8a1), // Light Orange
    TrueColor::from_u32(0xf2a7ab), // Pink
    TrueColor::from_u32(0xdbffff), // Light Cyan
    TrueColor::from_u32(0xffb4ff), // Light Purple
    TrueColor::from_u32(0xd7ffce), // Light Green
    TrueColor::from_u32(0x9d9aff), // Light Blue
    TrueColor::from_u32(0xffffc9), // Light Yellow
];

const NAMES: [&str; PALETTE_SIZE] = [
    "Black",
    "White",
    "Red",
    "Cyan",
    "Purple",
    "Green",
    "Blue",
    "Yellow",
    "Orange",
    "Light Orange",
    "Pink",
    "Light Cyan",
    "Light Purple",
    "Light Green",
    "Light Blue",
    "Light Yellow",
];
