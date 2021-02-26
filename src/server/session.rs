use tonic::{Request, Response, Status};
use crate::transactions::*;
use crate::proto::user_proto;
use super::*;

// Proto generated server traits
use user_proto::session_server::Session;

// Proto message structs
use user_proto::{LoginRequest, LogoutRequest, SignupRequest, LoginResponse, DeleteRequest };

#[derive(Default)]
pub struct SessionImplementation {}

#[tonic::async_trait]
impl Session for SessionImplementation {
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<LoginResponse>, Status> {
        let msg_ref = request.into_inner();
        let tx_login = login::TxLogin::new(
            &msg_ref.ident,
            &msg_ref.pwd,
            &msg_ref.app,
        );
        
        match tx_login.execute() {
            Ok(sess) => Ok(Response::new(sess)),
            Err(err) => Err(parse_error(err))
        }
    }

    async fn logout(&self, request: Request<LogoutRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();
        let tx_logout = logout::TxLogout::new(
            &msg_ref.cookie,
        );
        
        match tx_logout.execute() {
            Ok(_) => Ok(Response::new(())),
            Err(err) => Err(parse_error(err))
        }
    }

    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();
        let tx_signup = signup::TxSignup::new(
            &msg_ref.name, 
            &msg_ref.email, 
            &msg_ref.pwd,
        );
        
        match tx_signup.execute() {
            Ok(_) => Ok(Response::new(())),
            Err(err) => Err(parse_error(err))
        }
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();
        let tx_delete = delete_user::TxDelete::new(
            &msg_ref.ident,
            &msg_ref.pwd,
        );
        
        match tx_delete.execute() {
            Ok(_) => Ok(Response::new(())),
            Err(err) => Err(parse_error(err))
        }
    }
}