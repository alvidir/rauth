#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use dotenv;
use redis;
use std::env;
use std::error::Error;
use std::sync::{Arc, Mutex};
use tonic::transport::Server;
use diesel::{
    r2d2::{Pool, ConnectionManager},
    pg::PgConnection
};

use rauth::metadata::{
    repository::PostgresMetadataRepository,
};

use rauth::user::{
    grpc::{UserServiceServer, UserServiceImplementation},
    repository::PostgresUserRepository,
    application::UserApplication,
};

use rauth::session::{
    grpc::{SessionServiceServer, SessionServiceImplementation},
    repository::RedisSessionRepository,
    application::SessionApplication,
};

const ENV_SERVICE_PORT: &str = "SERVICE_PORT";
const ENV_POSTGRES_DSN: &str = "DATABASE_URL";
const ENV_JWT_SECRET: &str = "JWT_SECRET";
const ENV_JWT_PUBLIC: &str = "JWT_PUBLIC";
const ENV_REDIS_DSN: &str = "REDIS_DSN";

type PgPool = Pool<ConnectionManager<PgConnection>>;

lazy_static! {
    static ref PG_POOL: PgPool = {
        let postgres_dsn = env::var(ENV_POSTGRES_DSN).expect("postgres url must be set");
        match PgPool::builder().max_size(1).build(ConnectionManager::new(&postgres_dsn)) {
            Ok(pool) => {
                info!("connection with postgres cluster established");
                pool
            },
            Err(err) => {
                error!("{}", err);
                panic!("cannot establish connection with postgres cluster");
            }
        }
    };
}

fn get_redis_conn() -> redis::Connection {
    let redis_dsn = env::var(ENV_REDIS_DSN).expect("redis url must be set");
    match redis::Client::open(redis_dsn).unwrap().get_connection() {
        Ok(conn) => {
            info!("connection with redis cluster established");
            conn
        },
        Err(err) => {
            error!("{}", err);
            panic!("cannot establish connection with redis cluster");
        }
    }
}

pub async fn start_server(address: String) -> Result<(), Box<dyn Error>> {
    let metadata_repo = PostgresMetadataRepository{
        pool: &PG_POOL,
    };

    let user_repo = Arc::new(PostgresUserRepository{
        pool: &PG_POOL,
        metadata_repo: metadata_repo,
    });

    let jwt_secret = base64::decode(env::var(ENV_JWT_SECRET)?)?;
    let jwt_public = base64::decode(env::var(ENV_JWT_PUBLIC)?)?;

    let sess_repo = Arc::new(RedisSessionRepository{
        conn: get_redis_conn(),
        jwt_secret: jwt_secret.clone(),
        jwt_public: jwt_public.clone(),
    });

    let user_app = UserApplication{
        user_repo: user_repo.clone(),
        sess_repo: sess_repo.clone(),
    };

    let sess_app = SessionApplication{
        sess_repo: sess_repo.clone(),
        user_repo: user_repo.clone(),
    };

    let user_server = UserServiceImplementation{
        user_app: user_app,
        jwt_secret: jwt_secret.clone(),
        jwt_public: jwt_public.clone(),
    };

    let sess_server = SessionServiceImplementation {
        sess_app: sess_app,
        jwt_secret: jwt_secret.clone(),
        jwt_public: jwt_public.clone(),
    };
 
    let addr = address.parse().unwrap();
    info!("server listening on {}", addr);
 
    Server::builder()
        .add_service(UserServiceServer::new(user_server))
        .add_service(SessionServiceServer::new(sess_server))
        .serve(addr)
        .await?;
 
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // configuring logs
    env_logger::init();

    // seting up environment variables
    if let Err(_) = dotenv::dotenv() {
        warn!("no dotenv file has been found");
    }

    let port = env::var(ENV_SERVICE_PORT)?;
    let addr = format!("127.0.0.1:{}", port);
    start_server(addr).await
}