use super::domain::Secret;
use crate::result::Result;
use async_trait::async_trait;

#[async_trait]
pub trait SecretRepository {
    async fn find(&self, id: i32) -> Result<Secret>;
    async fn find_by_user_and_name(&self, user: i32, name: &str) -> Result<Secret>;
    async fn create(&self, secret: &mut Secret) -> Result<()>;
    async fn save(&self, secret: &Secret) -> Result<()>;
    async fn delete(&self, secret: &Secret) -> Result<()>;
}

#[cfg(test)]
pub mod tests {
    use super::super::domain::{tests::new_secret, Secret};
    use super::SecretRepository;
    use crate::result::Result;
    use async_trait::async_trait;

    type MockFnFind = Option<fn(this: &SecretRepositoryMock, id: i32) -> Result<Secret>>;
    type MockFnFindByUserAndName =
        Option<fn(this: &SecretRepositoryMock, user: i32, name: &str) -> Result<Secret>>;
    type MockFnCreate = Option<fn(this: &SecretRepositoryMock, secret: &mut Secret) -> Result<()>>;
    type MockFnSave = Option<fn(this: &SecretRepositoryMock, secret: &Secret) -> Result<()>>;
    type MockFnDelete = Option<fn(this: &SecretRepositoryMock, secret: &Secret) -> Result<()>>;

    #[derive(Default)]
    pub struct SecretRepositoryMock {
        pub fn_find: MockFnFind,
        pub fn_find_by_user_and_name: MockFnFindByUserAndName,
        pub fn_create: MockFnCreate,
        pub fn_save: MockFnSave,
        pub fn_delete: MockFnDelete,
    }

    #[async_trait]
    impl SecretRepository for SecretRepositoryMock {
        async fn find(&self, id: i32) -> Result<Secret> {
            if let Some(f) = self.fn_find {
                return f(self, id);
            }

            Ok(new_secret())
        }

        async fn find_by_user_and_name(&self, user: i32, name: &str) -> Result<Secret> {
            if let Some(f) = self.fn_find_by_user_and_name {
                return f(self, user, name);
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
