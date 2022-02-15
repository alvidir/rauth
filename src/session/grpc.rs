use tonic::{
    Request, Response, Status,
};

use crate::{security, constants, grpc};
use crate::user::application::UserRepository;
use crate::secret::application::SecretRepository;
use super::{
    application::{SessionApplication, TokenRepository},
};

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("session");
}

// Proto generated server traits
use proto::session_server::Session;
pub use proto::session_server::SessionServer;

// Proto message structs
use proto::{LoginRequest, Empty};

pub struct SessionImplementation<
    S: TokenRepository + Sync + Send,
    U: UserRepository + Sync + Send,
    E: SecretRepository + Sync + Send
    > {
    pub sess_app: SessionApplication<S, U, E>,
    pub jwt_secret: &'static [u8],
    pub jwt_public: &'static [u8],
    pub jwt_header: &'static str,
    pub pwd_sufix: &'static str,
}

#[tonic::async_trait]
impl<
    S: 'static + TokenRepository + Sync + Send,
    U: 'static + UserRepository + Sync + Send,
    E: 'static + SecretRepository + Sync + Send
    > Session for SessionImplementation<S, U, E> {
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<Empty>, Status> {
        let msg_ref = request.into_inner();
        let shadowed_pwd = security::shadow(&msg_ref.pwd, self.pwd_sufix);

        let token = self.sess_app.login(&msg_ref.ident, &shadowed_pwd, &msg_ref.totp, self.jwt_secret)
            .map(|token| base64::encode(token))
            .map_err(|err| Status::aborted(err.to_string()))?;

        let mut res = Response::new(Empty{});
        let token = token.parse()
            .map_err(|err| {
                error!("{} parsing token to header: {}", constants::ERR_UNKNOWN, err);
                Status::unknown(constants::ERR_UNKNOWN)
            })?;

        res.metadata_mut().append(self.jwt_header, token);
        Ok(res)
    }

    async fn logout(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let token = grpc::get_encoded_header(&request, self.jwt_header)?;        
        if let Err(err) = self.sess_app.logout(&token, self.jwt_public){    
            return Err(Status::aborted(err.to_string()));
        }

        Ok(Response::new(Empty{}))
    }
}