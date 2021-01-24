use serde_json;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Problem occurred while importing or exporting.")]
    IOError { path: String, source: io::Error },

    #[error("IO error")]
    SerializationError(serde_json::Error),

    #[error("Zettel doesn't exist error")]
    ZettelDoesntExistsError,

    #[error("Zettel already exists")]
    ZettelExistsError,
}
