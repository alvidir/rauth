use std::error::Error;
use crate::metadata::domain::Metadata;

pub trait SecretRepository {
    fn find(&self, id: i32) -> Result<Secret, Box<dyn Error>>;
    fn create(&self, secret: &mut Secret) -> Result<(), Box<dyn Error>>;
    fn save(&self, secret: &Secret) -> Result<(), Box<dyn Error>>;
    fn delete(&self, secret: &Secret) -> Result<(), Box<dyn Error>>;
}

#[derive(Clone)]
pub struct Secret {
    pub(super) id: i32,
    pub(super) data: Vec<u8>, // pkey as a pem file
    pub(super) meta: Metadata,
}

impl Secret {
    pub fn new(data: &[u8]) -> Self {
        Secret {
            id: 0,
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
}


#[cfg(test)]
pub mod tests {
    use crate::metadata::domain::Metadata;
    use super::Secret;

    pub fn new_secret() -> Secret {
        let inner_meta = Metadata::new();

        Secret {
            id: 999,
            data: "this is a secret".as_bytes().to_vec(),
            meta: inner_meta,
        }
    }

    #[test]
    fn secret_new_should_not_fail() {
        let secret = Secret::new("secret_new_should_success".as_bytes());

        assert_eq!(0, secret.id); 
        assert_eq!("secret_new_should_success".as_bytes(), secret.data);
    }
}