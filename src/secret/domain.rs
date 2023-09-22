use crate::user::domain::{User, UserID};

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, strum_macros::EnumString, strum_macros::AsRefStr,
)]
#[strum(serialize_all = "lowercase")]
pub enum SecretKind {
    Otp,
    Salt,
}

/// Represent some sensitive data that cannot be updated.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Secret {
    pub owner: UserID,
    pub kind: SecretKind,
    pub(crate) data: Vec<u8>, // read-only data
}

impl Secret {
    pub fn new(kind: SecretKind, owner: &User, data: &[u8]) -> Self {
        Secret {
            owner: owner.id,
            kind,
            data: data.to_vec(),
        }
    }

    /// Returns an immutable reference to the secret's data.
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}
