use std::time::{SystemTime, Duration};
use std::thread;
use mongodb::{
    bson::doc,
    sync::{Client, Collection, Database},
};

use std::env;
use crate::constants::{environment, settings, errors};

struct Stream {
   db_connection: Database,
}

lazy_static! {
    static ref STREAM: Stream = {
        Stream {
            db_connection: {
                let mongo_dsn = env::var(environment::MONGO_DSN).expect("mongodb dsn must be set");
                let mongo_db = env::var(environment::MONGO_DB).expect("mongodb database name must be set");
                
                let start = SystemTime::now();
                let timeout = Duration::from_secs(settings::CONNECTION_TIMEOUT);
                let sleep = Duration::from_secs(settings::CONNECTION_SLEEP);

                let client = Client::with_uri_str(&mongo_dsn);
                while let Err(err) = &client {
                    warn!("{}: {}", errors::CANNOT_CONNECT, err);
                    let lapse = SystemTime::now().duration_since(start).unwrap();
                    if  lapse > timeout  {
                        error!("{}: timeout exceeded", errors::CANNOT_CONNECT);
                        panic!("{}", errors::CANNOT_CONNECT);
                    }

                    thread::sleep(sleep);
                }

                info!("connection with mongodb cluster established");
                client.unwrap().database(&mongo_db)
            },
        }
    };
}

pub fn get_connection(name: &str) -> Collection {
    // Get a handle to a database.
    STREAM.db_connection.collection(name)
}