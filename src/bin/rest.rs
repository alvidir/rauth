#[macro_use]
extern crate tracing;

use actix_web::web::Data;
use actix_web::{middleware, App, HttpServer};
use rauth::cache::RedisCache;
use rauth::{
    config, redis, session::rest::SessionRestService, token::service::TokenServiceImpl, tracer,
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

    let cache = Arc::new(RedisCache {
        pool: &redis::REDIS_POOL,
    });

    let token_srv = TokenServiceImpl {
        timeout: Duration::from_secs(*config::TOKEN_TIMEOUT),
        issuer: &config::TOKEN_ISSUER,
        private_key: &config::JWT_SECRET,
        public_key: &config::JWT_PUBLIC,
        cache: cache.clone(),
    };

    let session_server = Arc::new(SessionRestService {
        token_srv,
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
