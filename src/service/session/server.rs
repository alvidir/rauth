use tonic::{transport::Server, Request, Response, Status, Code};
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

impl SessionImplementation {
    fn signup_response(&self, result: &std::result::Result<std::boxed::Box<dyn std::any::Any>, std::string::String>) -> Result<Response<SessionResponse>, Status> {
        match result {
            Err(err) => {
                Err(Status::new(Code::Aborted, err))
            }

            Ok(any) => {
                Ok(Response::new(
                    SessionResponse {
                        deadline: 0,
                        cookie: "".to_string(),
                        status: 0,
                    }
                ))
            }
        }
    }
}

#[tonic::async_trait]
impl Session for SessionImplementation {
    async fn signup(&self, request: Request<SignupRequest>) -> Result<Response<SessionResponse>, Status> {
        let msg_ref = request.into_inner();
        let mut tx_signup = TxFactory::new_tx_signup(
            msg_ref.name, 
            msg_ref.addr, 
            msg_ref.pwd,
        );
        
        tx_signup.execute();
        match tx_signup.result() {
            None => {
                let status = Status::new(Code::Internal, "");
                Err(status)
            }

            Some(res) => {
                self.signup_response(res)
            }
        }
    }
    
    async fn login(&self, request: Request<LoginRequest>) -> Result<Response<SessionResponse>, Status> {
        let msg_ref = request.into_inner();
        let mut tx_login = TxFactory::new_tx_login(
            msg_ref.cookie,
            msg_ref.name,
            msg_ref.addr,
            msg_ref.pwd,
        );
        
        tx_login.execute();

        let response = SessionResponse {
            deadline: 0,
            cookie: "".to_string(),
            status: 0,
        };

        Ok(Response::new(response))
    }

    async fn google_login(&self, request: Request<GoogleLoginRequest>) -> Result<Response<SessionResponse>, Status> {
        let response = SessionResponse {
            deadline: 0,
            cookie: "".to_string(),
            status: 0,
        };

        Ok(Response::new(response))
    }

    async fn logout( &self, request: Request<LogoutRequest>) -> Result<Response<SessionResponse>, Status> {
        let response = SessionResponse {
            deadline: 0,
            cookie: "".to_string(),
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