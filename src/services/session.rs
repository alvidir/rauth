use tonic::{transport::Server, Request, Response, Status, Code};
use std::time;

use crate::transactions::{login::TxLogin, signup::TxSignup};
use crate::models::session::Controller as SessionController;
use crate::models::client::Controller as ClientController;

// Import the generated rust code into module
pub mod client_proto {
   tonic::include_proto!("client");
}

// Proto generated server traits
use client_proto::session_server::{Session, SessionServer};

// Proto message structs
use client_proto::{LoginRequest, GoogleSigninRequest, LogoutRequest, SignupRequest, SessionResponse };

pub async fn start_server(address: String) -> Result<(), Box<dyn std::error::Error>> {
    let addr = address.parse().unwrap();
    let session_server = SessionImplementation::default();
 
    println!("Session service listening on {}", addr);
 
    Server::builder()
        .add_service(SessionServer::new(session_server))
        .serve(addr)
        .await?;
 
    Ok(())
 }

#[derive(Default)]
pub struct SessionImplementation {}

impl SessionImplementation {
    fn signup_response(&self, result: Box<dyn ClientController>) -> Result<Response<SessionResponse>, Status> {
        Ok(Response::new(
            SessionResponse {
                deadline: 0,
                cookie: "".to_string(),
                status: 0,
                token: "".to_string(),
            }
        ))
    }
}

#[tonic::async_trait]
impl Session for SessionImplementation {
    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<SessionResponse>, Status> {
        let msg_ref = request.into_inner();
        let tx_signup = TxSignup::new(
            &msg_ref.name, 
            &msg_ref.addr, 
            &msg_ref.pwd,
        );
        
        match tx_signup.execute() {
            Err(err) => {
                let status = Status::new(Code::Internal, err.to_string());
                Err(status)
            }

            Ok(res) => {
                self.signup_response(res)
            }
        }
    }
    
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<SessionResponse>, Status> {
        let msg_ref = request.into_inner();
        let mut tx_login = TxLogin::new(
            &msg_ref.ident,
            &msg_ref.pwd,
        );
        
        //tx_login.execute();
        let response = SessionResponse {
            deadline: 0,
            cookie: "".to_string(),
            status: 0,
            token: "".to_string(),
        };

        Ok(Response::new(response))
    }

    async fn google_signin(&self, request: Request<GoogleSigninRequest>) -> Result<Response<SessionResponse>, Status> {
        let response = SessionResponse {
            deadline: 0,
            cookie: "".to_string(),
            status: 0,
            token: "".to_string(),
        };

        Ok(Response::new(response))
    }

    async fn logout( &self, request: Request<LogoutRequest>) -> Result<Response<SessionResponse>, Status> {
        let response = SessionResponse {
            deadline: 0,
            cookie: "".to_string(),
            status: 0,
            token: "".to_string(),
        };

        Ok(Response::new(response))
    }
    
}