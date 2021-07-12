use std::error::Error;

use crate::regex;
use crate::metadata::domain::Metadata;


pub trait SecretRepository {
    fn find(id: &str) -> Result<Secret, Box<dyn Error>>;
    fn save(secret: &mut Secret) -> Result<(), Box<dyn Error>>;
    fn delete(secret: &Secret) -> Result<(), Box<dyn Error>>;
}

pub struct Secret {
    pub id: String,
    pub data: String, // secret file in base64
    // the updated_at field from metadata works as a touch_at field, being updated for each
    // usage of the secret
    pub meta: Metadata,
}

impl Secret {
    pub fn new(data: &str) -> Result<Self, Box<dyn Error>> {
        regex::match_regex(regex::BASE64, data)?;

        let app = Secret {
            id: "".to_string(),
            data: data.to_string(),
            meta: Metadata::new(),
        };

        Ok(app)
    }
}