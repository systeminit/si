use axum::{
    extract::State,
    http::{HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::get,
    Router,
};
use hyper::header;
use hyper::Method;
use serde_json::{json, Value};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;
use tower_http::{
    compression::CompressionLayer,
    cors::{AllowOrigin, CorsLayer},
};

use crate::{
    app_state::{AppState, ApplicationRuntimeMode},
    ServerError,
};

const MAINTENANCE_MODE_MESSAGE: &str = concat!(
    " SI is currently in maintenance mode. ",
    "Please refresh & try again later. ",
    "Reach out to support@systeminit.com ",
    "or in the SI Discord for more information if this problem persists",
);

async fn app_state_middeware<B>(
    State(state): State<AppState>,
    request: Request<B>,
    next: Next<B>,
) -> Response {
    match *state.application_runtime_mode.read().await {
        ApplicationRuntimeMode::Maintenance => {
            // Return a 503 when the server is in maintenance/offline
            (StatusCode::SERVICE_UNAVAILABLE, MAINTENANCE_MODE_MESSAGE).into_response()
        }
        ApplicationRuntimeMode::Running => next.run(request).await,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn routes(state: AppState) -> Router {
    Router::new()
        .nest("/api", v1_routes())
        .nest("/api/public", crate::service::public::routes(state.clone()))
        .nest("/api/v2", crate::service::v2::routes(state.clone()))
        .nest("/api/whoami", crate::service::whoami::routes())
        .layer(CompressionLayer::new())
        // allows us to be permissive about cors from our owned subdomains
        .layer(
            CorsLayer::new()
                .allow_origin(AllowOrigin::predicate(|origin: &HeaderValue, _| {
                    origin.as_bytes().ends_with(b".systeminit.com")
                }))
                .allow_credentials(true)
                .allow_headers(vec![
                    header::ACCEPT,
                    header::ACCEPT_LANGUAGE,
                    header::AUTHORIZATION,
                    header::CONTENT_LANGUAGE,
                    header::CONTENT_TYPE,
                ])
                .allow_methods(vec![
                    Method::GET,
                    Method::POST,
                    Method::PUT,
                    Method::DELETE,
                    Method::HEAD,
                    Method::OPTIONS,
                    Method::CONNECT,
                    Method::PATCH,
                    Method::TRACE,
                ]),
        )
        .layer(middleware::from_fn_with_state(
            state.clone(),
            app_state_middeware,
        ))
        // root health route is currently pinged by auth portal to check if backend is up and running so we need permissive CORS headers
        // it is last in the list so that it still services even if we are in maintenance mode
        .nest(
            "/api/",
            Router::new().route("/", get(system_status_route).layer(CorsLayer::permissive())),
        )
        // Load dev routes if we are in dev mode (decided by "opt-level" at the moment).
        .nest("/api/dev", dev_routes())
        // Consider turning app state into an Arc so that all of the middleware
        // share the same state object, instead of cloning
        .with_state(state)
}

fn v1_routes() -> Router<AppState> {
    Router::new()
        .nest("/action", crate::service::action::routes())
        .nest("/node_debug", crate::service::node_debug::routes())
        .nest("/attribute", crate::service::attribute::routes())
        .nest("/change_set", crate::service::change_set::routes())
        .nest("/component", crate::service::component::routes())
        .nest("/diagram", crate::service::diagram::routes())
        .nest("/graphviz", crate::service::graphviz::routes())
        .nest("/qualification", crate::service::qualification::routes())
        .nest("/secret", crate::service::secret::routes())
        .nest("/session", crate::service::session::routes())
        .nest("/ws", crate::service::ws::routes())
        .nest("/module", crate::service::module::routes())
        .nest("/variant", crate::service::variant::routes())
}

async fn system_status_route() -> Json<Value> {
    Json(json!({ "ok": true }))
}

#[cfg(debug_assertions)]
pub fn dev_routes() -> Router<AppState> {
    crate::service::dev::routes()
}

#[cfg(not(debug_assertions))]
pub fn dev_routes() -> Router<AppState> {
    telemetry::prelude::debug!("skipping dev routes...");
    Router::new()
}

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Nats(#[from] NatsError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    Server(#[from] ServerError),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(serde_json::json!({
            "error": {
                "message": error_message,
                "code": 42,
                "statusCode": status.as_u16(),
            },
        }));
        error!(si.error.message = error_message);
        (status, body).into_response()
    }
}
