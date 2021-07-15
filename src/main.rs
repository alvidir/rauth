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

use tonic::{transport::Server, Status, Code};

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

use metadata::framework::PostgresMetadataRepository;
use secret::framework::MongoSecretRepository;
use user::framework::{PostgresUserRepository, EmailSenderImplementation};


lazy_static! {
    static ref META_REPO: PostgresMetadataRepository = PostgresMetadataRepository{};
    static ref SECRET_REPO: MongoSecretRepository = MongoSecretRepository{};
    static ref USER_REPO: PostgresUserRepository = PostgresUserRepository::new(&META_REPO, &SECRET_REPO);
    static ref EMAIL_SENDER: EmailSenderImplementation = EmailSenderImplementation{};
}

pub fn parse_error(err: Box<dyn Error>) -> Status {
    println!("{:?}", err.to_string());
    let code = Code::from(Code::Unknown);
    Status::new(code, err.to_string())
}

pub async fn start_server(address: String,
                          user_repo: &'static user::framework::PostgresUserRepository,
                          email_sender: &'static user::framework::EmailSenderImplementation)
                          -> Result<(), Box<dyn Error>> {
    use user::framework::UserServiceServer;
    use app::framework::AppServiceServer;

    let addr = address.parse().unwrap();
    let user_server = user::framework::UserServiceImplementation::new(user_repo, email_sender);
    let app_server = app::framework::AppServiceImplementation::default();
    let session_server = session::framework::SessionServiceImplementation::default();
 
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
    if let Err(_) = dotenv::dotenv() {
        // seting up environment variables (if there is no .env: must NOT fail)
        println!("no dotenv file has been found");
    }

    postgres::must_connect(); // checking postgres connectivity
    mongo::must_connect(); // checking mongodb connectivity

    

    let port = env::var(constants::ENV_SERVICE_PORT)
        .expect("service port must be set");

    let addr = format!("{}:{}", constants::SERVER_IP, port);
    start_server(addr, &USER_REPO, &EMAIL_SENDER).await?;
    Ok(())
}