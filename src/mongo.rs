use std::error::Error;
use mongodb::{
    bson::doc,
    sync::{Client, Collection, Database},
};

use std::env;
use crate::default;

const ERR_NO_DSN: &str = "Mongodb dsn must be set";
const ERR_NO_DB_NAME: &str = "Mongodb database name must be set";
const ERR_CONNECT: &str = "Error connecting to mongodb cluster";

struct Stream {
   db_connection: Database,
}

lazy_static! {
    static ref STREAM: Stream = {
        Stream {
            db_connection: Client::with_uri_str(&env::var(default::ENV_MONGO_DSN).expect(ERR_NO_DSN))
                .expect(ERR_CONNECT)
                .database(&env::var(default::ENV_MONGO_DB).expect(ERR_NO_DB_NAME)),
        }
    };
}

pub fn open_stream(name: &str) -> Collection {
    // Get a handle to a database.
    STREAM.db_connection.collection(name)
}

pub fn must_connect() {
    Client::with_uri_str(&env::var(default::ENV_MONGO_DSN).expect(ERR_NO_DSN))
        .expect(ERR_CONNECT)
        .database(&env::var(default::ENV_MONGO_DB).expect(ERR_NO_DB_NAME))
        .run_command(doc! {"ping": 1}, None)
        .expect(ERR_CONNECT);
}

pub fn get_collection_name() -> Result<String, Box<dyn Error>> {
    match env::var(default::ENV_MONGO_COLL) {
        Err(err) => Err(err.into()),
        Ok(coll) => Ok(coll),
    }
}