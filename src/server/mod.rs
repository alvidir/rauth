mod session;
mod dashboard;
mod gateway;

use std::error::Error;
use tonic::{transport::Server, Status, Code};
use crate::proto::{user_proto, app_proto, dashboard_proto};

// Proto generated server traits
use user_proto::session_server::{SessionServer};
use dashboard_proto::dashboard_server::{DashboardServer};
use app_proto::gateway_server::{GatewayServer};

pub fn parse_error(err: Box<dyn Error>) -> Status {
    println!("{:?}", err.to_string());
    let code = Code::from(Code::Unknown);
    Status::new(code, err.to_string())
}

pub async fn start_server(address: String) -> Result<(), Box<dyn Error>> {
    let addr = address.parse().unwrap();
    let session_server = session::SessionImplementation::default();
    let gateway_server = gateway::GatewayImplementation::default();
    let dashboard_server = dashboard::DashboardImplementation::default();

 
    println!("Session service listening on {}", addr);
 
    Server::builder()
        .add_service(SessionServer::new(session_server))
        .add_service(GatewayServer::new(gateway_server))
        .add_service(DashboardServer::new(dashboard_server))
        .serve(addr)
        .await?;
 
    Ok(())
 }