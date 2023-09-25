#[macro_use]
extern crate tracing;

use jsonwebtoken::{DecodingKey, EncodingKey};
use rauth::cache::RedisCache;
use rauth::{config, redis, token::service::JsonWebTokenService, tracer};
use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    Ok(())
    // if let Err(err) = dotenv::dotenv() {
    //     warn!(error = err.to_string(), "processing dotenv file",);
    // }

    // tracer::init()?;

    // let cache = Arc::new(RedisCache {
    //     pool: &redis::REDIS_POOL,
    // });

    // let token_srv = Arc::new(JsonWebTokenService {
    //     session_timeout: Duration::from_secs(*config::TOKEN_TIMEOUT),
    //     verification_timeout: Duration::from_secs(*config::TOKEN_TIMEOUT),
    //     reset_timeout: Duration::from_secs(*config::TOKEN_TIMEOUT),
    //     issuer: &config::TOKEN_ISSUER,
    //     encode: EncodingKey::from_ec_pem(&config::JWT_SECRET)?,
    //     decode: DecodingKey::from_ec_pem(&config::JWT_PUBLIC)?,
    //     cache,
    // });

    // let session_server = Arc::new(SessionRestService {
    //     token_srv,
    //     jwt_header: &config::JWT_HEADER,
    // });

    // info!(
    //     address = *config::SERVICE_ADDR,
    //     "server ready to accept connections"
    // );

    // HttpServer::new(move || {
    //     App::new()
    //         .wrap(middleware::Logger::default())
    //         .app_data(Data::new(session_server.clone()))
    //         .configure(session_server.router())
    // })
    // .bind(&*config::SERVICE_ADDR)?
    // .run()
    // .await
    // .unwrap();

    // tracer::shutdown();
    // Ok(())
}
