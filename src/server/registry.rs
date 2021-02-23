use tonic::{Request, Response, Status};
use crate::transactions::*;
use crate::proto::app_proto;
use super::*;

// Proto generated server traits
use app_proto::registry_server::Registry;

// Proto message structs
use app_proto::{RegisterRequest, RegisterResponse, DeleteRequest};

#[derive(Default)]
pub struct RegistryImplementation {}

#[tonic::async_trait]
impl Registry for RegistryImplementation {
    async fn register(&self, _request: Request<RegisterRequest>) -> Result<Response<RegisterResponse>, Status> {
        //let msg_ref = request.into_inner();
        Err(parse_error("err".into()))
    }

    async fn delete(&self, _request: Request<DeleteRequest>) -> Result<Response<()>, Status> {
        //let msg_ref = request.into_inner();
        Err(parse_error("err".into()))
    }
}