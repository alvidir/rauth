use tonic::{Request, Response, Status};
use crate::security;
use crate::user::application::{UserRepository, UserApplication};
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

pub struct UserServiceImplementation<UR: UserRepository + Sync + Send, SR: SessionRepository + Sync + Send> {
    pub user_app: UserApplication<UR, SR>,
    pub jwt_secret: Vec<u8>,
    pub jwt_public: Vec<u8>,
}

#[tonic::async_trait]
impl<
    UR: 'static + UserRepository + Sync + Send,
    SR: 'static + SessionRepository + Sync + Send
    > UserService for UserServiceImplementation<UR, SR> {
    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<()>, Status> {
        if let None = request.metadata().get("token") {
            return Err(Status::failed_precondition("token required"));
        };
    
        // this line will not fail due to the previous check of None 
        let token = match request.metadata().get("token").unwrap().to_str() {
            Err(err) => return Err(Status::aborted(err.to_string())),
            Ok(token) => token.to_string(),
        };

        let claims = match security::decode_jwt::<VerificationToken>(&self.jwt_public, &token) {
            Ok(claims) => claims,
            Err(err) => return Err(Status::aborted(err.to_string())),
        };

        let msg_ref = request.into_inner();
        match self.user_app.signup(claims, &msg_ref.email, &msg_ref.pwd) {
            Err(err) => Err(Status::aborted(err.to_string())),
            Ok(_) => Ok(Response::new(())),
        }
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<()>, Status> {
        if let None = request.metadata().get("token") {
            return Err(Status::failed_precondition("token required"));
        };

        // this line will not fail due to the previous check of None 
        let token = match request.metadata().get("token").unwrap().to_str() {
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
        if let None = request.metadata().get("token") {
            return Err(Status::failed_precondition("token required"));
        };

        // this line will not fail due to the previous check of None 
        let token = match request.metadata().get("token").unwrap().to_str() {
            Err(err) => return Err(Status::aborted(err.to_string())),
            Ok(token) => token.to_string(),
        };

        let claims = match security::decode_jwt::<SessionToken>(&self.jwt_public, &token) {
            Ok(claims) => claims,
            Err(err) => return Err(Status::aborted(err.to_string())),
        };

        let msg_ref = request.into_inner(); 
        let result = match msg_ref.action {
            0 => self.user_app.enable_totp(claims.sub, &msg_ref.pwd, &msg_ref.totp),
            1 => self.user_app.disable_totp(claims.sub, &msg_ref.pwd, &msg_ref.totp),
            _ => return Err(Status::invalid_argument("wrong action")),
        };

        if let Err(err) = result {
            return Err(Status::aborted(err.to_string()));
        }

        Ok(Response::new(()))
    }
}