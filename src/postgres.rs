use std::env;
use lazy_static;
use diesel::{
    r2d2::{Pool, ConnectionManager},
    pg::PgConnection
};

const ERR_NOT_URL: &str = "Postgres url must be set";
const ERR_CONNECT: &str = "Error connecting to postgres cluster";
const ENV_DATABASE_DSN: &str = "DATABASE_URL";
const POOL_SIZE: u32 = 1_u32; // by default: single thread

type PgPool = Pool<ConnectionManager<PgConnection>>;

struct Stream {
   db_connection: PgPool,
}

lazy_static! {
    static ref STREAM: Stream = {
       Stream {
           db_connection: PgPool::builder()
               .max_size(POOL_SIZE)
               .build(ConnectionManager::new(env::var(ENV_DATABASE_DSN).expect(ERR_NOT_URL)))
               .expect(ERR_CONNECT)
        }
    };
}

pub fn open_stream() -> &'static PgPool {
    &STREAM.db_connection
}

pub fn can_connect() {
    open_stream().get().expect(ERR_CONNECT);
}