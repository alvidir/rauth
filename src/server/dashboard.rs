use tonic::{Request, Response, Status};
use crate::transactions::*;
use crate::proto::dashboard_proto;
use super::*;

// Proto generated server traits
use dashboard_proto::dashboard_server::Dashboard;

// Proto message structs
use dashboard_proto::{RegisterAppRequest, DeleteAppRequest};

#[derive(Default)]
pub struct DashboardImplementation {}

#[tonic::async_trait]
impl Dashboard for DashboardImplementation {
    async fn register_app(&self, _request: Request<RegisterAppRequest>) -> Result<Response<()>, Status> {
        //let msg_ref = request.into_inner();
        Err(parse_error("err".into()))
    }

    async fn delete_app(&self, _request: Request<DeleteAppRequest>) -> Result<Response<()>, Status> {
        //let msg_ref = request.into_inner();
        Err(parse_error("err".into()))
    }
}