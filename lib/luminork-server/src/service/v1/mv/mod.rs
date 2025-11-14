use axum::{
    Router,
    http::StatusCode,
    response::IntoResponse,
    routing::get,
};
use dal::TransactionsError;
use frigg::FriggError;
use thiserror::Error;

use crate::AppState;

pub mod get;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum MvError {
    #[error("decode error: {0}")]
    Decode(#[from] ulid::DecodeError),
    #[error("frigg error: {0}")]
    Frigg(#[from] FriggError),
    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("MV not found error: {0} {1}")]
    NotFound(String, String),
    #[error("slow runtime error: {0}")]
    SlowRuntime(#[from] dal::slow_rt::SlowRuntimeError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
}

impl IntoResponse for MvError {
    fn into_response(self) -> axum::response::Response {
        use crate::service::v1::common::ErrorIntoResponse;
        self.to_api_response()
    }
}

impl crate::service::v1::common::ErrorIntoResponse for MvError {
    fn status_and_message(&self) -> (StatusCode, String) {
        match self {
            MvError::NotFound(_kind, _id) => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        }
    }
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/", get(get::get))
}

pub type MvResult<T> = Result<T, MvError>;
