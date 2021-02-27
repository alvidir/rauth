use crate::transactions::{register, delete_app};
use tonic::{Request, Response, Status};
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
    async fn register(&self, request: Request<RegisterRequest>) -> Result<Response<RegisterResponse>, Status> {
        let msg_ref = request.into_inner();
        let tx_register = register::TxRegister::new(
            &msg_ref.name,
            &msg_ref.url,
            &msg_ref.descr,
            &msg_ref.public,
            &msg_ref.firm,
        );
        
        match tx_register.execute() {
            Ok(resp) => Ok(Response::new(resp)),
            Err(err) => Err(parse_error(err))
        }
    }

    async fn delete(&self, request: Request<DeleteRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();
        let tx_delete = delete_app::TxDelete::new(
            &msg_ref.label,
            &msg_ref.dust,
            &msg_ref.firm,
        );
        
        match tx_delete.execute() {
            Ok(_) => Ok(Response::new(())),
            Err(err) => Err(parse_error(err))
        }
    }
}