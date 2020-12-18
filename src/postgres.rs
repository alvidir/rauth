use std::env;
use diesel::prelude::*;
use diesel::pg::PgConnection;

const ERR_NOT_URL: &str = "Postgres url must be set";
const ERR_CONNECT: &str = "Error connecting to";
const INFO_STREAM_SETUP: &str = "Setting up postgres stream to:";
const ENV_DATABASE_URL: &str = "DATABASE_URL";

pub static mut STREAM: Option<PgConnection> = None;

pub fn open_stream() -> &'static PgConnection {
    let postgres_conn: &Option<PgConnection>;
    unsafe {
        postgres_conn = &STREAM;
    }
   
    match &postgres_conn {
        Some(conn) => {
            return &conn
        }
        
        None => {
            let database_url = env::var(ENV_DATABASE_URL)
                .expect(ERR_NOT_URL);

            let conn = PgConnection::establish(&database_url)
                .expect(&format!("{} {}", ERR_CONNECT, database_url));

            unsafe {
                STREAM = Some(conn);
            }

            return open_stream();
        }
    }
}