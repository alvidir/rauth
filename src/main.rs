#[macro_use]
extern crate diesel;

use dotenv;
use std::env;
use std::error::Error;
use diesel::prelude::*;
use diesel::pg::PgConnection;

mod service;
mod model;
mod transactions;
mod schema;

const envServicePort: &str = "SERVICE_PORT";
const envDatabaseUrl: &str = "DATABASE_URL";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
   dotenv::dotenv().ok();

   let database_url = env::var(envDatabaseUrl).expect("Postgres url must be set.");
   let conn = PgConnection::establish(&database_url).unwrap();

   let port = env::var(envServicePort).expect("Service port must be set.");
   let addr = format!("127.0.0.1:{}", port);
   service::client::session::start_server(addr).await?;

   Ok(())
}