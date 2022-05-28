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
    pub struct SecretRepositoryMock {
        pub fn_find:
            Option<fn(this: &SecretRepositoryMock, id: i32) -> Result<Secret, Box<dyn Error>>>,
        pub fn_find_by_user_and_name: Option<
            fn(
                this: &SecretRepositoryMock,
                user: i32,
                name: &str,
            ) -> Result<Secret, Box<dyn Error>>,
        >,
        pub fn_create: Option<
            fn(this: &SecretRepositoryMock, secret: &mut Secret) -> Result<(), Box<dyn Error>>,
        >,
        pub fn_save:
            Option<fn(this: &SecretRepositoryMock, secret: &Secret) -> Result<(), Box<dyn Error>>>,
        pub fn_delete:
            Option<fn(this: &SecretRepositoryMock, secret: &Secret) -> Result<(), Box<dyn Error>>>,
    }

    impl SecretRepositoryMock {
        pub fn new() -> Self {
            SecretRepositoryMock {
                fn_find: None,
                fn_find_by_user_and_name: None,
                fn_create: None,
                fn_save: None,
                fn_delete: None,
            }
        }
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
