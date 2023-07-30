use super::application::SessionApplication;
use crate::base64::B64_CUSTOM_ENGINE;
use crate::secret::application::SecretRepository;
use crate::token::application::TokenRepository;
use crate::user::application::UserRepository;
use crate::{grpc, result::Error};
use base64::Engine;
use tonic::metadata::errors::InvalidMetadataValue;
use tonic::{Request, Response, Status};

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("session");
}

// Proto generated server traits
use proto::session_server::Session;
pub use proto::session_server::SessionServer;

// Proto message structs
use proto::{Empty, LoginRequest};

pub struct SessionGrpcService<
    T: TokenRepository + Sync + Send,
    U: UserRepository + Sync + Send,
    S: SecretRepository + Sync + Send,
> {
    pub session_app: SessionApplication<'static, T, U, S>,
    pub jwt_header: &'static str,
}

#[tonic::async_trait]
impl<
        T: 'static + TokenRepository + Sync + Send,
        U: 'static + UserRepository + Sync + Send,
        S: 'static + SecretRepository + Sync + Send,
    > Session for SessionGrpcService<T, U, S>
{
    #[instrument(skip(self))]
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<Empty>, Status> {
        let msg_ref = request.into_inner();
        let token = self
            .session_app
            .login(&msg_ref.ident, &msg_ref.pwd, &msg_ref.totp)
            .await
            .map(|token| B64_CUSTOM_ENGINE.encode(token))
            .map_err(|err| Status::aborted(err.to_string()))?;

        let mut res = Response::new(Empty {});
        let token = token.parse().map_err(|err: InvalidMetadataValue| {
            error!(error = err.to_string(), "parsing token to header");
            Into::<Status>::into(Error::Unknown)
        })?;

        res.metadata_mut().append(self.jwt_header, token);
        Ok(res)
    }

    #[instrument(skip(self))]
    async fn logout(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let token = grpc::get_encoded_header(&request, self.jwt_header)?;
        if let Err(err) = self.session_app.logout(&token).await {
            return Err(Status::aborted(err.to_string()));
        }

        Ok(Response::new(Empty {}))
    }
}
