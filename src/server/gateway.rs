use tonic::{Request, Response, Status};
use crate::transactions::*;
use crate::proto::app_proto;
use super::*;

// Proto generated server traits
use app_proto::gateway_server::Gateway;

// Proto message structs
use app_proto::{OpenRequest, CloseRequest};

#[derive(Default)]
pub struct GatewayImplementation {}

#[tonic::async_trait]
impl Gateway for GatewayImplementation {
    async fn open(&self, _request: Request<OpenRequest>) -> Result<Response<()>, Status> {
        //let msg_ref = request.into_inner();
        Err(parse_error("err".into()))
    }

    async fn close(&self, _request: Request<CloseRequest>) -> Result<Response<()>, Status> {
        //let msg_ref = request.into_inner();
        Err(parse_error("err".into()))
    }
}