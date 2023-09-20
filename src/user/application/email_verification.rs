use super::{MailService, UserApplication, UserRepository};
use crate::{
    cache::Cache,
    token::{domain::TokenKind, service::TokenService},
    user::{
        domain::{CredentialsPrelude, Email, Password, PasswordHash, Salt},
        error::{Error, Result},
    },
    Command,
};
use async_trait::async_trait;
use std::sync::Arc;

/// Represents the email verification transaction.
///
/// It temporally stores the associated credentials in the cache and sends an email with the corresponding verification token.
pub struct EmailVerification<U, T, M, C> {
    pub email: Email,
    pub password: Option<Password>,
    pub hash_length: usize,
    pub user_repo: Arc<U>,
    pub token_srv: Arc<T>,
    pub mail_srv: Arc<M>,
    pub cache: Arc<C>,
}

impl<U, T, M, C> EmailVerification<U, T, M, C>
where
    U: UserRepository,
    T: TokenService,
    M: MailService,
    C: Cache,
{
    pub fn with_password(mut self, password: Password) -> Self {
        self.password = Some(password);
        self
    }
}

#[async_trait]
impl<U, T, M, C> Command for EmailVerification<U, T, M, C>
where
    U: UserRepository + Sync + Send,
    T: TokenService + Sync + Send,
    M: MailService + Sync + Send,
    C: Cache + Sync + Send,
{
    type Result = Result<()>;

    #[instrument(skip(self))]
    async fn execute(self) -> Self::Result {
        let Err(err) = self.user_repo.find_by_email(&self.email).await else {
            return Error::AlreadyExists.into();
        };

        if !err.not_found() {
            return Err(err);
        }

        let credentials_prelude = CredentialsPrelude {
            email: self.email,
            password: self
                .password
                .map(|password| {
                    let salt = Salt::with_length(self.hash_length)?;
                    PasswordHash::with_salt(&password, &salt)
                })
                .transpose()?,
        };

        let key = credentials_prelude.hash();
        let claims = self
            .token_srv
            .issue(TokenKind::Verification, &key.to_string())
            .await?;

        self.cache
            .save(
                &key.to_string(),
                &credentials_prelude,
                claims.payload().timeout(),
            )
            .await?;

        self.mail_srv
            .send_credentials_verification_email(&credentials_prelude.email, claims.token())?;

        Ok(())
    }
}

impl<U, S, T, F, M, B, C> UserApplication<U, S, T, F, M, B, C>
where
    U: UserRepository + Sync + Send,
    T: TokenService + Sync + Send,
    M: MailService + Sync + Send,
    C: Cache + Sync + Send,
{
    /// Returns a new [EmailVerification] transaction
    pub fn email_verification(&self, email: Email, password: Option<Password>) -> impl Command {
        EmailVerification {
            email,
            password,
            hash_length: self.hash_length,
            user_repo: self.user_repo.clone(),
            token_srv: self.token_srv.clone(),
            mail_srv: self.mail_srv.clone(),
            cache: self.cache.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::EmailVerification;
    use crate::{
        cache::test::InMemoryCache,
        token::{
            domain::{Claims, Payload, Token, TokenKind},
            service::test::TokenServiceMock,
        },
        user::{
            application::test::{MailServiceMock, UserRepositoryMock},
            domain::{Credentials, Email, Password, PasswordHash, Preferences, Salt, User},
            error::Error,
        },
        Command,
    };
    use std::sync::Arc;
    use std::time::Duration;

    fn new_email_verification(
        email: Email,
        password: Option<Password>,
    ) -> EmailVerification<UserRepositoryMock, TokenServiceMock, MailServiceMock, InMemoryCache>
    {
        EmailVerification {
            email,
            password,
            hash_length: Default::default(),
            user_repo: Default::default(),
            token_srv: Default::default(),
            mail_srv: Default::default(),
            cache: Default::default(),
        }
    }
}
