mod email;
pub use email::*;

mod password;
pub use password::*;

mod credentials;
pub use credentials::*;

/// Represents a signed up user
#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub credentials: Credentials,
}

impl From<Credentials> for User {
    fn from(credentials: Credentials) -> Self {
        Self { id: 0, credentials }
    }
}
