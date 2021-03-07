use bincode::Options;
use serde::{de::DeserializeOwned, Deserialize};
use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
};

use crate::{
    error::Error,
    vic::{GlobalColors, VicImage},
};

#[derive(Deserialize)]
#[repr(packed(1))]
struct FluffHeader {
    /// "FLUFF64"
    pub identifier: [u8; 7],
    pub version: u32,
    pub image_type: u8,
    pub palette_type: u8,
    pub background: u8,
    pub border: u8,
    pub pen1: u8, // border?
    pub pen2: u8, // aux?
    pub pen3: u8, // draw color?
    pub width_chars: u8,
    pub height_chars: u8,
}

pub fn load_fluff64(reader: &mut impl Read) -> Result<VicImage, Error> {
    let header: FluffHeader = read_struct(reader)?;
    let mut image = VicImage::new(header.width_chars as usize, header.height_chars as usize);
    image.colors = GlobalColors([header.background, header.border, header.pen2]);
    Ok(image)
}

pub fn load_file(filename: &Path) -> Result<VicImage, Error> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    load_fluff64(&mut reader)
}

fn read_struct<T>(reader: &mut impl Read) -> Result<T, Error>
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
