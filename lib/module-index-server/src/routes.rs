use axum::{
    extract::DefaultBodyLimit,
    response::Json,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use hyper::StatusCode;
use serde_json::{json, Value};
use si_data_pg::PgError;
use thiserror::Error;
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;

mod download_builtin_route;
mod download_module_route;
mod get_module_details_route;
mod list_builtins_route;
mod list_modules_route;
pub(crate) mod promote_builtin_route;
pub(crate) mod reject_module_route;
pub(crate) mod upsert_module_route;

use super::{app_state::AppState, server::ServerError};

// 512Mb upload limit
const MAX_UPLOAD_BYTES: usize = 1024 * 1024 * 512;

#[allow(clippy::too_many_arguments)]
pub fn routes(state: AppState) -> Router {
    let mut router: Router<AppState> = Router::new();
    router = router
        .route("/", get(system_status_route))
        .route("/modules", get(list_modules_route::list_module_route))
        .route("/builtins", get(list_builtins_route::list_builtins_route))
        .route(
            "/builtins/:module_id/promote",
            post(promote_builtin_route::promote_builtin_route),
        )
        .route("/modules", post(upsert_module_route::upsert_module_route))
        .route(
            "/modules/:module_id",
            get(get_module_details_route::get_module_details_route),
        )
        .route(
            "/modules/:module_id/download",
            get(download_module_route::download_module_route),
        )
        .route(
            "/modules/:module_id/download_builtin",
            get(download_builtin_route::download_builtin_route),
        )
        .route(
            "/modules/:module_id/reject",
            post(reject_module_route::reject_module),
        )
        .layer(CorsLayer::permissive())
        .layer(DefaultBodyLimit::max(MAX_UPLOAD_BYTES))
        .layer(CompressionLayer::new());

    router.with_state(state)
}

async fn system_status_route() -> Json<Value> {
    Json(json!({ "ok": true }))
}

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum AppError {
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

        (status, body).into_response()
    }
}
