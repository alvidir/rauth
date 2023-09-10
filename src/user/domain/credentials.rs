use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
};

use super::{Email, PasswordHash};
use serde::{Deserialize, Serialize};

/// Represents the credentials of a [User].
#[derive(Debug, Default, Hash, Serialize, Deserialize)]
pub struct Credentials {
    pub email: Email,
    pub password: PasswordHash,
}

impl Credentials {
    pub fn new(email: Email, password: PasswordHash) -> Self {
        Credentials { email, password }
    }

    /// Returns the result of hashing self.
    pub fn hash(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        Hash::hash(self, &mut hasher);
        hasher.finish()
    }
}
