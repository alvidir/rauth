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
                match PgPool::builder().max_size(settings::POOL_SIZE).build(ConnectionManager::new(&postgres_url)) {
                    Ok(pool) => {
                        info!("connection with postgres cluster established");
                        pool
                    },
                    Err(err) => {
                        error!("{}", err);
                        panic!("{}", errors::CANNOT_CONNECT);
                    }
                }
            },
        }
    };
}

pub fn get_connection() -> &'static PgPool {
    &STREAM.db_connection
}