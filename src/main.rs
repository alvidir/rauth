#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use rauth::secret::repository::PostgresSecretRepository;
use dotenv;
use redis;
use std::env;
use std::error::Error;
use std::sync::Arc;
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
const ENV_JWT_HEADER: &str = "JWT_HEADER";
const ENV_REDIS_DSN: &str = "REDIS_DSN";

type PgPool = Pool<ConnectionManager<PgConnection>>;

lazy_static! {
    static ref JWT_SECRET: Vec<u8> = base64::decode(env::var(ENV_JWT_SECRET).unwrap()).unwrap();
    static ref JWT_PUBLIC: Vec<u8> = base64::decode(env::var(ENV_JWT_PUBLIC).unwrap()).unwrap();
    static ref JWT_HEADER: String = env::var(ENV_JWT_HEADER).unwrap();

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
    let metadata_repo = Arc::new(PostgresMetadataRepository{
        pool: &PG_POOL,
    });

    let secret_repo = Arc::new(PostgresSecretRepository {
        pool: &PG_POOL,
        metadata_repo: metadata_repo.clone(),
    });

    let user_repo = Arc::new(PostgresUserRepository{
        pool: &PG_POOL,
        metadata_repo: metadata_repo.clone(),
    });

    let session_repo = Arc::new(RedisSessionRepository{
        conn: get_redis_conn(),
        jwt_secret: &JWT_SECRET,
        jwt_public: &JWT_PUBLIC,
    });

    let user_app = UserApplication{
        user_repo: user_repo.clone(),
        session_repo: session_repo.clone(),
        secret_repo: secret_repo.clone(),
    };

    let sess_app = SessionApplication{
        session_repo: session_repo.clone(),
        user_repo: user_repo.clone(),
        secret_repo: secret_repo.clone(),
    };

    let user_server = UserServiceImplementation{
        user_app: user_app,
        jwt_secret: &JWT_SECRET,
        jwt_public: &JWT_PUBLIC,
        jwt_header: &JWT_HEADER,
    };

    let sess_server = SessionServiceImplementation {
        sess_app: sess_app,
        jwt_secret: &JWT_SECRET,
        jwt_public: &JWT_PUBLIC,
        jwt_header: &JWT_HEADER,
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