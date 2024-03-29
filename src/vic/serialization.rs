use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{cell_image::CellImageSize, coords::SizeInCells, error::Error};

use super::{Char, GlobalColors, VicImage};

/// Image for serialization to or deserialization from file.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct VicImageFile {
    columns: usize,
    rows: usize,
    colors: GlobalColorsFile,

    /// The character at each position.
    /// Size: columns x rows.
    video_chars: Vec<usize>,

    /// The color and multicolor bit at each position.
    /// Size: columns x rows.
    video_colors: Vec<u8>,

    /// Bitmap for each character as hex string
    characters: Vec<Option<String>>,
}

/// Supports deserializing GlobalColors from an array of three integers, used in old files.
#[derive(Serialize, Deserialize, Clone)]
#[serde(untagged)]
enum GlobalColorsFile {
    List(u8, u8, u8),
    Struct(GlobalColors),
}

impl From<GlobalColorsFile> for GlobalColors {
    fn from(file: GlobalColorsFile) -> Self {
        match file {
            GlobalColorsFile::List(background, border, aux) => GlobalColors {
                background,
                border,
                aux,
            },
            GlobalColorsFile::Struct(g) => g,
        }
    }
}
impl From<GlobalColors> for GlobalColorsFile {
    fn from(g: GlobalColors) -> Self {
        Self::Struct(g)
    }
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
        let max_char = character_map
            .left_values()
            .max()
            .map(|m| m + 1)
            .unwrap_or(0);
        let characters = (0..max_char)
            .map(|i| character_map.get_by_left(&i).map(hex::encode))
            .collect();
        let instance = Self {
            columns: image.size_in_cells().width as usize,
            rows: image.size_in_cells().height as usize,
            colors: image.colors.clone().into(),
            video_chars,
            video_colors,
            characters,
        };
        assert!(instance.verify().is_ok());
        instance
    }

    pub fn into_image(self) -> Result<VicImage, Error> {
        let characters = self
            .characters
            .iter()
            .enumerate()
            .filter_map(|(num, bits_string)| bits_string.clone().map(|b| (num, b)))
            .map(|(num, bits_string)| {
                let mut bits = [0u8; Char::HEIGHT];
                hex::decode_to_slice(bits_string, &mut bits)?;
                Ok((num, bits))
            })
            .collect::<Result<HashMap<usize, [u8; Char::HEIGHT]>, Error>>()?;
        VicImage::from_data(
            SizeInCells::new(self.columns as i32, self.rows as i32),
            self.colors.into(),
            self.video_chars,
            self.video_colors,
            characters,
        )
    }

    pub fn verify(&self) -> Result<(), Error> {
        if self.columns == 0
            || self.rows == 0
            || self.columns >= VicImage::MAX_SIZE.width as usize
            || self.rows >= VicImage::MAX_SIZE.height as usize
        {
            Err(Error::InvalidSize(self.columns, self.rows))
        } else if self.characters.is_empty() {
            Err(Error::NoCharacters)
        } else {
            Ok(())
        }
    }
}

impl Serialize for VicImage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let doc = VicImageFile::from_image(self);
        doc.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for VicImage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let file_image = VicImageFile::deserialize(deserializer)?;
        file_image
            .verify()
            .and_then(|_| file_image.into_image())
            .map_err(serde::de::Error::custom)
    }
}
