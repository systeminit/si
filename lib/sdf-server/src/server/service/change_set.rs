use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use dal::change_status::ChangeStatusError;
use dal::{
    ChangeSetError as DalChangeSetError, ComponentError as DalComponentError, FixError,
    StandardModelError, TransactionsError, UserError, UserPk,
};
use thiserror::Error;

use crate::server::state::AppState;

pub mod apply_change_set;
pub mod apply_change_set2;
pub mod create_change_set;
pub mod get_change_set;
pub mod get_stats;
pub mod list_open_change_sets;
pub mod update_selected_change_set;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangeSetError {
    #[error(transparent)]
    ChangeSet(#[from] DalChangeSetError),
    #[error("change set not found")]
    ChangeSetNotFound,
    #[error(transparent)]
    ChangeStatusError(#[from] ChangeStatusError),
    #[error(transparent)]
    Component(#[from] DalComponentError),
    #[error(transparent)]
    ContextError(#[from] TransactionsError),
    #[error(transparent)]
    Fix(#[from] FixError),
    #[error("invalid user {0}")]
    InvalidUser(UserPk),
    #[error("invalid user system init")]
    InvalidUserSystemInit,
    #[error(transparent)]
    Nats(#[from] si_data_nats::NatsError),
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    User(#[from] UserError),
}

pub type ChangeSetResult<T> = std::result::Result<T, ChangeSetError>;

impl IntoResponse for ChangeSetError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ChangeSetError::ChangeSetNotFound => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router<AppState> {
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
        .route("/get_stats", get(get_stats::get_stats))
        .route(
            "/apply_change_set",
            post(apply_change_set::apply_change_set),
        )
        .route(
            "/apply_change_set2",
            post(apply_change_set2::apply_change_set),
        )
        .route(
            "/update_selected_change_set",
            post(update_selected_change_set::update_selected_change_set),
        )
}
