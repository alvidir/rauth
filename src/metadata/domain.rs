use std::error::Error;
use std::time::{SystemTime};

pub trait MetadataRepository {
    fn find(id: i32) -> Result<Metadata, Box<dyn Error>>;
    fn save(meta: &mut Metadata) -> Result<(), Box<dyn Error>>;
    fn delete(meta: &Metadata) -> Result<(), Box<dyn Error>>;
}

pub struct Metadata {
    pub id: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl Metadata {
    pub fn new<'a>(id: i32, created_at: SystemTime, updated_at: SystemTime) -> Self {
        Metadata {
            id: 0,
            created_at: created_at,
            updated_at: updated_at
        }
    }
}