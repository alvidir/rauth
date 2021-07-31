use std::time::{SystemTime, Duration};
use std::thread;
use mongodb::{
    bson::doc,
    sync::{Client, Collection, Database},
};

use std::env;
use crate::constants;

const ERR_NO_DSN: &str = "mongodb dsn must be set";
const ERR_NO_DB_NAME: &str = "mongodb database name must be set";
const ERR_CONNECT: &str = "error connecting to mongodb cluster";

struct Stream {
   db_connection: Database,
}

lazy_static! {
    static ref STREAM: Stream = {
        Stream {
            db_connection: {
                let mongo_dsn = env::var(constants::ENV_MONGO_DSN).expect(ERR_NO_DSN);
                let mongo_db = env::var(constants::ENV_MONGO_DB).expect(ERR_NO_DB_NAME);
                
                let start = SystemTime::now();
                let timeout = Duration::from_secs(constants::CONNECTION_TIMEOUT);
                let sleep = Duration::from_secs(constants::CONNECTION_SLEEP);

                let client = Client::with_uri_str(&mongo_dsn);
                while let Err(err) = &client {
                    warn!("{}: {}", ERR_CONNECT, err);
                    let lapse = SystemTime::now().duration_since(start).unwrap();
                    if  lapse > timeout  {
                        error!("{}: timeout exceeded", ERR_CONNECT);
                        panic!("{}", ERR_CONNECT);
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