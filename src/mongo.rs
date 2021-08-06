use mongodb::{
    bson::doc,
    sync::{Client, Collection, Database},
};

use std::env;
use crate::constants::{environment, errors};

struct Stream {
   db_connection: Database,
}

lazy_static! {
    static ref STREAM: Stream = {
        Stream {
            db_connection: {
                let mongo_dsn = env::var(environment::MONGO_DSN).expect("mongodb dsn must be set");
                let mongo_db = env::var(environment::MONGO_DB).expect("mongodb database name must be set");

                match Client::with_uri_str(&mongo_dsn) {
                    Ok(client) => {
                        info!("connection with mongodb cluster established");
                        client.database(&mongo_db)
                    },
                    Err(err) => {
                        error!("{}", err);
                        panic!("{}", errors::CANNOT_CONNECT);
                    } 
                }               
            },
        }
    };
}

pub fn get_connection(name: &str) -> Collection {
    // Get a handle to a database.
    STREAM.db_connection.collection(name)
}