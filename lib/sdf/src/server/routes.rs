use std::{convert::Infallible, sync::Arc};

use crate::server::config::JwtSigningKey;
use axum::{
    body::{Bytes, Full},
    response::IntoResponse,
    routing::get,
    AddExtensionLayer, Json, Router,
};
use hyper::StatusCode;
use serde_json::json;
use si_data::{nats, pg};
use thiserror::Error;
use tokio::sync::mpsc;

use super::{
    handlers,
    server::{ServerError, ShutdownSource},
};

pub struct State {
    // TODO(fnichol): we're likely going to use this, but we can't allow it to be dropped because
    // that will trigger the read side and... shutdown. Cool, no?
    #[allow(dead_code)]
    tmp_shutdown_tx: mpsc::Sender<ShutdownSource>,
}

impl State {
    pub fn new(tmp_shutdown_tx: mpsc::Sender<ShutdownSource>) -> Self {
        Self { tmp_shutdown_tx }
    }
}

#[must_use]
pub fn routes(
    pg_pool: pg::PgPool,
    nats: nats::Client,
    jwt_signing_key: JwtSigningKey,
    shutdown_tx: mpsc::Sender<ShutdownSource>,
) -> Router {
    let shared_state = Arc::new(State::new(shutdown_tx));

    let mut router: Router = Router::new();
    router = router
        .route("/demo", get(handlers::demo))
        .nest("/api/signup", crate::server::service::signup::routes())
        .nest("/api/session", crate::server::service::session::routes())
        .nest(
            "/api/change_set",
            crate::server::service::change_set::routes(),
        )
        .nest("/api/schema", crate::server::service::schema::routes());
    router = test_routes(router);
    router = router
        .layer(AddExtensionLayer::new(shared_state.clone()))
        .layer(AddExtensionLayer::new(pg_pool.clone()))
        .layer(AddExtensionLayer::new(nats.clone()))
        .layer(AddExtensionLayer::new(jwt_signing_key));
    router
}

#[cfg(debug_assertions)]
pub fn test_routes(mut router: Router) -> Router {
    router = router.nest("/api/test", crate::server::service::test::routes());
    router
}

#[cfg(not(debug_assertions))]
pub fn test_routes(mut router: Router) -> Router {
    router
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Nats(#[from] nats::Error),
    #[error(transparent)]
    Pg(#[from] pg::Error),
    #[error(transparent)]
    Server(#[from] ServerError),
}

pub type AppResult<T> = std::result::Result<T, AppError>;

impl IntoResponse for AppError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> hyper::Response<Self::Body> {
        let (status, error_message) = match self {
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(json!({ "error": error_message }));

        (status, body).into_response()
    }
}
