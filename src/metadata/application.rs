use std::error::Error;
use super::domain::Metadata;

pub trait MetadataRepository {
    fn find(&self, id: i32) -> Result<Metadata, Box<dyn Error>>;
    fn create(&self, meta: &mut Metadata) -> Result<(), Box<dyn Error>>;
    fn save(&self, meta: &Metadata) -> Result<(), Box<dyn Error>>;
    fn delete(&self, meta: &Metadata) -> Result<(), Box<dyn Error>>;
}

#[cfg(test)]
pub mod tests {
    use std::error::Error;
    use super::MetadataRepository;
    use super::super::domain::{
        tests::new_metadata,
        Metadata
    };
  
    pub struct MetadataRepositoryMock {
        pub fn_find: Option<fn (this: &MetadataRepositoryMock, id: i32) -> Result<Metadata, Box<dyn Error>>>,
        pub fn_create: Option<fn (this: &MetadataRepositoryMock, meta: &mut Metadata) -> Result<(), Box<dyn Error>>>,
        pub fn_save: Option<fn (this: &MetadataRepositoryMock, meta: &Metadata) -> Result<(), Box<dyn Error>>>,
        pub fn_delete: Option<fn (this: &MetadataRepositoryMock, meta: &Metadata) -> Result<(), Box<dyn Error>>>,
    }

    impl MetadataRepositoryMock {
        pub fn new() -> Self {
            MetadataRepositoryMock {
                fn_find: None,
                fn_create: None,
                fn_save: None,
                fn_delete: None,
            }
        }
    }

    impl MetadataRepository for MetadataRepositoryMock {
        fn find(&self, id: i32) -> Result<Metadata, Box<dyn Error>> {
            if let Some(f) = self.fn_find {
                return f(self, id);
            }

            Ok(new_metadata())
        }

        fn create(&self, meta: &mut Metadata) -> Result<(), Box<dyn Error>> {
            if let Some(f) = self.fn_create {
                return f(self, meta);
            }

            Ok(())
        }

        fn save(&self, meta: &Metadata) -> Result<(), Box<dyn Error>> {
            if let Some(f) = self.fn_save {
                return f(self, meta);
            }

            Ok(())
        }

        fn delete(&self, meta: &Metadata) -> Result<(), Box<dyn Error>> {
            if let Some(f) = self.fn_delete {
                return f(self, meta);
            }

            Ok(())
        }
    }
}