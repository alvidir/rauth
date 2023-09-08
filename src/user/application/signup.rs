use super::{EventBus, Mailer, UserApplication, UserRepository};
use crate::cache::Cache;
use crate::crypto;
use crate::mfa::service::MfaService;
use crate::token::domain::{Token, TokenKind};
use crate::token::service::TokenService;
use crate::user::domain::{Credentials, Email, Password, User};
use crate::user::error::{Error, Result};

impl<'a, U, S, T, F, M, B, C> UserApplication<'a, U, S, T, F, M, B, C>
where
    U: UserRepository,
    T: TokenService,
    F: MfaService,
    M: Mailer,
    B: EventBus,
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

        let credentials = Credentials {
            email,
            password: password.try_into()?,
        };

        let key = crypto::hash(&credentials);
        let payload = self
            .token_service
            .new_payload(TokenKind::Verification, key.to_string());

        self.cache
            .save(&key.to_string(), &credentials, Some(payload.timeout()))
            .await?;

        let token = self.token_service.issue(payload).await?;
        self.mailer
            .send_credentials_verification_email(&credentials.email, &token)?;

        Ok(())
    }

    /// Given a valid verification token, performs the signup of the corresponding user.
    #[instrument(skip(self))]
    pub async fn signup_with_token(&self, token: Token) -> Result<Token> {
        let payload = self
            .token_service
            .claims(TokenKind::Verification, token)
            .await?;
        self.token_service.revoke(&payload).await?;

        let mut user = self
            .cache
            .find(&payload.subject())
            .await
            .map(Credentials::into)?;

        self.signup(&mut user).await
    }

    /// Performs the signup for the given user.
    #[instrument(skip(self))]
    pub async fn signup(&self, user: &mut User) -> Result<Token> {
        self.user_repo.create(user).await?;
        // TODO: implement outbox pattern for events publishment
        self.event_bus.emit_user_created(user).await?;

        // FIXME: use session application for loging in
        let payload = self
            .token_service
            .new_payload(TokenKind::Session, user.id.to_string());

        self.token_service.issue(payload).await.map_err(Into::into)
    }
}
