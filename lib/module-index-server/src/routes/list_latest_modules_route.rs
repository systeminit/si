use axum::{
    Json,
    response::{IntoResponse, Response},
};
use hyper::StatusCode;
use module_index_types::ListLatestModulesResponse;
use sea_orm::{DbBackend, DbErr, EntityTrait, Statement};
use thiserror::Error;

use crate::{
    extract::DbConnection,
    models::si_module::{self, make_latest_modules_response},
    whoami::WhoamiError,
};

const LIST_LATEST_MODULES_QUERY: &str = include_str!("../queries/list_latest_modules.sql");

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ListModulesError {
    #[error("db error: {0}")]
    DbErr(#[from] DbErr),
    #[error("whoami error: {0}")]
    Whoami(#[from] WhoamiError),
}

// TODO: figure out how to not keep this serialization logic here
impl IntoResponse for ListModulesError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub async fn list_latest_modules_route(
    DbConnection(txn): DbConnection,
) -> Result<Json<ListLatestModulesResponse>, ListModulesError> {
    let raw_modules = si_module::Entity::find()
        .from_raw_sql(Statement::from_sql_and_values(
            DbBackend::Postgres,
            LIST_LATEST_MODULES_QUERY,
            [],
        ))
        .all(&txn)
        .await?;

    let modules = raw_modules
        .into_iter()
        .map(make_latest_modules_response)
        .collect();

    Ok(Json(ListLatestModulesResponse { modules }))
}
