use mongodb::{
    bson::{doc, Bson},
    sync::{Client, Collection, Database},
};

use std::env;

const ERR_NOT_URL: &str = "Mongodb url must be set";
const ERR_NOT_APP_NAME: &str = "Application name must be set";
const ERR_CONNECT: &str = "Error connecting to mongodb cluster";
const ENV_DATABASE_DSN: &str = "MONGO_DSN";
const ENV_APP_NAME: &str = "APP_NAME";

struct Stream {
   db_connection: Database,
}

lazy_static! {
    static ref STREAM: Stream = {
        Stream {
            db_connection: Client::with_uri_str(&env::var(ENV_DATABASE_DSN).expect(ERR_NOT_URL))
                .expect(ERR_CONNECT)
                .database(&env::var(ENV_APP_NAME).expect(ERR_NOT_APP_NAME)),
        }
    };
}

pub fn open_stream(name: &str) -> Collection {
    // Get a handle to a database.
    STREAM.db_connection.collection(name)
}

pub fn can_connect() {
    Client::with_uri_str(&env::var(ENV_DATABASE_DSN).expect(ERR_NOT_URL))
        .expect(ERR_CONNECT)
        .database(&env::var(ENV_APP_NAME).expect(ERR_NOT_APP_NAME))
        .run_command(doc! {"ping": 1}, None)
        .expect(ERR_CONNECT);
}