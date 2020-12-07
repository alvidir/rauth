#[macro_use]
extern crate dotenv_codegen;
use dotenv;
use std::error::Error;

mod service;
mod model;
mod transactions;

const envServicePort: &str = "SERVICE_PORT";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
   dotenv::dotenv().ok();

   let addr = format!("127.0.0.1:{}", dotenv!(envServicePort));
   service::client::session::start_server(addr).await?;

   Ok(())
}