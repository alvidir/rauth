use tonic::{transport::Server, Request, Response, Status};

// Import the generated rust code into module
pub mod session_proto {
   tonic::include_proto!("session");
}

// Proto generated server traits
use session_proto::session_server::{Session, SessionServer};

// Proto message structs
use session_proto::{LoginRequest, GoogleLoginRequest, LogoutRequest, SignupRequest, SessionResponse };

// For the server listening address
use crate::ServerOptions;

#[derive(Default)]
pub struct SessionImplementation {}

#[tonic::async_trait]
impl Session for SessionImplementation {
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<SessionResponse>, Status> {
        return;
    }

    async fn google_login(&self, request: Request<GoogleLoginRequest>) -> Result<Response<SessionResponse>, Status> {
        return;
    }

    async fn logout( &self, request: Request<LogoutRequest>) -> Result<Response<SessionResponse>, Status> {
        return;
    }

    async fn signup( &self, request: Request<SignupRequest>) -> Result<Response<SessionResponse>, Status> {
        return;
    }
    
}

pub async fn start_server(opts: ServerOptions) -> Result<(), Box<dyn std::error::Error>> {
   let addr = opts.server_listen_addr.parse().unwrap();
   let session_server = SessionImplementation::default();

   println!("SessionServer listening on {}", addr);

   Server::builder()
       .add_service(SessionServer::new(session_server))
       .serve(addr)
       .await?;

   Ok(())
}