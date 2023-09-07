//! Result type and errors related to user stuff.

use std::num::ParseIntError;

pub type Result<T> = std::result::Result<T, Error>;

impl<T> From<Error> for Result<T> {
    fn from(value: Error) -> Self {
        Self::Err(value)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("email regex did not match")]
    NotAnEmail,
    #[error("password regex did not match")]
    NotAPassword,
    #[error("wrong user credentials")]
    WrongCredentials,
    #[error("user not found")]
    NotFound,
    #[error("user already exists")]
    AlreadyExists,
    #[error("{0}")]
    ParseInt(#[from] ParseIntError),
    #[error("{0}")]
    Base64(#[from] base64::DecodeError),
    #[cfg(feature = "postgres")]
    #[error("{0}")]
    Sql(#[from] sqlx::error::Error),
    #[cfg(feature = "smtp")]
    #[error("{0}")]
    Tera(#[from] tera::Error),
    #[cfg(feature = "rabbitmq")]
    #[error("{0}")]
    Deadpool(#[from] deadpool_lapin::PoolError),
    #[cfg(feature = "rabbitmq")]
    #[error("{0}")]
    Lapin(#[from] lapin::Error),
    #[error("{0}")]
    Cache(#[from] crate::cache::Error),
    #[error("{0}")]
    Token(#[from] crate::token::error::Error),
    #[error("unexpected error")]
    Unknown(String),
    #[cfg(test)]
    #[error("unexpected error")]
    Debug,
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
