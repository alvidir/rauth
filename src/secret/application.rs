use super::domain::Secret;
use async_trait::async_trait;
use std::error::Error;

#[async_trait]
pub trait SecretRepository {
    async fn find(&self, id: i32) -> Result<Secret, Box<dyn Error>>;
    async fn find_by_user_and_name(&self, user: i32, name: &str) -> Result<Secret, Box<dyn Error>>;
    async fn create(&self, secret: &mut Secret) -> Result<(), Box<dyn Error>>;
    async fn save(&self, secret: &Secret) -> Result<(), Box<dyn Error>>;
    async fn delete(&self, secret: &Secret) -> Result<(), Box<dyn Error>>;
}

#[cfg(test)]
pub mod tests {
    use super::super::domain::{tests::new_secret, Secret};
    use super::SecretRepository;
    use async_trait::async_trait;
    use std::error::Error;

    type MockFnFind =
        Option<fn(this: &SecretRepositoryMock, id: i32) -> Result<Secret, Box<dyn Error>>>;
    type MockFnFindByUserAndName = Option<
        fn(this: &SecretRepositoryMock, user: i32, name: &str) -> Result<Secret, Box<dyn Error>>,
    >;
    type MockFnCreate =
        Option<fn(this: &SecretRepositoryMock, secret: &mut Secret) -> Result<(), Box<dyn Error>>>;
    type MockFnSave =
        Option<fn(this: &SecretRepositoryMock, secret: &Secret) -> Result<(), Box<dyn Error>>>;
    type MockFnDelete =
        Option<fn(this: &SecretRepositoryMock, secret: &Secret) -> Result<(), Box<dyn Error>>>;

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
        async fn find(&self, id: i32) -> Result<Secret, Box<dyn Error>> {
            if let Some(f) = self.fn_find {
                return f(self, id);
            }

            Ok(new_secret())
        }

        async fn find_by_user_and_name(
            &self,
            user: i32,
            name: &str,
        ) -> Result<Secret, Box<dyn Error>> {
            if let Some(f) = self.fn_find_by_user_and_name {
                return f(self, user, name);
            }

            Ok(new_secret())
        }

        async fn create(&self, secret: &mut Secret) -> Result<(), Box<dyn Error>> {
            if let Some(f) = self.fn_create {
                return f(self, secret);
            }

            Ok(())
        }

        async fn save(&self, secret: &Secret) -> Result<(), Box<dyn Error>> {
            if let Some(f) = self.fn_save {
                return f(self, secret);
            }

            Ok(())
        }

        async fn delete(&self, secret: &Secret) -> Result<(), Box<dyn Error>> {
            if let Some(f) = self.fn_delete {
                return f(self, secret);
            }

            Ok(())
        }
    }
}
