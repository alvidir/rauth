use super::application::Mailer;
use crate::base64::B64_CUSTOM_ENGINE;
use crate::cache::Cache;
use crate::secret::application::SecretRepository;
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
    S: SecretRepository + Sync + Send,
    B: EventBus + Sync + Send,
    M: Mailer,
    C: Cache,
> {
    pub user_app: UserApplication<'static, U, S, B, M, C>,
    pub jwt_header: &'static str,
    pub totp_header: &'static str,
}

impl<
        U: 'static + UserRepository + Sync + Send,
        S: 'static + SecretRepository + Sync + Send,
        B: 'static + EventBus + Sync + Send,
        M: 'static + Mailer + Sync + Send,
        C: 'static + Cache + Sync + Send,
    > UserGrpcService<U, S, B, M, C>
{
    #[instrument(skip(self))]
    async fn signup_with_token(&self, token: &str) -> Result<Response<Empty>, Status> {
        let token = self
            .user_app
            .signup_with_token(token)
            .await
            .map(|token| B64_CUSTOM_ENGINE.encode(token))
            .map_err(Status::from)?;

        let mut res = Response::new(Empty {});
        let token = token.parse().map_err(|err: InvalidMetadataValue| {
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
            .verify_credentials(&request.email, &request.pwd)
            .await
            .map_err(|err| Status::aborted(err.to_string()))?;

        Err(Error::NotAvailable.into())
    }
}

#[tonic::async_trait]
impl<
        U: 'static + UserRepository + Sync + Send,
        S: 'static + SecretRepository + Sync + Send,
        B: 'static + EventBus + Sync + Send,
        M: 'static + Mailer + Sync + Send,
        C: 'static + Cache + Sync + Send,
    > User for UserGrpcService<U, S, B, M, C>
{
    #[instrument(skip(self))]
    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<Empty>, Status> {
        match grpc::get_encoded_header(&request, self.jwt_header) {
            Ok(token) => self.signup_with_token(&token).await,
            Err(err) if matches!(err, Error::NotFound) => {
                let request = request.into_inner();
                self.signup_with_credentials(request).await
            }
            Err(err) => Err(err.into()),
        }
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
        let request = request.into_inner();
        self.user_app
            .delete_with_token(&token, &request.pwd, &request.totp)
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
