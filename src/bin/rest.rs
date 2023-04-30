#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use actix_web::web::Data;
use actix_web::{middleware, App, HttpServer};
use base64::{engine::general_purpose, Engine as _};
use rauth::{
    session::rest::SessionRestService,
    token::{application::TokenApplication, repository::RedisTokenRepository},
};
use reool::RedisPool;
use std::env;
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tokio::runtime::Handle;

const DEFAULT_ADDR: &str = "127.0.0.1";
const DEFAULT_PORT: &str = "8001";
const DEFAULT_JWT_HEADER: &str = "authorization";
const DEFAULT_TOKEN_TIMEOUT: u64 = 7200;
const DEFAULT_POOL_SIZE: u32 = 10;

const ENV_SERVICE_PORT: &str = "SERVICE_PORT";
const ENV_SERVICE_ADDR: &str = "SERVICE_ADDR";
const ENV_JWT_SECRET: &str = "JWT_SECRET";
const ENV_JWT_PUBLIC: &str = "JWT_PUBLIC";
const ENV_JWT_HEADER: &str = "JWT_HEADER";
const ENV_REDIS_DSN: &str = "REDIS_DSN";
const ENV_TOKEN_TIMEOUT: &str = "TOKEN_TIMEOUT";
const ENV_REDIS_POOL: &str = "REDIS_POOL";
const ENV_TOKEN_ISSUER: &str = "TOKEN_ISSUER";

lazy_static! {
    static ref SERVER_ADDR: String = {
        let netw = env::var(ENV_SERVICE_ADDR).unwrap_or_else(|_| DEFAULT_ADDR.to_string());
        let port = env::var(ENV_SERVICE_PORT).unwrap_or_else(|_| DEFAULT_PORT.to_string());
        format!("{}:{}", netw, port)
    };
    static ref TOKEN_TIMEOUT: u64 = env::var(ENV_TOKEN_TIMEOUT)
        .map(|timeout| timeout.parse().unwrap())
        .unwrap_or(DEFAULT_TOKEN_TIMEOUT);
    static ref JWT_SECRET: Vec<u8> = env::var(ENV_JWT_SECRET)
        .map(|secret| general_purpose::STANDARD.decode(secret).unwrap())
        .expect("jwt secret must be set");
    static ref JWT_PUBLIC: Vec<u8> = env::var(ENV_JWT_PUBLIC)
        .map(|secret| general_purpose::STANDARD.decode(secret).unwrap())
        .expect("jwt public key must be set");
    static ref JWT_HEADER: String =
        env::var(ENV_JWT_HEADER).unwrap_or_else(|_| DEFAULT_JWT_HEADER.to_string());
    static ref RD_POOL: RedisPool = {
        let redis_dsn: String = env::var(ENV_REDIS_DSN).expect("redis url must be set");
        let redis_pool: usize = env::var(ENV_REDIS_POOL)
            .map(|pool_size| pool_size.parse().unwrap())
            .unwrap_or_else(|_| DEFAULT_POOL_SIZE.try_into().unwrap());

        RedisPool::builder()
            .connect_to_node(redis_dsn)
            .desired_pool_size(redis_pool)
            .task_executor(Handle::current())
            .finish_redis_rs()
            .unwrap()
    };
    static ref TOKEN_ISSUER: String = env::var(ENV_TOKEN_ISSUER).expect("token issuer must be set");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    if let Err(err) = dotenv::dotenv() {
        warn!("processing dotenv file {}", err);
    }

    let token_repo = Arc::new(RedisTokenRepository { pool: &RD_POOL });

    let token_app = TokenApplication {
        token_repo: token_repo.clone(),
        timeout: Duration::from_secs(*TOKEN_TIMEOUT),
        token_issuer: &TOKEN_ISSUER,
        private_key: &JWT_SECRET,
        public_key: &JWT_PUBLIC,
    };

    let session_server = Arc::new(SessionRestService {
        token_app,
        jwt_header: &JWT_HEADER,
    });

    info!("server listening on {}", *SERVER_ADDR);
    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(Data::new(session_server.clone()))
            .configure(session_server.router())
    })
    .bind(&*SERVER_ADDR)?
    .run()
    .await
    .unwrap();

    Ok(())
}
