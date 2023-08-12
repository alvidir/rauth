#[macro_use]
extern crate tracing;

use actix_web::web::Data;
use actix_web::{middleware, App, HttpServer};
use rauth::{
    config, redis,
    session::rest::SessionRestService,
    token::{application::TokenApplication, repository::RedisTokenRepository},
    tracer,
};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if let Err(err) = dotenv::dotenv() {
        warn!(error = err.to_string(), "processing dotenv file",);
    }

    tracer::init()?;

    let token_repo = Arc::new(RedisTokenRepository {
        pool: &redis::REDIS_POOL,
    });

    let token_app = TokenApplication {
        token_repo: token_repo.clone(),
        timeout: Duration::from_secs(*config::TOKEN_TIMEOUT),
        token_issuer: &config::TOKEN_ISSUER,
        private_key: &config::JWT_SECRET,
        public_key: &config::JWT_PUBLIC,
    };

    let session_server = Arc::new(SessionRestService {
        token_app,
        jwt_header: &config::JWT_HEADER,
    });

    info!(
        address = *config::SERVICE_ADDR,
        "server ready to accept connections"
    );

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Logger::default())
            .app_data(Data::new(session_server.clone()))
            .configure(session_server.router())
    })
    .bind(&*config::SERVICE_ADDR)?
    .run()
    .await
    .unwrap();

    tracer::shutdown();
    Ok(())
}
