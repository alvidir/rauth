use std::error::Error;
use crate::metadata::domain::InnerMetadata;


pub trait SecretRepository {
    fn find(&self, id: &str) -> Result<Secret, Box<dyn Error>>;
    fn create(&self, secret: &mut Secret) -> Result<(), Box<dyn Error>>;
    fn save(&self, secret: &Secret) -> Result<(), Box<dyn Error>>;
    fn delete(&self, secret: &Secret) -> Result<(), Box<dyn Error>>;
}

pub struct Secret {
    pub(super) id: String,
    pub(super) data: Vec<u8>, // pkey as a pem file
    pub(super) meta: InnerMetadata,
}

impl Secret {
    pub fn new(data: &[u8]) -> Result<Self, Box<dyn Error>> {

        let mut secret = Secret {
            id: "".to_string(),
            data: data.to_vec(),
            meta: InnerMetadata::new(),
        };

        super::get_repository().create(&mut secret)?;
        Ok(secret)
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn get_id(&self) -> &str {
        &self.id
    }

    pub fn _save(&self) -> Result<(), Box<dyn Error>> {
        super::get_repository().save(self)
    }

    pub fn delete(&self) -> Result<(), Box<dyn Error>> {
        super::get_repository().delete(self)
    }
}