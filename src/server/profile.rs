use crate::transactions::{ticket, resolve};
use tonic::{Request, Response, Status};
use crate::proto::client_proto;
use super::*;

// Proto generated server traits
use client_proto::profile_server::Profile;

// Proto message structs
use client_proto::{TicketRequest, TicketResponse, ResolveRequest};

#[derive(Default)]
pub struct ProfileImplementation {}

#[tonic::async_trait]
impl Profile for ProfileImplementation {
    async fn ticket(&self, request: Request<TicketRequest>) -> Result<Response<TicketResponse>, Status> {
        let msg_ref = request.into_inner();
        let tx_register = ticket::TxTicket::new(
            msg_ref.kind,
            &msg_ref.ident
        );
        
        match tx_register.execute() {
            Ok(resp) => Ok(Response::new(resp)),
            Err(err) => Err(parse_error(err))
        }
    }

    async fn resolve(&self, request: Request<ResolveRequest>) -> Result<Response<()>, Status> {
        let msg_ref = request.into_inner();
        let tx_delete = resolve::TxResolve::new(
            &msg_ref.id,
            &msg_ref.data
        );
        
        match tx_delete.execute() {
            Ok(_) => Ok(Response::new(())),
            Err(err) => Err(parse_error(err))
        }
    }
}