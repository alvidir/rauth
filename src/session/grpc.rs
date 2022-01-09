use tonic::{Request, Response, Status};
use crate::security;
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
            .map_err(|err| Status::unknown(err.to_string()))?;

        let parsed_token = secure_token.parse()
            .map_err(|_| Status::unknown("unable to parse token"))?;

        res.metadata_mut().append(self.jwt_header, parsed_token);
        Ok(res)
    }

    async fn logout(&self, request: Request<()>) -> Result<Response<()>, Status> {
        let metadata = request.metadata();
        if metadata.get(self.jwt_header).is_none() {
            return Err(Status::failed_precondition("token required"));
        };

        // this line will not fail due to the previous check of None 
        let token = match metadata.get(self.jwt_header).unwrap().to_str() {
            Err(err) => return Err(Status::aborted(err.to_string())),
            Ok(secure_token) => security::verify_jwt::<SessionToken>(&self.jwt_public, secure_token)
                .map_err(|err| Status::unknown(err.to_string()))?,
        };

        if let Err(err) = self.sess_app.logout(token.sub){   
            error!("failed to logout user: {}", err);            
            return Err(Status::aborted(err.to_string()));
        }

        Ok(Response::new(()))
    }
}