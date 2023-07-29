use crate::metadata::domain::Metadata;
use crate::user::domain::User;
use chrono::naive::NaiveDateTime;

#[derive(Debug, Clone, Copy, strum_macros::EnumString, strum_macros::Display)]
#[strum(serialize_all = "lowercase")]
pub enum SecretKind {
    Totp,
}

#[derive(Debug, Clone)]
pub struct Secret {
    pub(super) id: i32,
    pub(super) owner: i32,
    pub(super) kind: SecretKind,
    pub(super) data: Vec<u8>,
    pub(super) meta: Metadata,
}

impl Secret {
    pub fn new(kind: SecretKind, data: &[u8], user: &User) -> Self {
        Secret {
            id: 0,
            owner: user.get_id(),
            kind,
            data: data.to_vec(),
            meta: Metadata::default(),
        }
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn is_deleted(&self) -> bool {
        self.meta.deleted_at.is_some()
    }

    pub fn set_deleted_at(&mut self, deleted_at: Option<NaiveDateTime>) {
        self.meta.deleted_at = deleted_at;
    }
}

#[cfg(test)]
pub mod tests {
    use super::{Secret, SecretKind};
    use crate::metadata::domain::Metadata;
    use crate::user::domain::tests::new_user;

    pub const TEST_DEFAULT_SECRET_DATA: &str = "this is a secret";

    pub fn new_secret() -> Secret {
        let inner_meta = Metadata::default();

        Secret {
            id: 999_i32,
            owner: 0_i32,
            kind: SecretKind::Totp,
            data: TEST_DEFAULT_SECRET_DATA.as_bytes().to_vec(),
            meta: inner_meta,
        }
    }

    #[test]
    fn secret_new_should_not_fail() {
        let data = "secret_new_should_success".as_bytes();
        let user = new_user();
        let secret = Secret::new(SecretKind::Totp, data, &user);

        assert_eq!(0, secret.id);
        assert!(matches!(secret.kind, SecretKind::Totp));
        assert_eq!(data, secret.data);
    }
}
