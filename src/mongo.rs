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
            db_connection: Client::with_uri_str(&env::var(constants::ENV_MONGO_DSN).expect(ERR_NO_DSN))
                .expect(ERR_CONNECT)
                .database(&env::var(constants::ENV_MONGO_DB).expect(ERR_NO_DB_NAME)),
        }
    };
}

pub fn open_stream(name: &str) -> Collection {
    // Get a handle to a database.
    STREAM.db_connection.collection(name)
}

pub fn must_connect() {
    Client::with_uri_str(&env::var(constants::ENV_MONGO_DSN).expect(ERR_NO_DSN))
        .expect(ERR_CONNECT)
        .database(&env::var(constants::ENV_MONGO_DB).expect(ERR_NO_DB_NAME))
        .run_command(doc! {"ping": 1}, None)
        .expect(ERR_CONNECT);
}