#[macro_use]
extern crate dotenv_codegen;
use dotenv;
use std::error::Error;

mod service;
mod model;

mod transactions;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
   dotenv::dotenv().ok();

   let url = format!("127.0.0.1:{}", dotenv!("SERVICE_PORT"));
   let address = String::from(url);
   println!("Server is listening on {}", address);
   service::client::session::start_server(address).await?;

   Ok(())
}