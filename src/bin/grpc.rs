#[macro_use]
extern crate log;

use rauth::{
    config,
    metadata::repository::PostgresMetadataRepository,
    secret::repository::PostgresSecretRepository,
    session::{
        application::SessionApplication,
        grpc::{SessionGrpcService, SessionServer},
    },
    smtp::Smtp,
    token::{application::TokenApplication, repository::RedisTokenRepository},
    user::{
        application::UserApplication,
        event_bus::RabbitMqUserBus,
        grpc::{UserGrpcService, UserServer},
        repository::PostgresUserRepository,
    },
};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    if let Err(err) = dotenv::dotenv() {
        warn!("processing dotenv file {}", err);
    }

    let metadata_repo = Arc::new(PostgresMetadataRepository {
        pool: config::POSTGRES_POOL.get().await,
    });

    let secret_repo = Arc::new(PostgresSecretRepository {
        pool: config::POSTGRES_POOL.get().await,
        metadata_repo: metadata_repo.clone(),
    });

    let user_repo = Arc::new(PostgresUserRepository {
        pool: config::POSTGRES_POOL.get().await,
        metadata_repo: metadata_repo.clone(),
    });

    let user_event_bus = Arc::new(RabbitMqUserBus {
        pool: config::RABBITMQ_POOL.get().await,
        exchange: &config::RABBITMQ_USERS_EXCHANGE,
        issuer: &config::EVENT_ISSUER,
    });

    let token_repo = Arc::new(RedisTokenRepository {
        pool: &config::REDIS_POOL,
    });
    let credentials = if config::SMTP_USERNAME.len() > 0 && config::SMTP_PASSWORD.len() > 0 {
        Some((
            config::SMTP_USERNAME.to_string(),
            config::SMTP_PASSWORD.to_string(),
        ))
    } else {
        None
    };

    let mut mailer = Smtp::new(
        &config::SMTP_TEMPLATES,
        &config::SMTP_TRANSPORT,
        credentials,
    )?;
    mailer.issuer = &*config::SMTP_ISSUER;
    mailer.origin = &*config::SMTP_ORIGIN;

    let token_app = Arc::new(TokenApplication {
        token_repo: token_repo.clone(),
        timeout: Duration::from_secs(*config::TOKEN_TIMEOUT),
        token_issuer: &config::TOKEN_ISSUER,
        private_key: &config::JWT_SECRET,
        public_key: &config::JWT_PUBLIC,
    });

    let user_app = UserApplication {
        user_repo: user_repo.clone(),
        secret_repo: secret_repo.clone(),
        token_app: token_app.clone(),
        mailer: Arc::new(mailer),
        event_bus: user_event_bus.clone(),
        totp_secret_len: *config::TOTP_SECRET_LEN,
        totp_secret_name: &config::TOTP_SECRET_NAME,
        pwd_sufix: &config::PWD_SUFIX,
    };

    let user_grpc_service = UserGrpcService {
        user_app,
        jwt_header: &config::JWT_HEADER,
        totp_header: &config::TOTP_HEADER,
    };

    let session_app = SessionApplication {
        user_repo: user_repo.clone(),
        secret_repo: secret_repo.clone(),
        token_app: token_app.clone(),
        totp_secret_name: &config::TOTP_SECRET_NAME,
        pwd_sufix: &config::PWD_SUFIX,
    };

    let session_grpc_service = SessionGrpcService {
        session_app,
        jwt_header: &config::JWT_HEADER,
    };

    let addr = config::SERVER_ADDR.parse().unwrap();
    info!("server listening on {}", addr);
    Server::builder()
        .add_service(UserServer::new(user_grpc_service))
        .add_service(SessionServer::new(session_grpc_service))
        .serve(addr)
        .await?;

    Ok(())
}
