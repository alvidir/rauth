use std::error::Error;

use tonic::{
    Request, Response, Status,
    metadata::MetadataMap,
};

use crate::security;
use crate::constants;
use crate::user::application::UserRepository;
use crate::secret::application::SecretRepository;
use super::{
    application::{SessionApplication, SessionRepository},
    domain::{SessionToken},
};

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("session");
}

// Proto generated server traits
use proto::session_service_server::SessionService;
pub use proto::session_service_server::SessionServiceServer;

// Proto message structs
use proto::LoginRequest;

pub fn get_session_token(meta: &MetadataMap, public: &[u8], header: &str) -> Result<SessionToken, Box<dyn Error>> {
    if meta.get(header).is_none() {
        return Err(constants::ERR_TOKEN_REQUIRED.into());
    };

    // this line will not fail due to the previous check of None 
    let token = meta.get(header).unwrap().to_str()?;
    let claims = security::verify_jwt::<SessionToken>(public, &token)?;
    Ok(claims)
}

pub struct SessionServiceImplementation<
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
    > SessionService for SessionServiceImplementation<S, U, E> {
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();

        let token = self.sess_app.login(&msg_ref.ident, &msg_ref.pwd, &msg_ref.totp)
            .map_err(|err| Status::aborted(err.to_string()))?;

        let mut res = Response::new(());
        let secure_token = security::sign_jwt(&self.jwt_secret, token)
            .map_err(|err| {
                error!("{}: {}", constants::ERR_SIGN_TOKEN, err);
                Status::unknown(constants::ERR_SIGN_TOKEN)
            })?;

        let parsed_token = secure_token.parse()
            .map_err(|err| {
                error!("{}: {}", constants::ERR_PARSE_TOKEN, err);
                Status::unknown(constants::ERR_PARSE_TOKEN)
            })?;

        res.metadata_mut().append(self.jwt_header, parsed_token);
        Ok(res)
    }

    async fn logout(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let metadata = request.metadata();
        if metadata.get(self.jwt_header).is_none() {
            return Err(Status::failed_precondition(constants::ERR_TOKEN_REQUIRED));
        };

        // this line will not fail due to the previous check of None 
        let token = match metadata.get(self.jwt_header).unwrap().to_str() {
            Ok(secure_token) => security::verify_jwt::<SessionToken>(&self.jwt_public, secure_token)
                .map_err(|err| {
                    warn!("{}: {}", constants::ERR_VERIFY_TOKEN, err);
                    Status::unknown(constants::ERR_VERIFY_TOKEN)
                })?,
            
            Err(err) => {
                error!("{}: {}", constants::ERR_PARSE_TOKEN, err);
                return Err(Status::aborted(constants::ERR_PARSE_TOKEN))
            },
        };

        if let Err(err) = self.sess_app.logout(token.sub){    
            return Err(Status::aborted(err.to_string()));
        }

        Ok(Response::new(()))
    }
}