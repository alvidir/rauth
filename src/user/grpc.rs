use std::ops::Not;

use super::application::MailService;
use super::domain::{Email, PasswordHash};
use super::error::Error;
use crate::cache::Cache;
use crate::grpc;
use crate::mfa::service::MfaService;
use crate::on_error;
use crate::secret::application::SecretRepository;
use crate::token::domain::Token;
use crate::token::service::TokenService;
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

pub struct UserGrpcService<U, S, T, F, M, B, C> {
    pub user_app: UserApplication<U, S, T, F, M, B, C>,
    pub jwt_header: &'static str,
    pub totp_header: &'static str,
}

impl<U, S, T, F, M, B, C> UserGrpcService<U, S, T, F, M, B, C>
where
    U: 'static + UserRepository + Sync + Send,
    S: 'static + SecretRepository + Sync + Send,
    T: 'static + TokenService + Sync + Send,
    F: 'static + MfaService + Sync + Send,
    B: 'static + EventBus + Sync + Send,
    M: 'static + MailService + Sync + Send,
    C: 'static + Cache + Sync + Send,
{
    #[instrument(skip(self))]
    async fn signup_with_token(&self, token: Token) -> Result<Response<Empty>, Status> {
        let session_token = self
            .user_app
            .signup_with_token(token)
            .await
            .map_err(Status::from)?;

        let mut res = Response::new(Empty {});
        let token = token
            .as_ref()
            .parse()
            .map_err(on_error!("parsing token to header"))?;

        res.metadata_mut().append(self.jwt_header, token);
        Ok(res)
    }

    #[instrument(skip(self))]
    async fn signup_with_credentials(
        &self,
        request: SignupRequest,
    ) -> Result<Response<Empty>, Status> {
        let email = request.email.try_into()?;
        let password = request.password.try_into()?;

        self.user_app
            .verify_credentials(email, password)
            .await
            .map(|_| Response::new(Empty {}))
            .map_err(Status::from)
    }
}

#[tonic::async_trait]
impl<U, S, T, F, M, B, C> User for UserGrpcService<U, S, T, F, M, B, C>
where
    U: 'static + UserRepository + Sync + Send,
    S: 'static + SecretRepository + Sync + Send,
    T: 'static + TokenService + Sync + Send,
    F: 'static + MfaService + Sync + Send,
    B: 'static + EventBus + Sync + Send,
    M: 'static + MailService + Sync + Send,
    C: 'static + Cache + Sync + Send,
{
    #[instrument(skip(self))]
    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<Empty>, Status> {
        let Some(header) = grpc::get_header(&request, self.jwt_header)? else {
            let request = request.into_inner();
            return self.signup_with_credentials(request).await;
        };

        let token = header.try_into().map_err(Into::into)?;
        self.signup_with_token(token).await
    }

    #[instrument(skip(self))]
    async fn reset(&self, request: Request<ResetRequest>) -> Result<Response<Empty>, Status> {
        // if request.metadata().get(self.jwt_header).is_some() {
        //     let token = grpc::get_header(&request, self.jwt_header)?;
        //     let msg_ref = request.into_inner();
        //     return self
        //         .user_app
        //         .reset_credentials_with_token(&token, &msg_ref.new_password, &msg_ref.otp)
        //         .await
        //         .map(|_| Response::new(Empty {}))
        //         .map_err(|err| Status::aborted(err.to_string()));
        // }

        // let msg_ref = request.into_inner();
        // self.user_app
        //     .verify_credentials_reset(&msg_ref.email)
        //     .await
        //     .map_err(|err| Status::aborted(err.to_string()))?;

        // Err(Error::NotAvailable.into())

        let Some(header) = grpc::get_header(&request, self.jwt_header)? else {
            let request = request.into_inner();
            return self.signup_with_credentials(request).await;
        };

        let token = header.try_into().map_err(Into::into)?;
        self.signup_with_token(token).await
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
