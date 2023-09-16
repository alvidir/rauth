pub mod delete;
pub mod mfa;
pub mod reset;
pub mod signup;

use super::domain::{Email, User};
use super::error::Result;
use crate::token::domain::Token;
use async_trait::async_trait;
use std::sync::Arc;

#[async_trait]
pub trait UserRepository {
    async fn find(&self, id: i32) -> Result<User>;
    async fn find_by_email(&self, email: &Email) -> Result<User>;
    async fn find_by_name(&self, name: &str) -> Result<User>;
    async fn create(&self, user: &mut User) -> Result<()>;
    async fn save(&self, user: &User) -> Result<()>;
    async fn delete(&self, user: &User) -> Result<()>;
}

#[async_trait]
pub trait EventService {
    async fn emit_user_created(&self, user: &User) -> Result<()>;
    async fn emit_user_deleted(&self, user: &User) -> Result<()>;
}

pub trait MailService {
    fn send_credentials_verification_email(&self, to: &Email, token: &Token) -> Result<()>;
    fn send_credentials_reset_email(&self, to: &Email, token: &Token) -> Result<()>;
}

pub struct UserApplication<U, S, T, F, M, B, C> {
    pub hash_length: usize,
    pub user_repo: Arc<U>,
    pub secret_repo: Arc<S>,
    pub token_srv: Arc<T>,
    pub multi_factor_srv: Arc<F>,
    pub mail_srv: Arc<M>,
    pub event_srv: Arc<B>,
    pub cache: Arc<C>,
}

#[cfg(test)]
mod test {
    use super::{EventService, MailService, UserRepository};
    use crate::{
        token::domain::Token,
        user::{
            domain::{Email, User},
            error::{Error, Result},
        },
    };
    use async_trait::async_trait;

    type FindFn = fn(&UserRepositoryMock, id: i32) -> Result<User>;
    type FindByEmailFn = fn(&UserRepositoryMock, email: &Email) -> Result<User>;
    type FindByNameFn = fn(&UserRepositoryMock, name: &str) -> Result<User>;
    type CreateFn = fn(&UserRepositoryMock, user: &mut User) -> Result<()>;
    type SaveFn = fn(&UserRepositoryMock, user: &User) -> Result<()>;
    type DeleteFn = fn(&UserRepositoryMock, user: &User) -> Result<()>;

    #[derive(Debug, Default)]
    struct UserRepositoryMock {
        pub find_fn: Option<FindFn>,
        pub find_by_email_fn: Option<FindByEmailFn>,
        pub find_by_name_fn: Option<FindByNameFn>,
        pub create_fn: Option<CreateFn>,
        pub save_fn: Option<SaveFn>,
        pub delete_fn: Option<DeleteFn>,
    }

    #[async_trait]
    impl UserRepository for UserRepositoryMock {
        async fn find(&self, id: i32) -> Result<User> {
            if let Some(find_fn) = self.find_fn {
                return find_fn(self, id);
            }

            Err(Error::Debug)
        }

        async fn find_by_email(&self, email: &Email) -> Result<User> {
            if let Some(find_by_email_fn) = self.find_by_email_fn {
                return find_by_email_fn(self, email);
            }

            Err(Error::Debug)
        }

        async fn find_by_name(&self, name: &str) -> Result<User> {
            if let Some(find_by_name_fn) = self.find_by_name_fn {
                return find_by_name_fn(self, name);
            }

            Err(Error::Debug)
        }

        async fn create(&self, user: &mut User) -> Result<()> {
            if let Some(create_fn) = self.create_fn {
                return create_fn(self, user);
            }

            Err(Error::Debug)
        }

        async fn save(&self, user: &User) -> Result<()> {
            if let Some(save_fn) = self.save_fn {
                return save_fn(self, user);
            }

            Err(Error::Debug)
        }

        async fn delete(&self, user: &User) -> Result<()> {
            if let Some(delete_fn) = self.delete_fn {
                return delete_fn(self, user);
            }

            Err(Error::Debug)
        }
    }

    type EmitUserCreatedFn = fn(&EventServiceMock, user: &User) -> Result<()>;
    type EmitUserDeletedFn = fn(&EventServiceMock, user: &User) -> Result<()>;

    #[derive(Debug, Default)]
    struct EventServiceMock {
        pub emit_user_created_fn: Option<EmitUserCreatedFn>,
        pub emit_user_deleted_fn: Option<EmitUserDeletedFn>,
    }

    #[async_trait]
    impl EventService for EventServiceMock {
        async fn emit_user_created(&self, user: &User) -> Result<()> {
            if let Some(emit_user_created_fn) = self.emit_user_created_fn {
                return emit_user_created_fn(self, user);
            }

            Err(Error::Debug)
        }

        async fn emit_user_deleted(&self, user: &User) -> Result<()> {
            if let Some(emit_user_deleted_fn) = self.emit_user_deleted_fn {
                return emit_user_deleted_fn(self, user);
            }

            Err(Error::Debug)
        }
    }

    type SendCredentialsVerificationEmailFn =
        fn(&MailServiceMock, to: &Email, token: &Token) -> Result<()>;
    type SendCredentialsResetEmailFn =
        fn(&MailServiceMock, to: &Email, token: &Token) -> Result<()>;

    struct MailServiceMock {
        pub send_credentials_verification_email_fn: Option<SendCredentialsVerificationEmailFn>,
        pub send_credentials_reset_email_fn: Option<SendCredentialsResetEmailFn>,
    }

    impl MailService for MailServiceMock {
        fn send_credentials_verification_email(&self, to: &Email, token: &Token) -> Result<()> {
            if let Some(send_credentials_verification_email_fn) =
                self.send_credentials_verification_email_fn
            {
                return send_credentials_verification_email_fn(self, to, token);
            }

            Err(Error::Debug)
        }

        fn send_credentials_reset_email(&self, to: &Email, token: &Token) -> Result<()> {
            if let Some(send_credentials_reset_email_fn) = self.send_credentials_reset_email_fn {
                return send_credentials_reset_email_fn(self, to, token);
            }

            Err(Error::Debug)
        }
    }
}
