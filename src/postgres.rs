use std::time::{SystemTime, Duration};
use std::thread;
use std::env;
use lazy_static;
use diesel::{
    r2d2::{Pool, ConnectionManager},
    pg::PgConnection
};

use crate::default;

const ERR_NOT_URL: &str = "Postgres url must be set";
const ERR_CONNECT: &str = "Error connecting to postgres cluster";
const POOL_SIZE: u32 = 1_u32; // by default: single thread

type PgPool = Pool<ConnectionManager<PgConnection>>;

struct Stream {
   db_connection: PgPool,
}

lazy_static! {
    static ref STREAM: Stream = {
       Stream {
            db_connection: {
                let postgres_url = env::var(default::ENV_POSTGRES_DSN).expect(ERR_NOT_URL);
                let start = SystemTime::now();
                let timeout = Duration::from_secs(default::CONNECTION_TIMEOUT);
                let sleep = Duration::from_secs(default::CONNECTION_SLEEP);

                let mut pool = PgPool::builder().max_size(POOL_SIZE).build(ConnectionManager::new(&postgres_url));
                while let Err(err) = pool {
                    println!("{}: {}", ERR_CONNECT, err);
                    let lapse = SystemTime::now().duration_since(start).unwrap();
                    if  lapse < timeout  {
                        thread::sleep(sleep);
                        pool = PgPool::builder().max_size(POOL_SIZE).build(ConnectionManager::new(&postgres_url));
                    } else {
                        panic!(ERR_CONNECT);
                    }
                }

                pool.unwrap()
            },
        }
    };
}

pub fn open_stream() -> &'static PgPool {
    &STREAM.db_connection
}

pub fn must_connect() {
    open_stream().get().expect(ERR_CONNECT);
    //let start = SystemTime::now();
    //let timeout = Duration::from_secs(default::CONNECTION_TIMEOUT);
    //let sleep = Duration::from_secs(default::CONNECTION_SLEEP);
    //
    //while let Err(err) = open_stream().get() {
    //    println!("{}: {}", ERR_CONNECT, err);
    //    let lapse = start.duration_since(SystemTime::now()).unwrap();
    //    if  lapse > timeout  {
    //        panic!(ERR_CONNECT);
    //    } else {
    //        thread::sleep(sleep);
    //    }
    //}
}