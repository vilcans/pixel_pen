use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read from file")]
    ReadFailure(#[from] io::Error),
    #[error("truncated data")]
    TruncatedData,
    #[error("incorrect file identifier - wrong file type?")]
    WrongMagic,
    #[error("invalid image size: {0} columns x {1} rows")]
    InvalidSize(usize, usize),
    #[error("deserializing struct")]
    Deserialization(Box<bincode::ErrorKind>),
    #[error("failed to load JSON data: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("No characters defined")]
    NoCharacters,
    #[error("Invalid hexadecimal value: {0}")]
    HexError(#[from] hex::FromHexError),
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Invalid image")]
    ImageError(#[from] image::ImageError),
    #[error("Unknown file format on file \"{0}\"")]
    UnknownFileFormat(std::path::PathBuf),
    #[error("File dialog failed: {0}")]
    FileDialogError(String),
    #[error("No file name given")]
    NoFileName,
}
