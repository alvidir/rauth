//! Result type and errors related to user stuff.

pub type Result<T> = std::result::Result<T, Error>;

impl<T> From<Error> for Result<T> {
    fn from(value: Error) -> Self {
        Self::Err(value)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("email regex did unmatch")]
    NotAnEmail,
    #[error("password regex did unmatch")]
    NotAPassword,
    #[error("user not found")]
    NotFound,
    #[error("{0}")]
    Base64(#[from] base64::DecodeError),
    #[cfg(feature = "postgres")]
    #[error("{0}")]
    Sql(#[from] sqlx::error::Error),
    #[error("{0}")]
    Tera(#[from] tera::Error),
    #[error("unexpected error: {0}")]
    Unknown(String),
}

impl From<String> for Error {
    fn from(error: String) -> Self {
        Self::Unknown(error)
    }
}

impl Error {
    pub fn is_not_found(&self) -> bool {
        matches!(self, Error::NotFound)
    }
}
