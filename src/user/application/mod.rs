pub mod delete;
pub mod signup;

use super::domain::{Email, User};
use super::error::Result;
use crate::token::domain::Token;
use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;

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

pub trait Mailer {
    fn send_credentials_verification_email(&self, to: &Email, token: &Token) -> Result<()>;
    fn send_credentials_reset_email(&self, to: &Email, token: &Token) -> Result<()>;
}

pub struct UserApplication<'a, U, S, T, F, M, B, C> {
    pub user_repo: Arc<U>,
    pub secret_repo: Arc<S>,
    pub token_service: Arc<T>,
    pub mfa_service: Arc<F>,
    pub mailer: Arc<M>,
    pub event_bus: Arc<B>,
    pub cache: Arc<C>,
}
