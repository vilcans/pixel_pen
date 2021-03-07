//! Loading (and saving) image files.

mod fluff;

use bincode::Options;
use serde::de::DeserializeOwned;
use std::{
    fs::File,
    io::{self, BufReader, Read},
    path::Path,
};

use crate::{error::Error, vic::VicImage};

pub fn load_file(filename: &Path) -> Result<VicImage, Error> {
    let file = File::open(filename)?;
    let mut reader = BufReader::new(file);
    fluff::load_fluff64(&mut reader)
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
