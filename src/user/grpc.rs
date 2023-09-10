use super::application::MailService;
use super::error::Error;
use crate::cache::Cache;
use crate::grpc;
use crate::mfa::domain::MfaMethod;
use crate::mfa::service::MfaService;
use crate::on_error;
use crate::secret::application::SecretRepository;
use crate::token::domain::Token;
use crate::token::service::TokenService;
use crate::user::application::{EventBus, UserApplication, UserRepository};
use std::ops::Not;
use tonic::metadata::errors::InvalidMetadataValue;
use tonic::{Request, Response, Status};

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("user");
}

// Proto generated server traits
use proto::user_server::User;
pub use proto::user_server::UserServer;

// Proto message structs
use proto::{
    mfa_request::{Actions, Methods},
    DeleteRequest, Empty, MfaRequest, ResetRequest, SignupRequest,
};

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
        let token = session_token.as_ref().parse().map_err(on_error!(
            InvalidMetadataValue as Error,
            "parsing token to header"
        ))?;

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
        let Some(header) = grpc::header(&request, self.jwt_header).map_err(Status::from)? else {
            let request = request.into_inner();
            return self.signup_with_credentials(request).await;
        };

        let token = header.try_into().map_err(Status::from)?;
        self.signup_with_token(token).await
    }

    #[instrument(skip(self))]
    async fn reset(&self, request: Request<ResetRequest>) -> Result<Response<Empty>, Status> {
        let Some(header) = grpc::header(&request, self.jwt_header).map_err(Status::from)? else {
            let request = request.into_inner();
            let email = request.email.try_into().map_err(Status::from)?;

            return self
                .user_app
                .verify_credentials_reset(email)
                .await
                .map(|_| Response::new(Empty {}))
                .map_err(Status::from);
        };

        let request = request.into_inner();
        let token = header.try_into().map_err(Status::from)?;
        let new_password = request.new_password.try_into().map_err(Status::from)?;
        let otp = request
            .otp
            .is_empty()
            .not()
            .then_some(request.otp.to_string())
            .map(TryInto::try_into)
            .transpose()
            .map_err(Status::from)?;

        self.user_app
            .reset_credentials_with_token(token, new_password, otp)
            .await
            .map(|_| Response::new(Empty {}))
            .map_err(Status::from)
    }

    #[instrument(skip(self))]
    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<Empty>, Status> {
        let Some(header) = grpc::header(&request, self.jwt_header).map_err(Status::from)? else {
            return Err(Error::Forbidden).map_err(Into::into);
        };

        let request = request.into_inner();
        let token = header.try_into().map_err(Status::from)?;
        let password = request.password.try_into().map_err(Status::from)?;
        let otp = request
            .otp
            .is_empty()
            .not()
            .then_some(request.otp.to_string())
            .map(TryInto::try_into)
            .transpose()
            .map_err(Status::from)?;

        self.user_app
            .delete_with_token(token, password, otp)
            .await
            .map(|_| Response::new(Empty {}))
            .map_err(|err| Status::aborted(err.to_string()))
    }

    #[instrument(skip(self))]
    async fn mfa(&self, request: Request<MfaRequest>) -> Result<Response<Empty>, Status> {
        let Some(header) = grpc::header(&request, self.jwt_header).map_err(Status::from)? else {
            return Err(Error::Forbidden).map_err(Into::into);
        };

        let request = request.into_inner();
        let token = header.try_into().map_err(Status::from)?;
        let password = request.password.try_into().map_err(Status::from)?;
        let otp = request
            .otp
            .is_empty()
            .not()
            .then_some(request.otp.to_string())
            .map(TryInto::try_into)
            .transpose()
            .map_err(Status::from)?;

        let method =
            match Methods::from_i32(request.method).ok_or(Status::invalid_argument("method"))? {
                Methods::Email => MfaMethod::Email,
                Methods::TpApp => MfaMethod::TpApp,
            };

        match Actions::from_i32(request.action).ok_or(Status::invalid_argument("action"))? {
            Actions::Enable => {
                self.user_app
                    .enable_mfa_with_token(token, method, password, otp)
                    .await
            }
            Actions::Disable => {
                self.user_app
                    .disable_mfa_with_token(token, method, password, otp)
                    .await
            }
        }
        .map(|_| Response::new(Empty {}))
        .map_err(Into::into)
    }
}
