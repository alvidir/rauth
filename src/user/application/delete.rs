use super::{EventBus, Mailer, UserApplication, UserRepository};
use crate::cache::Cache;
use crate::mfa::domain::Otp;
use crate::mfa::service::MfaService;
use crate::on_error;
use crate::secret::application::SecretRepository;
use crate::token::domain::Token;
use crate::token::domain::TokenKind;
use crate::token::service::TokenService;
use crate::user::domain::Password;
use crate::user::error::{Error, Result};

impl<'a, U, S, T, F, M, B, C> UserApplication<'a, U, S, T, F, M, B, C>
where
    U: UserRepository,
    S: SecretRepository,
    T: TokenService,
    F: MfaService,
    M: Mailer,
    B: EventBus,
    C: Cache,
{
    /// Given a valid session token and passwords, performs the deletion of the user.
    #[instrument(skip(self, password, otp))]
    pub async fn delete_with_token(
        &self,
        token: Token,
        password: Password,
        otp: Option<Otp>,
    ) -> Result<()> {
        let payload = self.token_service.claims(TokenKind::Session, token).await?;

        let user_id = payload
            .sub
            .parse()
            .map_err(on_error!("parsing token subject to user id"))?;

        self.delete(user_id, password, otp).await?;
        self.token_service
            .revoke(&payload)
            .await
            .map_err(Into::into)
    }

    /// Given a valid user ID and passwords, performs the deletion of the corresponding user.
    #[instrument(skip(self, password, otp))]
    pub async fn delete(&self, user_id: i32, password: Password, otp: Option<Otp>) -> Result<()> {
        let user = self.user_repo.find(user_id).await?;

        if !user.password_matches(password)? {
            return Err(Error::WrongCredentials);
        }

        if let Some(method) = user.preferences.multi_factor {
            self.mfa_service.run_method(method, &user, otp)?;
        };

        self.user_repo.delete(&user).await
    }
}
