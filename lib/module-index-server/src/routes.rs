use axum::{
    response::Json,
    response::{IntoResponse, Response},
    routing::{get, post, post_service},
    Router,
};
use hyper::StatusCode;
use serde_json::{json, Value};
use si_data_pg::PgError;
use thiserror::Error;
use tower_http::cors::CorsLayer;

mod get_module_details_route;
mod list_modules_route;
mod upsert_module_route;

use super::{app_state::AppState, server::ServerError};

#[allow(clippy::too_many_arguments)]
pub fn routes(state: AppState) -> Router {
    let mut router: Router<AppState> = Router::new();
    router = router
        .nest(
            "/",
            Router::new().route("/", get(system_status_route).layer(CorsLayer::permissive())),
        )
        .route("/modules", get(list_modules_route::list_module_route))
        .route("/modules", post(upsert_module_route::upsert_module_route))
        .route(
            "/modules/:module_id",
            post(get_module_details_route::get_module_details_route),
        );

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
