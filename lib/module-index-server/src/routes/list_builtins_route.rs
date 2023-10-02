use axum::{
    extract::{Query, State},
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::models::si_module::ModuleKind;
use crate::{app_state::AppState, extract::DbConnection, models::si_module, whoami::WhoamiError};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ListBuiltinsError {
    #[error("db error: {0}")]
    DbErr(#[from] DbErr),
    #[error("whoami error: {0}")]
    Whoami(#[from] WhoamiError),
}

// TODO: figure out how to not keep this serialization logic here
impl IntoResponse for ListBuiltinsError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListBuiltinsRequest {
    pub name: Option<String>,
    pub su: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListBuiltinsResponse {
    modules: Vec<si_module::Model>,
}

pub async fn list_builtins_route(
    DbConnection(txn): DbConnection,
    Query(_request): Query<ListBuiltinsRequest>,
    State(_state): State<AppState>,
) -> Result<Json<ListBuiltinsResponse>, ListBuiltinsError> {
    let query = si_module::Entity::find();

    // filters
    let query = query.filter(si_module::Column::IsBuiltinAt.is_not_null());

    let query = query
        .filter(si_module::Column::RejectedAt.is_null())
        .filter(si_module::Column::Kind.eq(ModuleKind::Module));

    // This should give us a list of builtin modules that are not rejected
    let modules: Vec<si_module::Model> = query.all(&txn).await?;

    Ok(Json(ListBuiltinsResponse { modules }))
}
