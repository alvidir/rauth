use crate::token::{domain::Token, service::TokenService};
use axum::{
    extract::State,
    headers::Header,
    http::StatusCode,
    routing::{delete, get},
    Router, TypedHeader,
};
use std::{marker::PhantomData, sync::Arc};

pub trait TokenHeader: Header {
    fn token(self) -> Token;
}

pub struct SessionRestService<T, H> {
    pub token_srv: Arc<T>,
    pub token_header: PhantomData<H>,
}

impl<T, H> SessionRestService<T, H>
where
    T: 'static + TokenService + Sync + Send,
    H: 'static + TokenHeader + Sync + Send,
{
    pub fn router(
        &self,
        router: Router<Arc<SessionRestService<T, H>>>,
    ) -> Router<Arc<SessionRestService<T, H>>> {
        router
            .route("/token", get(Self::verify))
            .route("/token", delete(Self::revoke))
    }

    #[instrument(skip_all)]
    async fn verify(
        State(state): State<Arc<Self>>,
        TypedHeader(header): TypedHeader<H>,
    ) -> StatusCode {
        if let Err(error) = state.token_srv.claims(header.token()).await {
            return error.into();
        }

        StatusCode::OK
    }

    #[instrument(skip_all)]
    async fn revoke(
        State(state): State<Arc<Self>>,
        TypedHeader(header): TypedHeader<H>,
    ) -> StatusCode {
        let claims = match state.token_srv.claims(header.token()).await {
            Err(error) => return error.into(),
            Ok(claims) => claims,
        };

        match state.token_srv.revoke(&claims).await {
            Err(error) => error.into(),
            Ok(_) => StatusCode::OK,
        }
    }
}
