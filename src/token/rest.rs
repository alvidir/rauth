use super::application::{TokenApplication, TokenRepository};
use crate::{
    http,
    result::Error,
    token::{application::VerifyOptions, domain::TokenKind},
};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::sync::Arc;

pub struct TokenRestService<T: TokenRepository + Sync + Send> {
    pub token_app: TokenApplication<'static, T>,
    pub jwt_header: &'static str,
}

impl<T: 'static + TokenRepository + Sync + Send> TokenRestService<T> {
    pub fn router(&self) -> impl Fn(&mut web::ServiceConfig) {
        |cfg: &mut web::ServiceConfig| {
            cfg.service(web::resource("/session").route(web::get().to(Self::get_session)));
        }
    }

    async fn get_session(
        app_data: web::Data<Arc<TokenRestService<T>>>,
        req: HttpRequest,
    ) -> impl Responder {
        let token = match http::get_encoded_header(req, app_data.jwt_header) {
            Ok(header) => header,
            Err(err) => {
                warn!("{} getting encoded header: {}", Error::InvalidHeader, err);
                return HttpResponse::Unauthorized().finish();
            }
        };

        let Ok(token) = app_data.token_app.decode(&token).await else {
            error!("cannot decode token");
            return HttpResponse::Unauthorized().finish();
        };

        if app_data
            .token_app
            .verify(TokenKind::Session, &token, VerifyOptions::default())
            .await
            .is_err()
        {
            error!("could not verify token");
            return HttpResponse::Unauthorized().finish();
        }

        HttpResponse::Accepted().json(token)
    }
}
