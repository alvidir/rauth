#[macro_use]
extern crate tracing;

use rauth::{
    cache::RedisCache,
    config, postgres, rabbitmq, redis,
    secret::repository::PostgresSecretRepository,
    session::{
        application::SessionApplication,
        grpc::{SessionGrpcService, SessionServer},
    },
    smtp,
    smtp::SmtpBuilder,
    token::service::TokenServiceImpl,
    tracer,
    user::{
        application::UserApplication,
        event_bus::RabbitMqUserBus,
        grpc::{UserGrpcService, UserServer},
        repository::PostgresUserRepository,
    },
};
use std::sync::Arc;
use std::time::Duration;
use std::{error::Error, net::SocketAddr};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if let Err(err) = dotenv::dotenv() {
        warn!(error = err.to_string(), "processing dotenv file");
    }

    tracer::init()?;

    let secret_repo = Arc::new(PostgresSecretRepository {
        pool: &postgres::POSTGRES_POOL,
    });

    let user_repo = Arc::new(PostgresUserRepository {
        pool: &postgres::POSTGRES_POOL,
    });

    let user_event_bus = Arc::new(RabbitMqUserBus {
        pool: &rabbitmq::RABBITMQ_POOL,
        exchange: &rabbitmq::RABBITMQ_USERS_EXCHANGE,
        issuer: &rabbitmq::EVENT_ISSUER,
    });

    let cache = Arc::new(RedisCache {
        pool: &redis::REDIS_POOL,
    });

    let smtp = SmtpBuilder {
        issuer: &smtp::SMTP_ISSUER,
        origin: &smtp::SMTP_ORIGIN,
        templates: &smtp::SMTP_TEMPLATES,
        transport: &smtp::SMTP_TRANSPORT,
        username: &smtp::SMTP_USERNAME,
        password: &smtp::SMTP_PASSWORD,
        ..Default::default()
    }
    .build()?;

    let token_srv = Arc::new(TokenServiceImpl {
        timeout: Duration::from_secs(*config::TOKEN_TIMEOUT),
        issuer: &config::TOKEN_ISSUER,
        private_key: &config::JWT_SECRET,
        public_key: &config::JWT_PUBLIC,
        cache: cache.clone(),
    });

    let user_app = UserApplication {
        user_repo: user_repo.clone(),
        secret_repo: secret_repo.clone(),
        token_service: token_srv.clone(),
        mailer: Arc::new(smtp),
        event_bus: user_event_bus.clone(),
        totp_secret_len: *config::TOTP_SECRET_LEN,
        cache: cache.clone(),
    };

    let user_grpc_service = UserGrpcService {
        user_app,
        jwt_header: &config::JWT_HEADER,
        totp_header: &config::TOTP_HEADER,
    };

    let session_app = SessionApplication {
        user_repo: user_repo.clone(),
        secret_repo: secret_repo.clone(),
        token_srv: token_srv.clone(),
    };

    let session_grpc_service = SessionGrpcService {
        session_app,
        jwt_header: &config::JWT_HEADER,
    };

    let addr: SocketAddr = config::SERVICE_ADDR.parse().unwrap();
    info!(
        address = addr.to_string(),
        "server ready to accept connections"
    );

    Server::builder()
        .add_service(UserServer::new(user_grpc_service))
        .add_service(SessionServer::new(session_grpc_service))
        .serve(addr)
        .await?;

    tracer::shutdown();
    Ok(())
}
