use async_once::AsyncOnce;
use base64::{engine::general_purpose, Engine as _};
use deadpool_lapin::{Config, Pool, Runtime};
use lapin::{options, types::FieldTable, ExchangeKind};
use lazy_static::lazy_static;
use reool::RedisPool;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
use tokio::runtime::Handle;

const DEFAULT_ADDR: &str = "127.0.0.1";
const DEFAULT_PORT: &str = "8000";
const DEFAULT_TEMPLATES_PATH: &str = "/etc/rauth/smtp/templates/*.html";
const DEFAULT_JWT_HEADER: &str = "authorization";
const DEFAULT_TOTP_HEADER: &str = "x-totp-secret";
const DEFAULT_TOKEN_TIMEOUT: u64 = 7200;
const DEFAULT_POOL_SIZE: u32 = 10;
const DEFAULT_TOTP_SECRET_LEN: usize = 32_usize;
const DEFAULT_TOTP_SECRET_NAME: &str = "totp";

const ENV_SERVICE_PORT: &str = "SERVICE_PORT";
const ENV_SERVICE_ADDR: &str = "SERVICE_ADDR";
const ENV_POSTGRES_DSN: &str = "POSTGRES_DSN";
const ENV_POSTGRES_POOL: &str = "POSTGRES_POOL";
const ENV_JWT_SECRET: &str = "JWT_SECRET";
const ENV_JWT_PUBLIC: &str = "JWT_PUBLIC";
const ENV_JWT_HEADER: &str = "JWT_HEADER";
const ENV_TOTP_HEADER: &str = "TOTP_HEADER";
const ENV_REDIS_URL: &str = "REDIS_URL";
const ENV_REDIS_POOL: &str = "REDIS_POOL";
const ENV_TOKEN_TIMEOUT: &str = "TOKEN_TIMEOUT";
const ENV_SMTP_TRANSPORT: &str = "SMTP_TRANSPORT";
const ENV_SMTP_USERNAME: &str = "SMTP_USERNAME";
const ENV_SMTP_PASSWORD: &str = "SMTP_PASSWORD";
const ENV_SMTP_ISSUER: &str = "SMTP_ISSUER";
const ENV_SMTP_TEMPLATES: &str = "SMTP_TEMPLATES";
const ENV_SMTP_ORIGIN: &str = "SMTP_ORIGIN";
const ENV_PWD_SUFIX: &str = "PWD_SUFIX";
const ENV_RABBITMQ_USERS_EXCHANGE: &str = "RABBITMQ_USERS_EXCHANGE";
const ENV_RABBITMQ_URL: &str = "RABBITMQ_URL";
const ENV_RABBITMQ_POOL: &str = "RABBITMQ_POOL";
const ENV_EVENT_ISSUER: &str = "EVENT_ISSUER";
const ENV_TOTP_SECRET_LEN: &str = "TOTP_SECRET_LEN";
const ENV_TOTP_SECRET_NAME: &str = "TOTP_SECRET_NAME";
const ENV_TOKEN_ISSUER: &str = "TOKEN_ISSUER";

lazy_static! {
    pub static ref SERVER_ADDR: String = {
        let netw = env::var(ENV_SERVICE_ADDR).unwrap_or_else(|_| DEFAULT_ADDR.to_string());
        let port = env::var(ENV_SERVICE_PORT).unwrap_or_else(|_| DEFAULT_PORT.to_string());
        format!("{}:{}", netw, port)
    };
    pub static ref TOKEN_TIMEOUT: u64 = env::var(ENV_TOKEN_TIMEOUT)
        .map(|timeout| timeout.parse().unwrap())
        .unwrap_or(DEFAULT_TOKEN_TIMEOUT);
    pub static ref JWT_SECRET: Vec<u8> = env::var(ENV_JWT_SECRET)
        .map(|secret| general_purpose::STANDARD.decode(secret).unwrap())
        .expect("jwt secret must be set");
    pub static ref JWT_PUBLIC: Vec<u8> = env::var(ENV_JWT_PUBLIC)
        .map(|secret| general_purpose::STANDARD.decode(secret).unwrap())
        .expect("jwt public key must be set");
    pub static ref JWT_HEADER: String =
        env::var(ENV_JWT_HEADER).unwrap_or_else(|_| DEFAULT_JWT_HEADER.to_string());
    pub static ref TOTP_HEADER: String =
        env::var(ENV_TOTP_HEADER).unwrap_or_else(|_| DEFAULT_TOTP_HEADER.to_string());
    pub static ref SMTP_TRANSPORT: String =
        env::var(ENV_SMTP_TRANSPORT).expect("smtp transport must be set");
    pub static ref SMTP_USERNAME: String = env::var(ENV_SMTP_USERNAME).unwrap_or_default();
    pub static ref SMTP_PASSWORD: String = env::var(ENV_SMTP_PASSWORD).unwrap_or_default();
    pub static ref SMTP_ORIGIN: String =
        env::var(ENV_SMTP_ORIGIN).expect("smpt origin must be set");
    pub static ref SMTP_ISSUER: String =
        env::var(ENV_SMTP_ISSUER).expect("smtp issuer must be set");
    pub static ref SMTP_TEMPLATES: String =
        env::var(ENV_SMTP_TEMPLATES).unwrap_or_else(|_| DEFAULT_TEMPLATES_PATH.to_string());
    pub static ref PWD_SUFIX: String = env::var(ENV_PWD_SUFIX).expect("password sufix must be set");
    pub static ref POSTGRES_POOL: AsyncOnce<PgPool> = AsyncOnce::new(async {
        let postgres_dsn = env::var(ENV_POSTGRES_DSN).expect("postgres url must be set");

        let postgres_pool = env::var(ENV_POSTGRES_POOL)
            .map(|pool_size| pool_size.parse().unwrap())
            .unwrap_or(DEFAULT_POOL_SIZE);

        PgPoolOptions::new()
            .max_connections(postgres_pool)
            .connect(&postgres_dsn)
            .await
            .map(|pool| {
                info!("connection with postgres cluster established");
                pool
            })
            .map_err(|err| format!("establishing connection with {}: {}", postgres_dsn, err))
            .unwrap()
    });
    pub static ref REDIS_POOL: RedisPool = {
        let redis_dsn: String = env::var(ENV_REDIS_URL).expect("redis url must be set");
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
    pub static ref RABBITMQ_USERS_EXCHANGE: String =
        env::var(ENV_RABBITMQ_USERS_EXCHANGE).expect("rabbitmq users bus name must be set");
    pub static ref RABBITMQ_POOL: AsyncOnce<Pool> = AsyncOnce::new(async {
        let rabbitmq_dsn = env::var(ENV_RABBITMQ_URL).expect("rabbitmq url must be set");
        let rabbitmq_pool = env::var(ENV_RABBITMQ_POOL)
            .map(|pool_size| pool_size.parse().unwrap())
            .unwrap_or_else(|_| DEFAULT_POOL_SIZE.try_into().unwrap());

        let pool = Config {
            url: Some(rabbitmq_dsn.clone()),
            ..Default::default()
        }
        .builder(Some(Runtime::Tokio1))
        .max_size(rabbitmq_pool)
        .build()
        .map(|pool| {
            info!("connection with rabbitmq cluster established");
            pool
        })
        .map_err(|err| format!("establishing connection with {}: {}", rabbitmq_dsn, err))
        .unwrap();

        let channel = pool
            .get()
            .await
            .unwrap()
            .create_channel()
            .await
            .map_err(|err| format!("creating rabbitmq channel: {}", err))
            .unwrap();

        let exchange_options = options::ExchangeDeclareOptions {
            durable: true,
            auto_delete: false,
            internal: false,
            nowait: false,
            passive: false,
        };

        channel
            .exchange_declare(
                &RABBITMQ_USERS_EXCHANGE,
                ExchangeKind::Fanout,
                exchange_options,
                FieldTable::default(),
            )
            .await
            .map_err(|err| {
                format!(
                    "creating rabbitmq exchange {}: {}",
                    &*RABBITMQ_USERS_EXCHANGE, err
                )
            })
            .unwrap();

        pool
    });
    pub static ref EVENT_ISSUER: String =
        env::var(ENV_EVENT_ISSUER).expect("event issuer must be set");
    pub static ref TOTP_SECRET_LEN: usize = env::var(ENV_TOTP_SECRET_LEN)
        .map(|len| len.parse().unwrap())
        .unwrap_or_else(|_| DEFAULT_TOTP_SECRET_LEN);
    pub static ref TOTP_SECRET_NAME: String =
        env::var(ENV_TOTP_SECRET_NAME).unwrap_or_else(|_| DEFAULT_TOTP_SECRET_NAME.to_string());
    pub static ref TOKEN_ISSUER: String =
        env::var(ENV_TOKEN_ISSUER).expect("token issuer must be set");
}
