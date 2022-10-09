#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

use async_once::AsyncOnce;
use lapin::{
    options::*, types::FieldTable, Channel, Connection, ConnectionProperties, ExchangeKind,
};

use rauth::{
    metadata::repository::PostgresMetadataRepository,
    secret::repository::PostgresSecretRepository,
    session::{
        application::SessionApplication,
        grpc::{SessionImplementation, SessionServer},
        repository::RedisTokenRepository,
    },
    smtp::Smtp,
    user::{
        application::UserApplication,
        bus::RabbitMqUserBus,
        grpc::{UserImplementation, UserServer},
        repository::PostgresUserRepository,
    },
};
use reool::RedisPool;
use sqlx::postgres::{PgPool, PgPoolOptions};
use std::env;
use std::error::Error;
use std::sync::Arc;
use tokio::runtime::Handle;
use tonic::transport::Server;

const DEFAULT_NETW: &str = "127.0.0.1";
const DEFAULT_PORT: &str = "8000";
const DEFAULT_TEMPLATES_PATH: &str = "/etc/rauth/smtp/templates/*.html";
const DEFAULT_EMAIL_ISSUER: &str = "rauth";
const DEFAULT_PWD_SUFIX: &str = "::PWD::RAUTH";
const DEFAULT_JWT_HEADER: &str = "authorization";
const DEFAULT_TOTP_HEADER: &str = "x-totp-secret";
const DEFAULT_TOKEN_TIMEOUT: u64 = 7200;
const DEFAULT_POOL_SIZE: u32 = 10;
const DEFAULT_BUS: &str = "rauth";
const DEFAULT_TOTP_SECRET_LEN: usize = 32_usize;
const DEFAULT_TOTP_SECRET_NAME: &str = ".totp_secret";
const DEFAULT_TOKEN_ISSUER: &str = "rauth.alvidir.com";

const ENV_SERVICE_PORT: &str = "SERVICE_PORT";
const ENV_SERVICE_NETW: &str = "SERVICE_NETW";
const ENV_POSTGRES_DSN: &str = "POSTGRES_DSN";
const ENV_JWT_SECRET: &str = "JWT_SECRET";
const ENV_JWT_PUBLIC: &str = "JWT_PUBLIC";
const ENV_JWT_HEADER: &str = "JWT_HEADER";
const ENV_TOTP_HEADER: &str = "TOTP_HEADER";
const ENV_REDIS_DSN: &str = "REDIS_DSN";
const ENV_TOKEN_TIMEOUT: &str = "TOKEN_TIMEOUT";
const ENV_POSTGRES_POOL: &str = "POSTGRES_POOL";
const ENV_REDIS_POOL: &str = "REDIS_POOL";
const ENV_SMTP_TRANSPORT: &str = "SMTP_TRANSPORT";
const ENV_SMTP_USERNAME: &str = "SMTP_USERNAME";
const ENV_SMTP_PASSWORD: &str = "SMTP_PASSWORD";
const ENV_SMTP_ISSUER: &str = "SMTP_ISSUER";
const ENV_SMTP_TEMPLATES: &str = "SMTP_TEMPLATES";
const ENV_SMTP_ORIGIN: &str = "SMTP_ORIGIN";
const ENV_PWD_SUFIX: &str = "PWD_SUFIX";
const ENV_RABBITMQ_USERS_BUS: &str = "RABBITMQ_USERS_BUS";
const ENV_RABBITMQ_DSN: &str = "RABBITMQ_URL";
const ENV_TOTP_SECRET_LEN: &str = "TOTP_SECRET_LEN";
const ENV_TOTP_SECRET_NAME: &str = "TOTP_SECRET_NAME";
const ENV_TOKEN_ISSUER: &str = "TOKEN_ISSUER";

lazy_static! {
    static ref TOKEN_TIMEOUT: u64 = env::var(ENV_TOKEN_TIMEOUT)
        .map(|timeout| timeout.parse().unwrap())
        .unwrap_or(DEFAULT_TOKEN_TIMEOUT);
    static ref JWT_SECRET: Vec<u8> = env::var(ENV_JWT_SECRET)
        .map(|secret| base64::decode(secret).unwrap())
        .expect("jwt secret must be set");
    static ref JWT_PUBLIC: Vec<u8> = env::var(ENV_JWT_PUBLIC)
        .map(|secret| base64::decode(secret).unwrap())
        .expect("jwt public key must be set");
    static ref JWT_HEADER: String =
        env::var(ENV_JWT_HEADER).unwrap_or_else(|_| DEFAULT_JWT_HEADER.to_string());
    static ref TOTP_HEADER: String =
        env::var(ENV_TOTP_HEADER).unwrap_or_else(|_| DEFAULT_TOTP_HEADER.to_string());
    static ref SMTP_TRANSPORT: String =
        env::var(ENV_SMTP_TRANSPORT).expect("smtp transport must be set");
    static ref SMTP_USERNAME: String = env::var(ENV_SMTP_USERNAME).unwrap_or_default();
    static ref SMTP_PASSWORD: String = env::var(ENV_SMTP_PASSWORD).unwrap_or_default();
    static ref SMTP_ORIGIN: String = env::var(ENV_SMTP_ORIGIN).expect("smpt origin must be set");
    static ref SMTP_ISSUER: String =
        env::var(ENV_SMTP_ISSUER).unwrap_or_else(|_| DEFAULT_EMAIL_ISSUER.to_string());
    static ref SMTP_TEMPLATES: String =
        env::var(ENV_SMTP_TEMPLATES).unwrap_or_else(|_| DEFAULT_TEMPLATES_PATH.to_string());
    static ref PWD_SUFIX: String =
        env::var(ENV_PWD_SUFIX).unwrap_or_else(|_| DEFAULT_PWD_SUFIX.to_string());
    static ref PG_POOL: AsyncOnce<PgPool> = AsyncOnce::new(async {
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
    static ref RABBITMQ_BUS: String =
        env::var(ENV_RABBITMQ_USERS_BUS).unwrap_or_else(|_| DEFAULT_BUS.to_string());
    static ref RABBITMQ_CONN: AsyncOnce<Channel> = AsyncOnce::new(async {
        let rabbitmq_dsn = env::var(ENV_RABBITMQ_DSN).expect("rabbitmq url must be set");
        let conn = Connection::connect(&rabbitmq_dsn, ConnectionProperties::default())
            .await
            .map(|pool| {
                info!("connection with rabbitmq cluster established");
                pool
            })
            .map_err(|err| format!("establishing connection with {}: {}", rabbitmq_dsn, err))
            .unwrap();

        let channel = conn
            .create_channel()
            .await
            .map_err(|err| format!("creating rabbitmq channel: {}", err))
            .unwrap();

        let exchange_options = ExchangeDeclareOptions {
            durable: true,
            auto_delete: false,
            internal: false,
            nowait: false,
            passive: false,
        };

        channel
            .exchange_declare(
                &RABBITMQ_BUS,
                ExchangeKind::Fanout,
                exchange_options,
                FieldTable::default(),
            )
            .await
            .map_err(|err| format!("creating rabbitmq exchange {}: {}", &*RABBITMQ_BUS, err))
            .unwrap();

        channel
    });
    static ref TOTP_SECRET_LEN: usize = env::var(ENV_TOTP_SECRET_LEN)
        .map(|len| len.parse().unwrap())
        .unwrap_or_else(|_| DEFAULT_TOTP_SECRET_LEN);
    static ref TOTP_SECRET_NAME: String =
        env::var(ENV_TOTP_SECRET_NAME).unwrap_or_else(|_| DEFAULT_TOTP_SECRET_NAME.to_string());
    static ref TOKEN_ISSUER: String =
        env::var(ENV_TOKEN_ISSUER).unwrap_or_else(|_| DEFAULT_TOKEN_ISSUER.to_string());
}

pub async fn start_server(address: String) -> Result<(), Box<dyn Error>> {
    let metadata_repo = Arc::new(PostgresMetadataRepository {
        pool: PG_POOL.get().await,
    });

    let secret_repo = Arc::new(PostgresSecretRepository {
        pool: PG_POOL.get().await,
        metadata_repo: metadata_repo.clone(),
    });

    let user_repo = Arc::new(PostgresUserRepository {
        pool: PG_POOL.get().await,
        metadata_repo: metadata_repo.clone(),
    });

    let user_event_bus = Arc::new(RabbitMqUserBus {
        channel: RABBITMQ_CONN.get().await,
        bus: &*RABBITMQ_BUS,
    });

    let token_repo = Arc::new(RedisTokenRepository {
        pool: &RD_POOL,
        jwt_secret: &JWT_SECRET,
        jwt_public: &JWT_PUBLIC,
    });

    let credentials = if SMTP_USERNAME.len() > 0 && SMTP_PASSWORD.len() > 0 {
        Some((SMTP_USERNAME.to_string(), SMTP_PASSWORD.to_string()))
    } else {
        None
    };

    let mut mailer = Smtp::new(&SMTP_TEMPLATES, &SMTP_TRANSPORT, credentials)?;
    mailer.issuer = &*SMTP_ISSUER;
    mailer.origin = &*SMTP_ORIGIN;

    let user_app = UserApplication {
        user_repo: user_repo.clone(),
        secret_repo: secret_repo.clone(),
        token_repo: token_repo.clone(),
        mailer: Arc::new(mailer),
        bus: user_event_bus.clone(),
        timeout: *TOKEN_TIMEOUT,
        totp_secret_len: *TOTP_SECRET_LEN,
        totp_secret_name: &TOTP_SECRET_NAME,
        token_issuer: &TOKEN_ISSUER,
    };

    let sess_app = SessionApplication {
        token_repo: token_repo.clone(),
        user_repo: user_repo.clone(),
        secret_repo: secret_repo.clone(),
        timeout: *TOKEN_TIMEOUT,
        totp_secret_name: &TOTP_SECRET_NAME,
        token_issuer: &TOKEN_ISSUER,
    };

    let user_server = UserImplementation {
        user_app,
        jwt_secret: &JWT_SECRET,
        jwt_public: &JWT_PUBLIC,
        jwt_header: &JWT_HEADER,
        totp_header: &TOTP_HEADER,
        pwd_sufix: &PWD_SUFIX,
    };

    let sess_server = SessionImplementation {
        sess_app,
        jwt_secret: &JWT_SECRET,
        jwt_public: &JWT_PUBLIC,
        jwt_header: &JWT_HEADER,
        pwd_sufix: &PWD_SUFIX,
    };
    let addr = address.parse().unwrap();
    info!("server listening on {}", addr);
    Server::builder()
        .add_service(UserServer::new(user_server))
        .add_service(SessionServer::new(sess_server))
        .serve(addr)
        .await?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    if let Err(err) = dotenv::dotenv() {
        warn!("processing dotenv file {}", err);
    }

    let netw = env::var(ENV_SERVICE_NETW).unwrap_or_else(|_| DEFAULT_NETW.to_string());

    let port = env::var(ENV_SERVICE_PORT).unwrap_or_else(|_| DEFAULT_PORT.to_string());
    let addr = format!("{}:{}", netw, port);
    start_server(addr).await
}
