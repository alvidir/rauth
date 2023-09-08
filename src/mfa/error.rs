pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("OTP regex did not match")]
    NotAOneTimePassword,
    #[error("one time password required")]
    Required,
    #[error("wrong one time password")]
    Invalid,
    #[error("{0}")]
    Secret(#[from] crate::secret::error::Error),
    #[error("{0}")]
    Oath(#[from] libreauth::oath::Error),
}
