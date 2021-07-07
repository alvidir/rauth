use tonic::{Request, Response, Status};

// Import the generated rust code into module
mod proto {
    tonic::include_proto!("app");
}

// Proto generated server traits
use proto::app_service_server::AppService;
pub use proto::app_service_server::AppServiceServer;

// Proto message structs
use proto::{RegisterRequest, RegisterResponse, DeleteRequest};

#[derive(Default)]
pub struct AppServiceImplementation {}

#[tonic::async_trait]
impl AppService for AppServiceImplementation {
    async fn register(&self, request: Request<RegisterRequest>) -> Result<Response<RegisterResponse>, Status> {
        let msg_ref = request.into_inner();
        Err(Status::unimplemented(""))
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();
        Err(Status::unimplemented(""))
    }
}