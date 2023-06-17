use super::application::Mailer;
use crate::base64::B64_CUSTOM_ENGINE;
use crate::secret::application::SecretRepository;
use crate::token::application::TokenRepository;
use crate::user::application::{EventBus, UserApplication, UserRepository};
use crate::{grpc, result::Error};
use base64::Engine;
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
use proto::{DeleteRequest, Empty, ResetRequest, SignupRequest, TotpRequest};

pub struct UserGrpcService<
    U: UserRepository + Sync + Send,
    E: SecretRepository + Sync + Send,
    S: TokenRepository + Sync + Send,
    B: EventBus + Sync + Send,
    M: Mailer,
> {
    pub user_app: UserApplication<'static, U, E, S, B, M>,
    pub jwt_header: &'static str,
    pub totp_header: &'static str,
}

#[tonic::async_trait]
impl<
        U: 'static + UserRepository + Sync + Send,
        E: 'static + SecretRepository + Sync + Send,
        S: 'static + TokenRepository + Sync + Send,
        B: 'static + EventBus + Sync + Send,
        M: 'static + Mailer + Sync + Send,
    > User for UserGrpcService<U, E, S, B, M>
{
    #[instrument(skip(self))]
    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<Empty>, Status> {
        if request.metadata().get(self.jwt_header).is_some() {
            let token = grpc::get_encoded_header(&request, self.jwt_header)?;
            let token = self
                .user_app
                .signup_with_token(&token)
                .await
                .map(|token| B64_CUSTOM_ENGINE.encode(token))
                .map_err(|err| Status::aborted(err.to_string()))?;

            let mut res = Response::new(Empty {});
            let token = token.parse().map_err(|err: InvalidMetadataValue| {
                error!(error = err.to_string(), "parsing token to header");
                Into::<Status>::into(Error::Unknown)
            })?;
            res.metadata_mut().append(self.jwt_header, token);
            return Ok(res);
        }

        let msg_ref = request.into_inner();
        self.user_app
            .verify_signup_email(&msg_ref.email, &msg_ref.pwd)
            .await
            .map_err(|err| Status::aborted(err.to_string()))?;

        Err(Error::NotAvailable.into())
    }

    #[instrument(skip(self))]
    async fn reset(&self, request: Request<ResetRequest>) -> Result<Response<Empty>, Status> {
        if request.metadata().get(self.jwt_header).is_some() {
            let token = grpc::get_encoded_header(&request, self.jwt_header)?;
            let msg_ref = request.into_inner();
            return self
                .user_app
                .reset_with_token(&token, &msg_ref.pwd, &msg_ref.totp)
                .await
                .map(|_| Response::new(Empty {}))
                .map_err(|err| Status::aborted(err.to_string()));
        }

        let msg_ref = request.into_inner();
        self.user_app
            .verify_reset_email(&msg_ref.email)
            .await
            .map_err(|err| Status::aborted(err.to_string()))?;

        Err(Error::NotAvailable.into())
    }

    #[instrument(skip(self))]
    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<Empty>, Status> {
        let token = grpc::get_encoded_header(&request, self.jwt_header)?;
        let msg_ref = request.into_inner();
        self.user_app
            .delete_with_token(&token, &msg_ref.pwd, &msg_ref.totp)
            .await
            .map(|_| Response::new(Empty {}))
            .map_err(|err| Status::aborted(err.to_string()))
    }

    #[instrument(skip(self))]
    async fn totp(&self, request: Request<TotpRequest>) -> Result<Response<Empty>, Status> {
        let token = grpc::get_encoded_header(&request, self.jwt_header)?;
        let msg_ref = request.into_inner();

        if msg_ref.action == TOTP_ACTION_DISABLE {
            return self
                .user_app
                .disable_totp_with_token(&token, &msg_ref.pwd, &msg_ref.totp)
                .await
                .map(|_| Response::new(Empty {}))
                .map_err(|err| Status::unknown(err.to_string()));
        }

        if msg_ref.action == TOTP_ACTION_ENABLE {
            let token = self
                .user_app
                .enable_totp_with_token(&token, &msg_ref.pwd, &msg_ref.totp)
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
