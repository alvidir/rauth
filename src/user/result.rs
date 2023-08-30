//! Result type and errors related to user stuff.

pub type Result<T> = std::result::Result<T, Error>;

impl<T> From<Error> for Result<T> {
    fn from(value: Error) -> Self {
        Self::Err(value)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    // #[error("data store disconnected")]
    // Disconnect(),
    // #[error("the data for key `{0}` is not available")]
    // Redaction(String),
    // #[error("invalid header (expected {expected:?}, found {found:?})")]
    // InvalidHeader { expected: String, found: String },
    // #[error("unknown data store error")]
    // Unknown,
    #[error("email regex did unmatch")]
    NotAnEmail,
    #[error("password regex did unmatch")]
    NotAPassword,
    #[error("user does not exists")]
    NotFound,
    #[error("unexpected error")]
    Unknown,
}
