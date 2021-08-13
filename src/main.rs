#[macro_use]
extern crate log;

use oauth::{
    user,
    app,
    session,
    constants::{
        environment,
        settings
    },
    postgres,
    mongo
};

use dotenv;
use std::env;
use std::error::Error;
use tonic::transport::Server;

pub async fn start_server(address: String) -> Result<(), Box<dyn Error>> {
    use user::framework::UserServiceServer;
    use app::framework::AppServiceServer;
    use session::framework::SessionServiceServer;

    let user_server = user::framework::UserServiceImplementation{};
    let app_server = app::framework::AppServiceImplementation{};
    let session_server = session::framework::SessionServiceImplementation{};
 
    let addr = address.parse().unwrap();
    info!("server listening on {}", addr);
 
    Server::builder()
        .add_service(UserServiceServer::new(user_server))
        .add_service(AppServiceServer::new(app_server))
        .add_service(SessionServiceServer::new(session_server))
        .serve(addr)
        .await?;
 
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // configuring logs
    env_logger::init();

    // seting up environment variables
    if let Err(_) = dotenv::dotenv() {
        warn!("no dotenv file has been found");
    }

    postgres::get_connection(); // checking postgres connectivity
    mongo::get_connection("secrets"); // checking mongodb connectivity

    let port = env::var(environment::SERVICE_PORT)
        .expect("service port must be set");

    let addr = format!("{}:{}", settings::SERVER_IP, port);
    start_server(addr).await?;
    Ok(())
}