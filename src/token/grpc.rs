use tonic::{Request, Response, Status};

use super::application::{TokenApplication, TokenRepository};
use crate::base64::B64_CUSTOM_ENGINE;
use crate::secret::application::SecretRepository;
use crate::user::application::UserRepository;
use crate::{crypto, grpc, result::Error};
use base64::Engine;

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("session");
}

// Proto generated server traits
use proto::session_server::Session;
pub use proto::session_server::SessionServer;

// Proto message structs
use proto::{Empty, LoginRequest};

pub struct SessionImplementation<
    S: TokenRepository + Sync + Send,
    U: UserRepository + Sync + Send,
    E: SecretRepository + Sync + Send,
> {
    pub sess_app: TokenApplication<'static, S, U, E>,
    pub jwt_secret: &'static [u8],
    pub jwt_public: &'static [u8],
    pub jwt_header: &'static str,
    pub pwd_sufix: &'static str,
}

#[tonic::async_trait]
impl<
        S: 'static + TokenRepository + Sync + Send,
        U: 'static + UserRepository + Sync + Send,
        E: 'static + SecretRepository + Sync + Send,
    > Session for SessionImplementation<S, U, E>
{
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<Empty>, Status> {
        let msg_ref = request.into_inner();
        let shadowed_pwd = crypto::shadow(&msg_ref.pwd, self.pwd_sufix);

        let token = self
            .sess_app
            .login(
                &msg_ref.ident,
                &shadowed_pwd,
                &msg_ref.totp,
                self.jwt_secret,
            )
            .await
            .map(|token| B64_CUSTOM_ENGINE.encode(token))
            .map_err(|err| Status::aborted(err.to_string()))?;

        let mut res = Response::new(Empty {});
        let token = token.parse().map_err(|err| {
            error!("{} parsing token to header: {}", Error::Unknown, err);
            Into::<Status>::into(Error::Unknown)
        })?;

        res.metadata_mut().append(self.jwt_header, token);
        Ok(res)
    }

    async fn logout(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let token = grpc::get_encoded_header(&request, self.jwt_header)?;
        if let Err(err) = self.sess_app.logout(&token, self.jwt_public).await {
            return Err(Status::aborted(err.to_string()));
        }

        Ok(Response::new(Empty {}))
    }
}
