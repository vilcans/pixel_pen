use std::{fmt, io};
use thiserror::Error;

/// How serious an error is.
pub enum Severity {
    /// There is no need to report this to the user.
    Silent,
    /// Not serious, but the user should be notified about it.
    Notification,
}

pub trait DisallowedAction: fmt::Debug + fmt::Display {
    fn severity(&self) -> Severity {
        Severity::Notification
    }
}

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
    #[error("Dialog failed: {0}")]
    DialogError(String),
    #[error("No file name given")]
    NoFileName,
}
