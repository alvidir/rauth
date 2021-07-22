#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;
#[macro_use(/*bson,*/ doc)]
extern crate bson;

use dotenv;
use std::env;
use std::error::Error;

use tonic::transport::Server;

mod schema;
mod regex;
mod postgres;
mod mongo;
mod smtp;
mod time;
mod constants;
mod metadata;
mod user;
mod session;
mod app;
mod secret;
mod security;
mod directory;

use secret::framework::MongoSecretRepository;
use metadata::framework::PostgresMetadataRepository;
use user::framework::PostgresUserRepository;
use app::framework::PostgresAppRepository;
use directory::framework::MongoDirectoryRepository;
use session::framework::InMemorySessionRepository;

lazy_static! {
    static ref META_REPO: PostgresMetadataRepository = PostgresMetadataRepository{};
    static ref SECRET_REPO: MongoSecretRepository = MongoSecretRepository{};
    static ref USER_REPO: PostgresUserRepository = PostgresUserRepository::new(&SECRET_REPO, &META_REPO);
    static ref APP_REPO: PostgresAppRepository = PostgresAppRepository::new(&SECRET_REPO, &META_REPO);

    static ref DIR_REPO: MongoDirectoryRepository = MongoDirectoryRepository{};
    static ref SESSION_REPO: InMemorySessionRepository = InMemorySessionRepository::new();
}

pub async fn start_server(address: String) -> Result<(), Box<dyn Error>> {
    use user::framework::UserServiceServer;
    use app::framework::AppServiceServer;
    use session::framework::SessionServiceServer;

    let user_server = user::framework::UserServiceImplementation::new(&USER_REPO, &SESSION_REPO, &META_REPO);
    let app_server = app::framework::AppServiceImplementation::new(&APP_REPO, &SECRET_REPO, &META_REPO);
    let session_server = session::framework::SessionServiceImplementation::new(&SESSION_REPO, &USER_REPO, &APP_REPO, &DIR_REPO);
 
    let addr = address.parse().unwrap();
    println!("server listening on {}", addr);
 
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
    if let Err(_) = dotenv::dotenv() {
        // seting up environment variables (if there is no .env: must NOT fail)
        println!("no dotenv file has been found");
    }

    postgres::must_connect(); // checking postgres connectivity
    mongo::must_connect(); // checking mongodb connectivity

    let port = env::var(constants::ENV_SERVICE_PORT)
        .expect("service port must be set");

    let addr = format!("{}:{}", constants::SERVER_IP, port);
    start_server(addr).await?;
    Ok(())
}