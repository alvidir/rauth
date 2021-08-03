use std::time::{SystemTime, Duration};
use std::thread;
use std::env;
use lazy_static;
use diesel::{
    r2d2::{Pool, ConnectionManager},
    pg::PgConnection
};

use crate::constants::{environment, settings, errors};

type PgPool = Pool<ConnectionManager<PgConnection>>;

struct Stream {
   db_connection: PgPool,
}

lazy_static! {
    static ref STREAM: Stream = {
       Stream {
            db_connection: {
                let postgres_url = env::var(environment::POSTGRES_DSN).expect("postgres url must be set");
                let start = SystemTime::now();
                let timeout = Duration::from_secs(settings::CONNECTION_TIMEOUT);
                let sleep = Duration::from_secs(settings::CONNECTION_SLEEP);

                let mut pool = PgPool::builder().max_size(settings::POOL_SIZE).build(ConnectionManager::new(&postgres_url));
                while let Err(err) = &pool {
                    warn!("{}: {}", errors::CANNOT_CONNECT, err);
                    let lapse = SystemTime::now().duration_since(start).unwrap();
                    if  lapse > timeout  {
                        error!("{}: timeout exceeded", errors::CANNOT_CONNECT);
                        panic!("{}", errors::CANNOT_CONNECT);
                    }

                    thread::sleep(sleep);
                    pool = PgPool::builder().max_size(settings::POOL_SIZE).build(ConnectionManager::new(&postgres_url));
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