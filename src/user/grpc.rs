use tonic::{Request, Response, Status};
use crate::security;
use crate::user::application::{UserRepository, UserApplication};
use crate::secret::application::SecretRepository;
use crate::session::{
    application::SessionRepository,
    domain::{SessionToken, VerificationToken},
};

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("user");
}

// Proto generated server traits
use proto::user_service_server::UserService;
pub use proto::user_service_server::UserServiceServer;

// Proto message structs
use proto::{SignupRequest, DeleteRequest, TotpRequest};

pub struct UserServiceImplementation<
    U: UserRepository + Sync + Send,
    S: SessionRepository + Sync + Send,
    E:  SecretRepository + Sync + Send
    > {
    pub user_app: UserApplication<U, S, E>,
    pub jwt_secret: &'static [u8],
    pub jwt_public: &'static [u8],
    pub jwt_header: &'static str,
}

#[tonic::async_trait]
impl<
    U: 'static + UserRepository + Sync + Send,
    S: 'static + SessionRepository + Sync + Send,
    E: 'static + SecretRepository + Sync + Send
    > UserService for UserServiceImplementation<U, S, E> {
    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<()>, Status> {
        if request.metadata().get(self.jwt_header).is_none() {
            let msg_ref = request.into_inner();
            match self.user_app.signup(&msg_ref.email, &msg_ref.pwd) {
                Err(err) => return Err(Status::aborted(err.to_string())),
                Ok(_) => return Ok(Response::new(())),
            };
        };
    
        // this line will not fail due to the previous check of None 
        let token = match request.metadata().get(self.jwt_header).unwrap().to_str() {
            Err(err) => return Err(Status::aborted(err.to_string())),
            Ok(token) => token.to_string(),
        };

        let claims = match security::decode_jwt::<VerificationToken>(&self.jwt_public, &token) {
            Ok(claims) => claims,
            Err(err) => return Err(Status::aborted(err.to_string())),
        };

        let msg_ref = request.into_inner();
        if claims.sub.is_some() && claims.pwd.is_none() {
            // this line will not fail due to the previous check of Some
            match self.user_app.signup(&claims.sub.unwrap(), &msg_ref.pwd) {
                Err(err) => return Err(Status::aborted(err.to_string())),
                Ok(_) => return Ok(Response::new(())),
            };
        }

        if claims.sub.is_some() && claims.pwd.is_some() {
            // this line will not fail due to the previous check of Some
            match self.user_app.signup(&claims.sub.unwrap(), &claims.pwd.unwrap()) {
                Err(err) => return Err(Status::aborted(err.to_string())),
                Ok(_) => return Ok(Response::new(())),
            };
        }

        Err(Status::invalid_argument("bad request"))
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<()>, Status> {
        if let None = request.metadata().get(self.jwt_header) {
            return Err(Status::failed_precondition("token required"));
        };

        // this line will not fail due to the previous check of None 
        let token = match request.metadata().get(self.jwt_header).unwrap().to_str() {
            Err(err) => return Err(Status::aborted(err.to_string())),
            Ok(token) => token.to_string(),
        };

        let claims = match security::decode_jwt::<SessionToken>(&self.jwt_public, &token) {
            Ok(claims) => claims,
            Err(err) => return Err(Status::aborted(err.to_string())),
        };

        let msg_ref = request.into_inner();
        match self.user_app.delete(claims.sub, &msg_ref.pwd, &msg_ref.totp) {
            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(()) => Ok(Response::new(())),
        }
    }

    async fn totp(&self, request: Request<TotpRequest>) -> Result<Response<()>, Status> {
        if let None = request.metadata().get(self.jwt_header) {
            return Err(Status::failed_precondition("token required"));
        };

        // this line will not fail due to the previous check of None 
        let token = match request.metadata().get(self.jwt_header).unwrap().to_str() {
            Err(err) => return Err(Status::aborted(err.to_string())),
            Ok(token) => token.to_string(),
        };

        let claims = match security::decode_jwt::<SessionToken>(&self.jwt_public, &token) {
            Ok(claims) => claims,
            Err(err) => return Err(Status::aborted(err.to_string())),
        };

        let msg_ref = request.into_inner(); 
        if msg_ref.action == 0 {
            match self.user_app.enable_totp(claims.sub, &msg_ref.pwd, &msg_ref.totp) {
                Err(err) => {
                    error!("cannot enable the totp: {:?}", err);
                    return Err(Status::unknown(err.to_string()))
                },
                Ok(token) => {
                    let mut response = Response::new(());
                    response.metadata_mut().insert(self.jwt_header, token.parse().unwrap());
                    return Ok(response);
                }
            }
        }

        if msg_ref.action == 1 {
            match self.user_app.disable_totp(claims.sub, &msg_ref.pwd, &msg_ref.totp) {
                Ok(_) => return Ok(Response::new(())),
                Err(err) => {
                    error!("cannot disable the totp: {:?}", err);
                    return Err(Status::unknown(err.to_string()))
                },
            }
        }

        Err(Status::invalid_argument("wrong action"))
    }
}