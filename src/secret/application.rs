use std::error::Error;
use super::domain::Secret;

pub trait SecretRepository {
    fn find(&self, id: i32) -> Result<Secret, Box<dyn Error>>;
    fn create(&self, secret: &mut Secret) -> Result<(), Box<dyn Error>>;
    fn save(&self, secret: &Secret) -> Result<(), Box<dyn Error>>;
    fn delete(&self, secret: &Secret) -> Result<(), Box<dyn Error>>;
}