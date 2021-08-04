use std::error::Error;
use crate::metadata::domain::InnerMetadata;


pub trait SecretRepository {
    fn find(&self, id: &str) -> Result<Secret, Box<dyn Error>>;
    fn save(&self, secret: &mut Secret) -> Result<(), Box<dyn Error>>;
    fn delete(&self, secret: &Secret) -> Result<(), Box<dyn Error>>;
}

pub struct Secret<'a> {
    pub id: String,
    pub data: &'a [u8], // pkey as a pem file
    // the updated_at field from metadata works as a touch_at field, being updated for each
    // usage of the secret
    pub meta: InnerMetadata,

    pub(super) repo: &'a dyn SecretRepository, 
}

impl<'a> Secret<'a> {
    pub fn new(secret_repo: &'a dyn SecretRepository,
               data: &'a [u8]) -> Result<Self, Box<dyn Error>> {

        let mut secret = Secret {
            id: "".to_string(),
            data: data,
            meta: InnerMetadata::new(),

            repo: secret_repo,
        };

        secret_repo.save(&mut secret)?;
        Ok(secret)
    }

    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn save(&mut self) -> Result<(), Box<dyn Error>> {
        self.repo.save(self)
    }

    pub fn delete(&self) -> Result<(), Box<dyn Error>> {
        self.repo.delete(self)
    }
}