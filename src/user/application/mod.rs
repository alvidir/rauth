pub mod delete;
pub mod reset;
pub mod signup;

use super::domain::{Email, User};
use super::error::Result;
use crate::mfa::domain::Otp;
use crate::mfa::service::MfaService;
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
pub trait EventBus {
    async fn emit_user_created(&self, user: &User) -> Result<()>;
    async fn emit_user_deleted(&self, user: &User) -> Result<()>;
}

pub trait MailService {
    fn send_credentials_verification_email(&self, to: &Email, token: &Token) -> Result<()>;
    fn send_credentials_reset_email(&self, to: &Email, token: &Token) -> Result<()>;
}

pub struct UserApplication<U, S, T, F, M, B, C> {
    pub user_repo: Arc<U>,
    pub secret_repo: Arc<S>,
    pub token_srv: Arc<T>,
    pub multi_factor_srv: Arc<F>,
    pub mail_srv: Arc<M>,
    pub event_bus: Arc<B>,
    pub cache: Arc<C>,
}

impl<U, S, T, F, M, B, C> UserApplication<U, S, T, F, M, B, C>
where
    F: MfaService,
{
    /// Performs the multi factor authentication method preferred by the given user.
    async fn multi_factor(&self, user: &User, otp: Option<Otp>) -> Result<()> {
        let Some(method) = user.preferences.multi_factor else {
            return Ok(());
        };

        self.multi_factor_srv
            .execute(method, &user, otp)
            .await
            .map_err(Into::into)
    }
}
