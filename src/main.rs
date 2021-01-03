#[macro_use]
extern crate diesel;

use dotenv;
use std::env;
use std::error::Error;

mod services;
mod models;
mod transactions;
mod schema;
mod regex;
mod postgres;
mod dummy;

const ERR_NO_PORT: &str = "Service port must be set";
const ENV_SERVICE_PORT: &str = "SERVICE_PORT";
const DEFAULT_IP: &str = "127.0.0.1";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
   dotenv::dotenv().ok();
   postgres::open_stream(); // checking connectivity

   let port = env::var(ENV_SERVICE_PORT)
      .expect(ERR_NO_PORT);

   dummy::dummy_setup()?;
   let addr = format!("{}:{}", DEFAULT_IP, port);
   services::session::start_server(addr).await?;

   Ok(())
}