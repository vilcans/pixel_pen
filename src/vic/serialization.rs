use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use super::{GlobalColors, VicImage};

/// Image for serialization to or deserialization from file.
#[derive(Serialize, Deserialize)]
struct VicImageFile {
    columns: usize,
    rows: usize,
    colors: GlobalColors,

    /// The character at each position.
    /// Size: columns x rows.
    video_chars: Vec<usize>,

    /// The color and multicolor bit at each position.
    /// Size: columns x rows.
    video_colors: Vec<u8>,

    /// Bitmap for each character
    characters: Vec<Option<[u8; 8]>>,
}

impl VicImageFile {
    pub fn from_image(image: &VicImage) -> Self {
        let character_map = image.map_characters();

        let video_chars = image
            .video
            .pixels()
            .map(|char| *character_map.get_by_right(&char.bits).unwrap())
            .collect();
        let video_colors = image.video.pixels().map(|char| char.raw_nibble()).collect();
        //let characters = character_map.iter().map(|(k, v)| (*k as u32, *v)).collect();
        let max_char = character_map
            .left_values()
            .max()
            .map(|m| m + 1)
            .unwrap_or(0);
        let characters = (0..max_char)
            .map(|i| character_map.get_by_left(&i).cloned())
            .collect();
        Self {
            columns: image.columns,
            rows: image.rows,
            colors: image.colors.clone(),
            video_chars,
            video_colors,
            characters,
        }
    }
    pub fn to_image(self) -> VicImage {
        VicImage::default()
    }
}

impl Serialize for VicImage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let doc = VicImageFile::from_image(&self);
        doc.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for VicImage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ok(VicImageFile::deserialize(deserializer)?.to_image())
    }
}
