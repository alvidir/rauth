use super::domain::Metadata;
use crate::result::Result;
use async_trait::async_trait;

#[async_trait]
pub trait MetadataRepository {
    async fn find(&self, id: i32) -> Result<Metadata>;
    async fn create(&self, meta: &mut Metadata) -> Result<()>;
    async fn save(&self, meta: &Metadata) -> Result<()>;
    async fn delete(&self, meta: &Metadata) -> Result<()>;
}

#[cfg(test)]
pub mod tests {
    use super::super::domain::{tests::new_metadata, Metadata};
    use super::MetadataRepository;
    use crate::result::Result;
    use async_trait::async_trait;

    type MockFnFind = Option<fn(this: &MetadataRepositoryMock, id: i32) -> Result<Metadata>>;

    type MockFnCreate =
        Option<fn(this: &MetadataRepositoryMock, meta: &mut Metadata) -> Result<()>>;

    type MockFnSave = Option<fn(this: &MetadataRepositoryMock, meta: &Metadata) -> Result<()>>;

    type MockFnDelete = Option<fn(this: &MetadataRepositoryMock, meta: &Metadata) -> Result<()>>;

    #[derive(Default)]
    pub struct MetadataRepositoryMock {
        pub fn_find: MockFnFind,
        pub fn_create: MockFnCreate,
        pub fn_save: MockFnSave,
        pub fn_delete: MockFnDelete,
    }

    #[async_trait]
    impl MetadataRepository for MetadataRepositoryMock {
        async fn find(&self, id: i32) -> Result<Metadata> {
            if let Some(f) = self.fn_find {
                return f(self, id);
            }

            Ok(new_metadata())
        }

        async fn create(&self, meta: &mut Metadata) -> Result<()> {
            if let Some(f) = self.fn_create {
                return f(self, meta);
            }

            Ok(())
        }

        async fn save(&self, meta: &Metadata) -> Result<()> {
            if let Some(f) = self.fn_save {
                return f(self, meta);
            }

            Ok(())
        }

        async fn delete(&self, meta: &Metadata) -> Result<()> {
            if let Some(f) = self.fn_delete {
                return f(self, meta);
            }

            Ok(())
        }
    }
}
