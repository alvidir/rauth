use crate::metadata::domain::Metadata;
use crate::user::domain::User;
use chrono::naive::NaiveDateTime;

#[derive(Debug, Clone)]
pub struct Secret {
    pub(super) id: i32,
    pub(super) owner: i32,
    pub(super) name: String,
    pub(super) data: Vec<u8>,
    pub(super) meta: Metadata,
}

impl Secret {
    pub fn new(user: &User, name: &str, data: &[u8]) -> Self {
        Secret {
            id: 0,
            owner: user.get_id(),
            name: name.to_string(),
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
    use super::Secret;
    use crate::metadata::domain::Metadata;
    use crate::user::domain::tests::new_user;

    pub const TEST_DEFAULT_SECRET_NAME: &str = "dummysecret";
    pub const TEST_DEFAULT_SECRET_DATA: &str = "this is a secret";

    pub fn new_secret() -> Secret {
        let inner_meta = Metadata::default();

        Secret {
            id: 999_i32,
            owner: 0_i32,
            name: TEST_DEFAULT_SECRET_NAME.to_string(),
            data: TEST_DEFAULT_SECRET_DATA.as_bytes().to_vec(),
            meta: inner_meta,
        }
    }

    #[test]
    fn secret_new_should_not_fail() {
        let name = "dummy secret";
        let data = "secret_new_should_success".as_bytes();
        let user = new_user();
        let secret = Secret::new(&user, name, data);

        assert_eq!(0, secret.id);
        assert_eq!(name, secret.name);
        assert_eq!(data, secret.data);
    }
}
