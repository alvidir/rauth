use mongodb::{
    bson::doc,
    sync::{Client, Collection},
};

use std::env;
use crate::constants::{environment, errors};

struct Conn {
   client: Client,
   db_name: String,
}

lazy_static! {
    static ref CONN: Conn = {
        Conn {
            client: {
                let mongo_dsn = env::var(environment::MONGO_DSN).expect("mongodb dsn must be set");
                match Client::with_uri_str(&mongo_dsn) {
                    Ok(client) => {
                        info!("connection with mongodb cluster established");
                        client
                    },
                    Err(err) => {
                        error!("{}", err);
                        panic!("{}", errors::CANNOT_CONNECT);
                    } 
                }               
            },
            
            db_name: env::var(environment::MONGO_DB).expect("mongodb database name must be set"),
        }
    };
}

pub fn get_connection(name: &str) -> Collection {
    // Get a handle to a database.
    CONN.client.clone().database(&CONN.db_name).collection(name)
}