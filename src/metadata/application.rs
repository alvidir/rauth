use super::domain::Metadata;
use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait MetadataRepository {
    async fn find(&self, id: i32) -> Result<Metadata, Box<dyn Error>>;
    async fn create(&self, meta: &mut Metadata) -> Result<(), Box<dyn Error>>;
    async fn save(&self, meta: &Metadata) -> Result<(), Box<dyn Error>>;
    async fn delete(&self, meta: &Metadata) -> Result<(), Box<dyn Error>>;
}

#[cfg(test)]
pub mod tests {
    use super::super::domain::{tests::new_metadata, Metadata};
    use super::MetadataRepository;
    use async_trait::async_trait;
    use std::error::Error;
    pub struct MetadataRepositoryMock {
        pub fn_find:
            Option<fn(this: &MetadataRepositoryMock, id: i32) -> Result<Metadata, Box<dyn Error>>>,
        pub fn_create: Option<
            fn(this: &MetadataRepositoryMock, meta: &mut Metadata) -> Result<(), Box<dyn Error>>,
        >,
        pub fn_save: Option<
            fn(this: &MetadataRepositoryMock, meta: &Metadata) -> Result<(), Box<dyn Error>>,
        >,
        pub fn_delete: Option<
            fn(this: &MetadataRepositoryMock, meta: &Metadata) -> Result<(), Box<dyn Error>>,
        >,
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

    #[async_trait]
    impl MetadataRepository for MetadataRepositoryMock {
        async fn find(&self, id: i32) -> Result<Metadata, Box<dyn Error>> {
            if let Some(f) = self.fn_find {
                return f(self, id);
            }

            Ok(new_metadata())
        }

        async fn create(&self, meta: &mut Metadata) -> Result<(), Box<dyn Error>> {
            if let Some(f) = self.fn_create {
                return f(self, meta);
            }

            Ok(())
        }

        async fn save(&self, meta: &Metadata) -> Result<(), Box<dyn Error>> {
            if let Some(f) = self.fn_save {
                return f(self, meta);
            }

            Ok(())
        }

        async fn delete(&self, meta: &Metadata) -> Result<(), Box<dyn Error>> {
            if let Some(f) = self.fn_delete {
                return f(self, meta);
            }

            Ok(())
        }
    }
}
