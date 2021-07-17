use std::error::Error;

use crate::regex;
use crate::metadata::domain::Metadata;


pub trait SecretRepository {
    fn find(&self, id: &str) -> Result<Secret, Box<dyn Error>>;
    fn save(&self, secret: &mut Secret) -> Result<(), Box<dyn Error>>;
    fn delete(&self, secret: &Secret) -> Result<(), Box<dyn Error>>;
}

pub struct Secret {
    pub id: String,
    pub data: Vec<u8>, // pkey as a pem file
    // the updated_at field from metadata works as a touch_at field, being updated for each
    // usage of the secret
    pub meta: Metadata,
}

impl Secret {
    pub fn new(secret_repo: Box<dyn SecretRepository>,
               data: &[u8]) -> Result<Self, Box<dyn Error>> {

        let mut secret = Secret {
            id: "".to_string(),
            data: data.to_vec(),
            meta: Metadata::now(),
        };

        secret_repo.save(&mut secret)?;
        Ok(secret)
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }
}