#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;
#[macro_use(/*bson,*/ doc)]
extern crate bson;

use dotenv;
use std::env;
use std::error::Error;

use tonic::{transport::Server, Status, Code};

mod schema;
mod regex;
mod postgres;
mod mongo;
mod time;
mod constants;
mod metadata;
mod user;
mod session;
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
    use user::framework::UserServiceServer;
    use app::framework::AppServiceServer;

    let addr = address.parse().unwrap();
    let user_server = user::framework::UserServiceImplementation::default();
    let app_server = app::framework::AppServiceImplementation::default();
 
    println!("Server listening on {}", addr);
 
    Server::builder()
        .add_service(UserServiceServer::new(user_server))
        .add_service(AppServiceServer::new(app_server))
        .serve(addr)
        .await?;
 
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    initialize();
    let port = env::var(constants::ENV_SERVICE_PORT)
        .expect(ERR_NO_PORT);

    let addr = format!("{}:{}", constants::SERVER_IP, port);
    start_server(addr).await?;
    Ok(())
}