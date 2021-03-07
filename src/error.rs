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
}
