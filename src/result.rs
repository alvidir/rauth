//! Custom result and common errors thrown by the application

/// Result represents a custom result where error is of the [`Error`] type.
pub type Result<T> = std::result::Result<T, Error>;

/// StdResult is an alias for [`std::result::Result`] where error impl the [`std::error::Error`] trait.
pub type StdResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Error represent an error thrown by the application
#[derive(strum_macros::Display, Debug, PartialEq)]
pub enum Error {
    #[strum(serialize = "E001")]
    Unknown,
    #[strum(serialize = "E002")]
    NotFound,
    #[strum(serialize = "E003")]
    NotAvailable,
    #[strum(serialize = "E004")]
    Unauthorized,
    #[strum(serialize = "E005")]
    InvalidToken,
    #[strum(serialize = "E006")]
    InvalidFormat,
    #[strum(serialize = "E007")]
    InvalidHeader,
    #[strum(serialize = "E008")]
    WrongCredentials,
    #[strum(serialize = "E009")]
    RegexNotMatch,
}

impl From<Error> for String {
    fn from(val: Error) -> Self {
        val.to_string()
    }
}
