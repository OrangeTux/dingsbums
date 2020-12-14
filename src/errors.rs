use serde_json;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Problems with writing to a file.")]
    WriteError(io::Error),

    #[error("IO error")]
    SerializationError(serde_json::Error),

    #[error("Zettel doesn't exist error")]
    ZettelDoesntExistsError,

    #[error("Zettel already exists")]
    ZettelExistsError,
}
