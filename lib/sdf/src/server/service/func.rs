use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};

use dal::{
    ReadTenancyError, StandardModelError, TransactionsError, WriteTenancyError, WsEventError,
};

use thiserror::Error;

pub mod create_func;
pub mod get_func;
pub mod list_funcs;
pub mod save_func;

#[derive(Error, Debug)]
pub enum FuncError {
    #[error(transparent)]
    Pg(#[from] si_data::PgError),
    #[error(transparent)]
    PgPool(#[from] si_data::PgPoolError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("write tenancy error: {0}")]
    WriteTenancy(#[from] WriteTenancyError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error(transparent)]
    Func(#[from] dal::FuncError),
    #[error("could not publish websocket event: {0}")]
    WsEvent(#[from] WsEventError),

    #[error("Function not found")]
    FuncNotFound,
    #[error("Function is read-only")]
    NotWritable,
}

pub type FuncResult<T> = std::result::Result<T, FuncError>;

impl IntoResponse for FuncError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub fn routes() -> Router {
    Router::new()
        .route("/list_funcs", get(list_funcs::list_funcs))
        .route("/get_func", get(get_func::get_func))
        .route("/create_func", post(create_func::create_func))
        .route("/save_func", post(save_func::save_func))
}
