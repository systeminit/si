use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use dal::{
    ChangeSetError as DalChangeSetError, EditSessionError, StandardModelError, TransactionsError,
};
use thiserror::Error;

pub mod apply_change_set;
pub mod cancel_and_start_edit_session;
pub mod cancel_edit_session;
pub mod create_change_set;
pub mod get_change_set;
pub mod list_open_change_sets;
pub mod save_and_start_edit_session;
pub mod save_edit_session;
pub mod save_edit_session_and_apply_change_set;
pub mod start_edit_session;
pub mod update_selected_change_set;

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
    ContextError(#[from] TransactionsError),
    #[error(transparent)]
    EditSession(#[from] EditSessionError),
    #[error("change set not found")]
    ChangeSetNotFound,
    #[error("edit session not found")]
    EditSessionNotFound,
}

pub type ChangeSetResult<T> = std::result::Result<T, ChangeSetError>;

impl IntoResponse for ChangeSetError {
    fn into_response(self) -> Response {
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
            "/save_edit_session_and_apply_change_set",
            post(save_edit_session_and_apply_change_set::save_edit_session_and_apply_change_set),
        )
        .route(
            "/save_and_start_edit_session",
            post(save_and_start_edit_session::save_and_start_edit_session),
        )
        .route(
            "/cancel_edit_session",
            post(cancel_edit_session::cancel_edit_session),
        )
        .route(
            "/cancel_and_start_edit_session",
            post(cancel_and_start_edit_session::cancel_and_start_edit_session),
        )
        .route(
            "/update_selected_change_set",
            post(update_selected_change_set::update_selected_change_set),
        )
}
