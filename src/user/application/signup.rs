use super::{EventService, MailService, UserApplication, UserRepository};
use crate::cache::Cache;
use crate::mfa::service::MfaService;
use crate::token::domain::{Token, TokenKind};
use crate::token::service::TokenService;
use crate::user::domain::{Credentials, Email, Password, PasswordHash, Salt, User};
use crate::user::error::{Error, Result};

impl<U, S, T, F, M, B, C> UserApplication<U, S, T, F, M, B, C>
where
    U: UserRepository,
    T: TokenService,
    F: MfaService,
    M: MailService,
    B: EventService,
    C: Cache,
{
    /// Stores the given credentials in the cache and sends an email with the token to be
    /// passed as parameter to the signup_with_token method.
    #[instrument(skip(self, password))]
    pub async fn verify_credentials(&self, email: Email, password: Password) -> Result<()> {
        if self.user_repo.find_by_email(&email).await.is_ok() {
            // TODO: grpc layer must ingore this error!!!
            return Error::AlreadyExists.into();
        }

        let salt = Salt::with_length(self.hash_length)?;
        let credentials = Credentials {
            email,
            password: PasswordHash::with_salt(&password, &salt)?,
        };

        let key = credentials.hash();
        let claims = self
            .token_srv
            .issue(TokenKind::Verification, &key.to_string())
            .await?;

        self.cache
            .save(&key.to_string(), &credentials, claims.payload().timeout())
            .await?;

        self.mail_srv
            .send_credentials_verification_email(&credentials.email, claims.token())?;

        Ok(())
    }

    /// Given a valid verification token, performs the signup of the corresponding user.
    #[instrument(skip(self))]
    pub async fn signup_with_token(&self, token: Token) -> Result<Token> {
        let claims = self.token_srv.claims(token).await?;

        if !claims.payload().kind().is_verification() {
            return Error::WrongToken.into();
        }

        let mut user = self
            .cache
            .find(claims.payload().subject())
            .await
            .map(Credentials::into)?;

        self.token_srv.revoke(&claims).await?;
        self.signup(&mut user).await
    }

    /// Performs the signup for the given user.
    #[instrument(skip(self))]
    pub async fn signup(&self, user: &mut User) -> Result<Token> {
        self.user_repo.create(user).await?;
        // TODO: implement outbox pattern for events publishment
        self.event_srv.emit_user_created(user).await?;

        self.token_srv
            .issue(TokenKind::Session, &user.id.to_string())
            .await
            .map_err(Into::into)
            .map(Into::into)
    }
}

#[cfg(test)]
mod test {}
