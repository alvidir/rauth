use tonic::{
    Request, Response, Status,
};

use crate::security;
use crate::constants;
use crate::grpc;
use crate::user::application::UserRepository;
use crate::secret::application::SecretRepository;
use super::{
    application::{SessionApplication, SessionRepository},
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
    S: SessionRepository + Sync + Send,
    U: UserRepository + Sync + Send,
    E: SecretRepository + Sync + Send
    > {
    pub sess_app: SessionApplication<S, U, E>,
    pub jwt_secret: &'static [u8],
    pub jwt_public: &'static [u8],
    pub jwt_header: &'static str,
}

#[tonic::async_trait]
impl<
    S: 'static + SessionRepository + Sync + Send,
    U: 'static + UserRepository + Sync + Send,
    E: 'static + SecretRepository + Sync + Send
    > Session for SessionImplementation<S, U, E> {
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<Empty>, Status> {
        let msg_ref = request.into_inner();
        let shadowed_pwd = security::shadow(&msg_ref.pwd, constants::PWD_SUFIX);

        let token = self.sess_app.login(&msg_ref.ident, &shadowed_pwd, &msg_ref.totp, self.jwt_secret)
            .map_err(|err| Status::aborted(err.to_string()))?;

        let mut res = Response::new(Empty{});
        let secure_token = security::sign_jwt(&self.jwt_secret, token)
            .map_err(|err| {
                error!("{}: {}", constants::ERR_SIGN_TOKEN, err);
                Status::unknown(constants::ERR_SIGN_TOKEN)
            })?;

        let parsed_token = secure_token.parse()
            .map_err(|err| {
                error!("{}: {}", constants::ERR_PARSE_HEADER, err);
                Status::unknown(constants::ERR_PARSE_HEADER)
            })?;

        res.metadata_mut().append(self.jwt_header, parsed_token);
        Ok(res)
    }

    async fn logout(&self, request: Request<Empty>) -> Result<Response<Empty>, Status> {
        let token = grpc::get_header(&request, self.jwt_header)?;
        if let Err(err) = self.sess_app.logout(&token, self.jwt_public){    
            return Err(Status::aborted(err.to_string()));
        }

        Ok(Response::new(Empty{}))
    }
}