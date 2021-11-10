use axum::body::{Bytes, Full};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post};
use axum::Json;
use axum::Router;
use dal::{ChangeSetError as DalChangeSetError, EditSessionError, StandardModelError};
use std::convert::Infallible;
use thiserror::Error;

pub mod apply_change_set;
pub mod cancel_edit_session;
pub mod create_change_set;
pub mod get_change_set;
pub mod list_open_change_sets;
pub mod save_edit_session;
pub mod start_edit_session;

#[derive(Debug, Error)]
pub enum ChangeSetError {
    #[error(transparent)]
    Nats(#[from] si_data::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    ChangeSet(#[from] DalChangeSetError),
    #[error(transparent)]
    EditSession(#[from] EditSessionError),
    #[error("change set not found")]
    ChangeSetNotFound,
    #[error("edit session not found")]
    EditSessionNotFound,
}

pub type ChangeSetResult<T> = std::result::Result<T, ChangeSetError>;

impl IntoResponse for ChangeSetError {
    type Body = Full<Bytes>;
    type BodyError = Infallible;

    fn into_response(self) -> hyper::Response<Self::Body> {
        let (status, error_message) = match self {
            ChangeSetError::ChangeSetNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            ChangeSetError::EditSessionNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router {
    Router::new()
        .route(
            "/list_open_change_sets",
            get(list_open_change_sets::list_open_change_sets),
        )
        .route(
            "/create_change_set",
            post(create_change_set::create_change_set),
        )
        .route("/get_change_set", get(get_change_set::get_change_set))
        .route(
            "/apply_change_set",
            post(apply_change_set::apply_change_set),
        )
        .route(
            "/start_edit_session",
            post(start_edit_session::start_edit_session),
        )
        .route(
            "/save_edit_session",
            post(save_edit_session::save_edit_session),
        )
        .route(
            "/cancel_edit_session",
            post(cancel_edit_session::cancel_edit_session),
        )
}
