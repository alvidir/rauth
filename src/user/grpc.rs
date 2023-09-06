use std::ops::Not;

use super::application::Mailer;
use super::domain::{Email, Password};
use super::error::Error;
use crate::cache::Cache;
use crate::grpc;
use crate::secret::application::SecretRepository;
use crate::user::{
    application::{EventBus, UserApplication, UserRepository},
    domain::Credentials,
};
use tonic::metadata::errors::InvalidMetadataValue;
use tonic::{Request, Response, Status};

const TOTP_ACTION_ENABLE: i32 = 0;
const TOTP_ACTION_DISABLE: i32 = 1;

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("user");
}

// Proto generated server traits
use proto::user_server::User;
pub use proto::user_server::UserServer;

// Proto message structs
use proto::{DeleteRequest, Empty, MfaRequest, ResetRequest, SignupRequest};

impl From<Error> for Status {
    fn from(error: Error) -> Status {
        // match error {
        //     Error::NotAnEmail => Status::invalid_argument("email"),
        //     Error::NotAPassword => Status::invalid_argument("password"),
        //     Error::NotFound => Status::not_found("user"),
        //     Error::Unknown => Status::unknown(""),
        // }

        Status::unknown("")
    }
}

impl TryFrom<SignupRequest> for Credentials {
    type Error = Status;

    fn try_from(value: SignupRequest) -> Result<Self, Self::Error> {
        let email: Email = value.email.try_into()?;
        let Some(password) = value
            .password
            .is_empty()
            .not()
            .then_some(value.password)
            .map(Password::try_from)
            .transpose()?
        else {
            return Ok(email.into());
        };

        Ok(Credentials::from(email).with_password(password))
    }
}

pub struct UserGrpcService<U, S, B, M, C>
where
    U: UserRepository + Sync + Send,
    S: SecretRepository + Sync + Send,
    B: EventBus + Sync + Send,
    M: Mailer,
    C: Cache,
{
    pub user_app: UserApplication<'static, U, S, B, M, C>,
    pub jwt_header: &'static str,
    pub totp_header: &'static str,
}

impl<U, S, B, M, C> UserGrpcService<U, S, B, M, C>
where
    U: 'static + UserRepository + Sync + Send,
    S: 'static + SecretRepository + Sync + Send,
    B: 'static + EventBus + Sync + Send,
    M: 'static + Mailer + Sync + Send,
    C: 'static + Cache + Sync + Send,
{
    #[instrument(skip(self))]
    async fn signup_with_token(&self, token: &str) -> Result<Response<Empty>, Status> {
        let token = self
            .user_app
            .signup_with_token(token.into())
            .await
            .map_err(Status::from)?;

        let mut res = Response::new(Empty {});
        let token = token
            .as_ref()
            .parse()
            .map_err(|err: InvalidMetadataValue| {
                error!(error = err.to_string(), "parsing token to header");
                Status::from(Error::Unknown)
            })?;

        res.metadata_mut().append(self.jwt_header, token);
        Ok(res)
    }

    #[instrument(skip(self))]
    async fn signup_with_credentials(
        &self,
        request: SignupRequest,
    ) -> Result<Response<Empty>, Status> {
        self.user_app
            .verify_credentials(request.try_into()?)
            .await
            .map_err(|err| Status::aborted(err.to_string()))?;

        Err(Error::NotAvailable.into())
    }
}

#[tonic::async_trait]
impl<U, S, B, M, C> User for UserGrpcService<U, S, B, M, C>
where
    U: 'static + UserRepository + Sync + Send,
    S: 'static + SecretRepository + Sync + Send,
    B: 'static + EventBus + Sync + Send,
    M: 'static + Mailer + Sync + Send,
    C: 'static + Cache + Sync + Send,
{
    #[instrument(skip(self))]
    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<Empty>, Status> {
        match grpc::get_header(&request, self.jwt_header) {
            Ok(token) => self.signup_with_token(&token).await,
            Err(err) if err.is => {
                let request = request.into_inner();
                self.signup_with_credentials(request).await
            }
            Err(err) => Err(err.into()),
        }
    }

    #[instrument(skip(self))]
    async fn reset(&self, request: Request<ResetRequest>) -> Result<Response<Empty>, Status> {
        if request.metadata().get(self.jwt_header).is_some() {
            let token = grpc::get_header(&request, self.jwt_header)?;
            let msg_ref = request.into_inner();
            return self
                .user_app
                .reset_credentials_with_token(&token, &msg_ref.new_password, &msg_ref.otp)
                .await
                .map(|_| Response::new(Empty {}))
                .map_err(|err| Status::aborted(err.to_string()));
        }

        let msg_ref = request.into_inner();
        self.user_app
            .verify_credentials_reset(&msg_ref.email)
            .await
            .map_err(|err| Status::aborted(err.to_string()))?;

        Err(Error::NotAvailable.into())
    }

    #[instrument(skip(self))]
    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<Empty>, Status> {
        let token = grpc::get_header(&request, self.jwt_header)?;
        let request = request.into_inner();
        self.user_app
            .delete_with_token(token.into(), request.password.try_into()?, &request.otp)
            .await
            .map(|_| Response::new(Empty {}))
            .map_err(|err| Status::aborted(err.to_string()))
    }

    #[instrument(skip(self))]
    async fn mfa(&self, request: Request<MfaRequest>) -> Result<Response<Empty>, Status> {
        let token = grpc::get_header(&request, self.jwt_header)?;
        let msg_ref = request.into_inner();

        if msg_ref.action == TOTP_ACTION_DISABLE {
            return self
                .user_app
                .disable_totp_with_token(&token, &msg_ref.password, &msg_ref.otp)
                .await
                .map(|_| Response::new(Empty {}))
                .map_err(|err| Status::unknown(err.to_string()));
        }

        if msg_ref.action == TOTP_ACTION_ENABLE {
            let token = self
                .user_app
                .enable_totp_with_token(&token, &msg_ref.password, &msg_ref.otp)
                .await
                .map_err(|err| Status::aborted(err.to_string()))?;

            let mut response = Response::new(Empty {});
            if let Some(token) = token {
                let token = token.parse().map_err(|err: InvalidMetadataValue| {
                    error!(error = err.to_string(), "parsing str to metadata",);
                    Status::aborted(Error::Unknown.to_string())
                })?;

                response.metadata_mut().insert(self.totp_header, token);
            }
        }

        Err(Error::NotAvailable.into())
    }
}
