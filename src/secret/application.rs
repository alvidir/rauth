use super::domain::{Secret, SecretKind};
use super::error::Result;
use crate::user::domain::User;
use async_trait::async_trait;

#[async_trait]
pub trait SecretRepository {
    async fn find(&self, id: i32) -> Result<Secret>;
    async fn find_by_owner_and_kind(&self, owner: i32, kind: SecretKind) -> Result<Secret>;
    async fn create(&self, secret: &mut Secret) -> Result<()>;
    async fn delete(&self, secret: &Secret) -> Result<()>;
    async fn delete_by_owner(&self, owner: &User) -> Result<()>;
}

#[cfg(test)]
pub mod test {
    use super::super::domain::Secret;
    use super::super::error::{Error, Result};
    use super::SecretRepository;
    use crate::secret::domain::SecretKind;
    use crate::user::domain::User;
    use async_trait::async_trait;

    type MockFnFind = fn(this: &SecretRepositoryMock, id: i32) -> Result<Secret>;
    type MockFnFindByOwnerAndKind =
        fn(this: &SecretRepositoryMock, owner: i32, kind: SecretKind) -> Result<Secret>;
    type MockFnCreate = fn(this: &SecretRepositoryMock, secret: &mut Secret) -> Result<()>;
    type MockFnSave = fn(this: &SecretRepositoryMock, secret: &Secret) -> Result<()>;
    type MockFnDelete = fn(this: &SecretRepositoryMock, secret: &Secret) -> Result<()>;
    type MockFnDeleteByOwner = fn(this: &SecretRepositoryMock, owner: &User) -> Result<()>;

    #[derive(Default)]
    pub struct SecretRepositoryMock {
        pub fn_find: Option<MockFnFind>,
        pub fn_find_by_owner_and_kind: Option<MockFnFindByOwnerAndKind>,
        pub fn_create: Option<MockFnCreate>,
        pub fn_save: Option<MockFnSave>,
        pub fn_delete: Option<MockFnDelete>,
        pub fn_delete_by_owner: Option<MockFnDeleteByOwner>,
    }

    #[async_trait]
    impl SecretRepository for SecretRepositoryMock {
        #[instrument(skip(self))]
        async fn find(&self, id: i32) -> Result<Secret> {
            if let Some(f) = self.fn_find {
                return f(self, id);
            }

            Err(Error::Debug)
        }

        async fn find_by_owner_and_kind(&self, owner: i32, kind: SecretKind) -> Result<Secret> {
            if let Some(f) = self.fn_find_by_owner_and_kind {
                return f(self, owner, kind);
            }

            Err(Error::Debug)
        }

        async fn create(&self, secret: &mut Secret) -> Result<()> {
            if let Some(f) = self.fn_create {
                return f(self, secret);
            }

            Err(Error::Debug)
        }

        async fn delete(&self, secret: &Secret) -> Result<()> {
            if let Some(f) = self.fn_delete {
                return f(self, secret);
            }

            Err(Error::Debug)
        }

        async fn delete_by_owner(&self, owner: &User) -> Result<()> {
            if let Some(f) = self.fn_delete_by_owner {
                return f(self, owner);
            }

            Err(Error::Debug)
        }
    }
}
