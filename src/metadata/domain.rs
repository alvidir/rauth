use std::error::Error;
use std::time::{SystemTime};

pub trait MetadataRepository {
    fn find(&self, id: i32) -> Result<Metadata, Box<dyn Error>>;
    fn save(&self, meta: &mut Metadata) -> Result<(), Box<dyn Error>>;
    fn delete(&self, meta: &Metadata) -> Result<(), Box<dyn Error>>;
}

pub struct Metadata {
    pub id: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl Metadata {
    pub fn new(repo: Box<dyn MetadataRepository>) -> Result<Self, Box<dyn Error>> {
        let mut meta = Metadata {
            id: 0,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        };

        repo.save(&mut meta)?;
        Ok(meta)
    }

    pub fn now() -> Self {
        Metadata {
            id: 0,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }
}