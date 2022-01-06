use std::time::{SystemTime};
use crate::metadata::domain::Metadata;

#[derive(Clone)]
pub struct Secret {
    pub(super) id: i32,
    pub(super) name: String,
    pub(super) data: Vec<u8>,
    pub(super) meta: Metadata,
}

impl Secret {
    pub fn new(name: &str, data: &[u8]) -> Self {
        Secret {
            id: 0,
            name: name.to_string(),
            data: data.to_vec(),
            meta: Metadata::new(),
        }
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn get_id(&self) -> i32 {
        self.id
    }

    pub fn is_deleted(&self) -> bool {
        self.meta.deleted_at.is_none()
    }

    pub fn set_deleted_at(&mut self, deleted_at: Option<SystemTime>) {
        self.meta.deleted_at = deleted_at;
    }
}


#[cfg(test)]
pub mod tests {
    use crate::metadata::domain::Metadata;
    use super::Secret;

    pub fn new_secret() -> Secret {
        let inner_meta = Metadata::new();

        Secret {
            id: 999,
            name: "dummy secret".to_string(),
            data: "this is a secret".as_bytes().to_vec(),
            meta: inner_meta,
        }
    }

    #[test]
    fn secret_new_should_not_fail() {
        let name = "dummy secret";
        let data = "secret_new_should_success".as_bytes();
        let secret = Secret::new(name.clone(), data.clone());

        assert_eq!(0, secret.id); 
        assert_eq!(name, secret.name); 
        assert_eq!(data, secret.data);
    }
}