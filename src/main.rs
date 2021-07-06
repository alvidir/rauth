#[macro_use]
extern crate diesel;
#[macro_use]
extern crate custom_derive;
#[macro_use]
extern crate enum_derive;
#[macro_use]
extern crate lazy_static;
#[macro_use(/*bson,*/ doc)]
extern crate bson;

use dotenv;
use std::env;
use std::error::Error;

use tonic::{transport::Server, Status, Code};
use crate::proto::{user_proto, app_proto, client_proto};

// Proto generated server traits
use user_proto::session_server::{SessionServer};
use app_proto::registry_server::{RegistryServer};
use client_proto::profile_server::{ProfileServer};

mod schema;
mod regex;
mod postgres;
mod mongo;
mod time;
mod token;
mod default;
mod enums;
mod client;
mod user;
mod app;

const ERR_NO_PORT: &str = "Service port must be set";

use std::sync::Once;

static INIT: Once = Once::new();

pub fn initialize() {
    INIT.call_once(|| {
        if let Err(_) = dotenv::dotenv() {
            // seting up environment variables (if there is no .env: must NOT fail)
            println!("No dotenv file has been found.");
        }

        postgres::must_connect(); // checking postgres connectivity
        mongo::must_connect(); // checking mongodb connectivity
    });
}

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    initialize();
    let port = env::var(default::ENV_SERVICE_PORT)
        .expect(ERR_NO_PORT);

   let addr = format!("{}:{}", default::SERVER_IP, port);
   server::start_server(addr).await?;
   Ok(())
}