#[macro_use]
extern crate diesel;

use dotenv;
use std::env;
use std::error::Error;
use diesel::prelude::*;
use diesel::pg::PgConnection;

mod service;
mod model;
mod transaction;
mod schema;

const ENV_SERVICE_PORT: &str = "SERVICE_PORT";
const ENV_DATABASE_URL: &str = "DATABASE_URL";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
   dotenv::dotenv().ok();

   let database_url = env::var(ENV_DATABASE_URL).expect("Postgres url must be set.");
   PgConnection::establish(&database_url)?; // checking connectivity

   let port = env::var(ENV_SERVICE_PORT).expect("Service port must be set.");
   let addr = format!("127.0.0.1:{}", port);
   service::session::start_server(addr).await?;

   Ok(())
}