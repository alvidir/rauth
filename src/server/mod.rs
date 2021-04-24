mod session;
mod profile;
mod registry;

use std::error::Error;
use tonic::{transport::Server, Status, Code};
use crate::proto::{user_proto, app_proto, client_proto};

// Proto generated server traits
use user_proto::session_server::{SessionServer};
use app_proto::registry_server::{RegistryServer};
use client_proto::profile_server::{ProfileServer};

pub fn parse_error(err: Box<dyn Error>) -> Status {
    println!("{:?}", err.to_string());
    let code = Code::from(Code::Unknown);
    Status::new(code, err.to_string())
}

pub async fn start_server(address: String) -> Result<(), Box<dyn Error>> {
    let addr = address.parse().unwrap();
    let session_server = session::SessionImplementation::default();
    let profile_server = profile::ProfileImplementation::default();
    let registry_server = registry::RegistryImplementation::default();
 
    println!("Server listening on {}", addr);
 
    Server::builder()
        .add_service(SessionServer::new(session_server))
        .add_service(RegistryServer::new(registry_server))
        .add_service(ProfileServer::new(profile_server))
        .serve(addr)
        .await?;
 
    Ok(())
 }