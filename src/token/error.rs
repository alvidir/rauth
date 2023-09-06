//! Result type and errors related to token stuff.

pub type Result<T> = std::result::Result<T, Error>;

impl<T> From<Error> for Result<T> {
    fn from(value: Error) -> Self {
        Self::Err(value)
    }
}

impl From<String> for Error {
    fn from(value: String) -> Self {
        Self::Unknown(value)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("invalid token: {0}")]
    Invalid(#[from] jsonwebtoken::errors::Error),
    #[error("token does not exists")]
    NotFound,
    #[error("unexpected error: {0}")]
    Unknown(String),
}
