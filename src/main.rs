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

mod server;
mod models;
mod transactions;
mod schema;
mod regex;
mod postgres;
mod mongo;
mod proto;
mod time;
mod token;
mod default;

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

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    initialize();
    let port = env::var(default::ENV_SERVICE_PORT)
        .expect(ERR_NO_PORT);

   let addr = format!("{}:{}", default::SERVER_IP, port);
   server::start_server(addr).await?;
   Ok(())
}