use crate::user::domain::User;

#[derive(Debug, Clone, Copy, strum_macros::EnumString, strum_macros::AsRefStr)]
#[strum(serialize_all = "lowercase")]
pub enum SecretKind {
    Totp,
}

/// Represent some sensitive data that cannot be updated.
#[derive(Debug, Clone)]
pub struct Secret {
    pub id: i32,
    pub owner: i32,
    pub kind: SecretKind,
    data: Vec<u8>, // read-only data
}

impl Secret {
    /// Builds a new [Secret] with the id set to the default value.
    pub fn new(kind: SecretKind, owner: &User, data: &[u8]) -> Self {
        Secret {
            id: Default::default(),
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
