use super::{
    application::{TokenApplication, TokenRepository},
    domain::Token,
};
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
            cfg.service(web::resource("/token/session").route(web::get().to(Self::get_session)));
        }
    }

    async fn get_session(
        app_data: web::Data<Arc<TokenRestService<T>>>,
        req: HttpRequest,
    ) -> impl Responder {
        match async move {
            let token = http::get_encoded_header(req, app_data.jwt_header)?;
            let token = app_data.token_app.decode(&token).await?;

            app_data
                .token_app
                .verify(TokenKind::Session, &token, VerifyOptions::default())
                .await?;

            Ok::<Token, Error>(token)
        }
        .await
        {
            Ok(token) => HttpResponse::Accepted().json(token),
            Err(err) => HttpResponse::from(err),
        }
    }
}
