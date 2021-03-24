//! Loading (and saving) image files.

mod fluff;

use bincode::Options;
use image::{self, GenericImageView};
use serde::de::DeserializeOwned;
use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
};

use crate::{error::Error, vic::VicImage};

const FLUFF64_EXTENSION: &str = "flf";

pub enum FileFormat {
    /// Turbo Rascal's format
    Fluff,
    /// Any image format supported by the `image` crate
    StandardImage,
}

pub fn identify_file(filename: &Path) -> Option<FileFormat> {
    match filename
        .extension()
        .map(|e| e.to_string_lossy().to_lowercase())
    {
        Some(ext) if FLUFF64_EXTENSION == ext => Some(FileFormat::Fluff),
        Some(ext) if "png" == ext => Some(FileFormat::StandardImage),
        _ => None,
    }
}

pub fn load_file(filename: &Path, format: FileFormat) -> Result<VicImage, Error> {
    match format {
        FileFormat::Fluff => {
            let file = File::open(filename)?;
            let mut reader = BufReader::new(file);
            fluff::load_fluff64(&mut reader)
        }
        FileFormat::StandardImage => load_standard_image(filename),
    }
}

/// Load an image in any format supported by `image` crate.
pub fn load_standard_image(filename: &Path) -> Result<VicImage, Error> {
    let img = image::open(filename)?;
    println!(
        "dimensions {:?}, colors {:?}",
        img.dimensions(),
        img.color()
    );
    VicImage::from_image(img)
}

pub fn read_struct<T>(reader: &mut impl Read) -> Result<T, Error>
where
    T: DeserializeOwned,
{
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .reject_trailing_bytes()
        .deserialize_from(reader)
        .map_err(|e| match *e {
            bincode::ErrorKind::Io(e @ io::Error { .. }) => {
                if e.kind() == io::ErrorKind::UnexpectedEof {
                    Error::TruncatedData
                } else {
                    Error::ReadFailure(e)
                }
            }
            _ => Error::Deserialization(e),
        })
}
