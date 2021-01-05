#![allow(unused)]
use std::error::Error;
use tonic::{transport::Server, Request, Response, Status};
use crate::transactions::*;
use crate::proto::client_proto;

// Proto generated server traits
use client_proto::session_server::{Session, SessionServer};

// Proto message structs
use client_proto::{LoginRequest, GoogleSigninRequest, LogoutRequest, SignupRequest, SessionResponse };

pub async fn start_server(address: String) -> Result<(), Box<dyn Error>> {
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

#[tonic::async_trait]
impl Session for SessionImplementation {
    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<SessionResponse>, Status> {
        let msg_ref = request.into_inner();
        let tx_signup = signup::TxSignup::new(
            &msg_ref.name, 
            &msg_ref.addr, 
            &msg_ref.pwd,
        );
        
        tx_signup.execute()
    }
    
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<SessionResponse>, Status> {
        let msg_ref = request.into_inner();
        let tx_login = login::TxLogin::new(
            &msg_ref.cookie,
            &msg_ref.email,
            &msg_ref.pwd,
        );
        
        tx_login.execute()
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

    async fn logout(&self, request: Request<LogoutRequest>) -> Result<Response<SessionResponse>, Status> {
        let msg_ref = request.into_inner();
        let tx_logout = logout::TxLogout::new(
            &msg_ref.cookie,
        );
        
        tx_logout.execute()
    }
    
}