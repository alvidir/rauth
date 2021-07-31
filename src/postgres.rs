use std::time::{SystemTime, Duration};
use std::thread;
use std::env;
use lazy_static;
use diesel::{
    r2d2::{Pool, ConnectionManager},
    pg::PgConnection
};

use crate::constants;

const ERR_NOT_URL: &str = "postgres url must be set";
const ERR_CONNECT: &str = "error connecting to postgres cluster";
const POOL_SIZE: u32 = 1_u32; // by constants: single thread

type PgPool = Pool<ConnectionManager<PgConnection>>;

struct Stream {
   db_connection: PgPool,
}

lazy_static! {
    static ref STREAM: Stream = {
       Stream {
            db_connection: {
                let postgres_url = env::var(constants::ENV_POSTGRES_DSN).expect(ERR_NOT_URL);
                let start = SystemTime::now();
                let timeout = Duration::from_secs(constants::CONNECTION_TIMEOUT);
                let sleep = Duration::from_secs(constants::CONNECTION_SLEEP);

                let mut pool = PgPool::builder().max_size(POOL_SIZE).build(ConnectionManager::new(&postgres_url));
                while let Err(err) = &pool {
                    warn!("{}: {}", ERR_CONNECT, err);
                    let lapse = SystemTime::now().duration_since(start).unwrap();
                    if  lapse > timeout  {
                        error!("{}: timeout exceeded", ERR_CONNECT);
                        panic!("{}", ERR_CONNECT);
                    }

                    thread::sleep(sleep);
                    pool = PgPool::builder().max_size(POOL_SIZE).build(ConnectionManager::new(&postgres_url));
                }

                info!("connection with postgres cluster established");
                pool.unwrap()
            },
        }
    };
}

pub fn get_connection() -> &'static PgPool {
    &STREAM.db_connection
}