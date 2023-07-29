use super::domain::{Secret, SecretKind};
use crate::result::Result;
use async_trait::async_trait;

#[async_trait]
pub trait SecretRepository {
    async fn find(&self, id: i32) -> Result<Secret>;
    async fn find_by_user_and_kind(&self, user: i32, kind: SecretKind) -> Result<Secret>;
    async fn create(&self, secret: &mut Secret) -> Result<()>;
    async fn save(&self, secret: &Secret) -> Result<()>;
    async fn delete(&self, secret: &Secret) -> Result<()>;
}

#[cfg(test)]
pub mod tests {
    use super::super::domain::{tests::new_secret, Secret};
    use super::SecretRepository;
    use crate::result::Result;
    use crate::secret::domain::SecretKind;
    use async_trait::async_trait;

    type MockFnFind = Option<fn(this: &SecretRepositoryMock, id: i32) -> Result<Secret>>;
    type MockFnFindByUserAndKind =
        Option<fn(this: &SecretRepositoryMock, user: i32, kind: SecretKind) -> Result<Secret>>;
    type MockFnCreate = Option<fn(this: &SecretRepositoryMock, secret: &mut Secret) -> Result<()>>;
    type MockFnSave = Option<fn(this: &SecretRepositoryMock, secret: &Secret) -> Result<()>>;
    type MockFnDelete = Option<fn(this: &SecretRepositoryMock, secret: &Secret) -> Result<()>>;

    #[derive(Default)]
    pub struct SecretRepositoryMock {
        pub fn_find: MockFnFind,
        pub fn_find_by_user_and_kind: MockFnFindByUserAndKind,
        pub fn_create: MockFnCreate,
        pub fn_save: MockFnSave,
        pub fn_delete: MockFnDelete,
    }

    #[async_trait]
    impl SecretRepository for SecretRepositoryMock {
        #[instrument(skip(self))]
        async fn find(&self, id: i32) -> Result<Secret> {
            if let Some(f) = self.fn_find {
                return f(self, id);
            }

            Ok(new_secret())
        }

        async fn find_by_user_and_kind(&self, user: i32, kind: SecretKind) -> Result<Secret> {
            if let Some(f) = self.fn_find_by_user_and_kind {
                return f(self, user, kind);
            }

            Ok(new_secret())
        }

        async fn create(&self, secret: &mut Secret) -> Result<()> {
            if let Some(f) = self.fn_create {
                return f(self, secret);
            }

            Ok(())
        }

        async fn save(&self, secret: &Secret) -> Result<()> {
            if let Some(f) = self.fn_save {
                return f(self, secret);
            }

            Ok(())
        }

        async fn delete(&self, secret: &Secret) -> Result<()> {
            if let Some(f) = self.fn_delete {
                return f(self, secret);
            }

            Ok(())
        }
    }
}
