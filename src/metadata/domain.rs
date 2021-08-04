use std::error::Error;
use std::time::{SystemTime};

pub trait MetadataRepository {
    fn find(&self, id: i32) -> Result<Metadata, Box<dyn Error>>;
    fn save(&self, meta: &mut Metadata) -> Result<(), Box<dyn Error>>;
    fn delete(&self, meta: &Metadata) -> Result<(), Box<dyn Error>>;
}

#[derive(Clone)]
pub struct Metadata {
    pub id: i32,
    pub created_at: SystemTime,
    pub updated_at: SystemTime,

    //repo: &'static dyn MetadataRepository,
}

impl Metadata {
    pub fn new(repo: &/*'static*/ dyn MetadataRepository) -> Result<Self, Box<dyn Error>> {
        let mut meta = Metadata {
            id: 0,
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),

            //repo: repo,
        };

        repo.save(&mut meta)?;
        Ok(meta)
    }

    // pub fn save(&mut self) -> Result<(), Box<dyn Error>> {
    //     self.repo.save(self)
    // }

    // pub fn delete(&self) -> Result<(), Box<dyn Error>> {
    //     self.repo.delete(self)
    // }
}

pub struct InnerMetadata {
    pub created_at: SystemTime,
    pub updated_at: SystemTime,
}

impl InnerMetadata {
    pub fn new() -> Self {
        InnerMetadata {
            created_at: SystemTime::now(),
            updated_at: SystemTime::now(),
        }
    }
}