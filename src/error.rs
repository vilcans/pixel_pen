use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("failed to read from file")]
    ReadFailure(#[from] io::Error),
    #[error("truncated data")]
    TruncatedData,
    #[error("invalid header")]
    InvalidHeader,
    #[error("deserializing struct")]
    Deserialization(Box<bincode::ErrorKind>),
}
