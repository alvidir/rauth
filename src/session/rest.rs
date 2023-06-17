use crate::{
    http,
    token::application::{TokenApplication, TokenRepository},
    token::{application::VerifyOptions, domain::TokenKind},
};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::sync::Arc;

use super::application;

pub struct SessionRestService<T: TokenRepository + Sync + Send> {
    pub token_app: TokenApplication<'static, T>,
    pub jwt_header: &'static str,
}

impl<T: 'static + TokenRepository + Sync + Send> SessionRestService<T> {
    pub fn router(&self) -> impl Fn(&mut web::ServiceConfig) {
        |cfg: &mut web::ServiceConfig| {
            cfg.service(web::resource("/session").route(web::get().to(Self::get_session)));
            cfg.service(web::resource("/session").route(web::delete().to(Self::delete_session)));
        }
    }

    #[instrument(skip(app_data))]
    async fn get_session(
        app_data: web::Data<Arc<SessionRestService<T>>>,
        req: HttpRequest,
    ) -> impl Responder {
        match async move {
            let token = http::get_encoded_header(req, app_data.jwt_header)?;
            let token = app_data.token_app.decode(&token).await?;

            app_data
                .token_app
                .verify(&token, VerifyOptions::new(TokenKind::Session))
                .await
                .map(|_| token)
        }
        .await
        {
            Ok(token) => HttpResponse::Accepted().json(token),
            Err(err) => HttpResponse::from(err),
        }
    }

    #[instrument(skip(app_data))]
    async fn delete_session(
        app_data: web::Data<Arc<SessionRestService<T>>>,
        req: HttpRequest,
    ) -> impl Responder {
        match async move {
            let token = http::get_encoded_header(req, app_data.jwt_header)?;
            application::logout_strategy::<T>(&app_data.token_app, &token).await
        }
        .await
        {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(err) => HttpResponse::from(err),
        }
    }
}
