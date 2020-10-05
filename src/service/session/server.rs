use tonic::{transport::Server, Request, Response, Status};
use crate::service::session::transactions::factory as TxFactory;

// Import the generated rust code into module
pub mod session_proto {
   tonic::include_proto!("session");
}

// Proto generated server traits
use session_proto::session_server::{Session, SessionServer};

// Proto message structs
use session_proto::{LoginRequest, GoogleLoginRequest, LogoutRequest, SignupRequest, SessionResponse };


#[derive(Default)]
pub struct SessionImplementation {}

#[tonic::async_trait]
impl Session for SessionImplementation {
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<SessionResponse>, Status> {
        let mut tx_login = TxFactory::new_tx_login();
        tx_login.execute();

        let response = SessionResponse {
            deadline: 0,
            key: "".to_string(),
            status: 0,
        };

        Ok(Response::new(response))
    }

    async fn google_login(&self, request: Request<GoogleLoginRequest>) -> Result<Response<SessionResponse>, Status> {
        let response = SessionResponse {
            deadline: 0,
            key: "".to_string(),
            status: 0,
        };

        Ok(Response::new(response))
    }

    async fn logout( &self, request: Request<LogoutRequest>) -> Result<Response<SessionResponse>, Status> {
        let response = SessionResponse {
            deadline: 0,
            key: "".to_string(),
            status: 0,
        };

        Ok(Response::new(response))
    }

    async fn signup( &self, request: Request<SignupRequest>) -> Result<Response<SessionResponse>, Status> {
        let response = SessionResponse {
            deadline: 0,
            key: "".to_string(),
            status: 0,
        };

        Ok(Response::new(response))
    }
    
}

pub async fn start_server(address: String) -> Result<(), Box<dyn std::error::Error>> {
   let addr = address.parse().unwrap();
   let session_server = SessionImplementation::default();

   println!("SessionServer listening on {}", addr);

   Server::builder()
       .add_service(SessionServer::new(session_server))
       .serve(addr)
       .await?;

   Ok(())
}