use super::application::{TokenApplication, TokenRepository};
use crate::result::{Error, Result};
use actix_web::{get, web, HttpResponse, Responder};
use async_trait::async_trait;

#[async_trait]
trait TokenApiRest {
    async fn get_indentifier(&self) -> Result<u32>;
}
pub struct TokenRestService<T: TokenRepository + Sync + Send> {
    pub token_app: TokenApplication<'static, T>,
    pub jwt_header: &'static str,
}

impl<T: TokenRepository + Sync + Send> TokenRestService<T> {
    pub fn register(cfg: &mut web::ServiceConfig) {
        cfg.service(get_indentifier);
    }
}

#[async_trait]
impl<T: TokenRepository + Sync + Send> TokenApiRest for TokenRestService<T> {
    async fn get_indentifier(&self) -> Result<u32> {
        todo!()
    }
}

#[derive(Serialize)]
struct Indentifier(String);

#[get("/indentifier")]
async fn get_indentifier(app_data: web::Data<Box<dyn TokenApiRest>>) -> impl Responder {
    match app_data.get_indentifier().await {
        Ok(identifier) => HttpResponse::Accepted().json(identifier),
        Err(err) => {
            error!(
                "{} performing GET command on redis: {}",
                Error::Unknown,
                err
            );

            HttpResponse::Unauthorized().finish()
        }
    }
}
