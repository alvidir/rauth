use super::error::Error;
use crate::{
    http,
    token::{domain::Token, service::TokenService},
};
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use std::sync::Arc;

pub struct SessionRestService<T> {
    pub token_srv: Arc<T>,
    pub jwt_header: &'static str,
}

impl<T> SessionRestService<T>
where
    T: 'static + TokenService + Sync + Send,
{
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
            let Some(header) = http::header(req, app_data.jwt_header).map_err(Error::from)? else {
                return Err(Error::Forbidden);
            };

            let token: Token = header.try_into().map_err(Error::from)?;
            let claims = app_data.token_srv.claims(token).await?;
            if !claims.payload().kind().is_session() {
                return Err(Error::WrongToken);
            }

            Ok(claims)
        }
        .await
        {
            Ok(claims) => HttpResponse::Accepted().json(claims.payload()),
            Err(err) => HttpResponse::from(err),
        }
    }

    #[instrument(skip(app_data))]
    async fn delete_session(
        app_data: web::Data<Arc<SessionRestService<T>>>,
        req: HttpRequest,
    ) -> impl Responder {
        match async move {
            let Some(header) = http::header(req, app_data.jwt_header).map_err(Error::from)? else {
                return Err(Error::Forbidden);
            };

            let token: Token = header.try_into().map_err(Error::from)?;
            let claims = app_data.token_srv.claims(token).await?;
            if !claims.payload().kind().is_session() {
                return Err(Error::WrongToken);
            }

            app_data.token_srv.revoke(&claims).await.map_err(Into::into)
        }
        .await
        {
            Ok(_) => HttpResponse::Accepted().finish(),
            Err(err) => HttpResponse::from(err),
        }
    }
}
