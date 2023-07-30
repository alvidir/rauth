use crate::{
    cache::Cache,
    http,
    token::application::TokenApplication,
    token::{application::VerifyOptions, domain::TokenKind},
};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::sync::Arc;

use super::application;

pub struct SessionRestService<C: Cache + Sync + Send> {
    pub token_app: TokenApplication<'static, C>,
    pub jwt_header: &'static str,
}

impl<C: 'static + Cache + Sync + Send> SessionRestService<C> {
    pub fn router(&self) -> impl Fn(&mut web::ServiceConfig) {
        |cfg: &mut web::ServiceConfig| {
            cfg.service(web::resource("/session").route(web::get().to(Self::get_session)));
            cfg.service(web::resource("/session").route(web::delete().to(Self::delete_session)));
        }
    }

    #[instrument(skip(app_data))]
    async fn get_session(
        app_data: web::Data<Arc<SessionRestService<C>>>,
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
        app_data: web::Data<Arc<SessionRestService<C>>>,
        req: HttpRequest,
    ) -> impl Responder {
        match async move {
            let token = http::get_encoded_header(req, app_data.jwt_header)?;
            application::logout_strategy::<C>(&app_data.token_app, &token).await
        }
        .await
        {
            Ok(_) => HttpResponse::Ok().finish(),
            Err(err) => HttpResponse::from(err),
        }
    }
}
