use axum::{
    extract::{Path, Query},
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use sea_orm::{DbErr, EntityTrait};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

use crate::{
    extract::{Authorization, DbConnection},
    models::si_module::{self, ModuleId},
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum GetModuleDetailsError {
    #[error("db error: {0}")]
    DbErr(#[from] DbErr),
    #[error(r#"Module "{0}" not found"#)]
    NotFound(ModuleId),
}

// TODO: figure out how to not keep this serialization logic here
impl IntoResponse for GetModuleDetailsError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::NotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetModuleDetailsRequest {
    pub foo: Option<bool>,
}

pub async fn get_module_details_route(
    Path(module_id): Path<ModuleId>,
    Authorization { .. }: Authorization,
    DbConnection(txn): DbConnection,
    Query(_request): Query<GetModuleDetailsRequest>,
) -> Result<Json<Value>, GetModuleDetailsError> {
    let module = match si_module::Entity::find_by_id(module_id).one(&txn).await? {
        Some(module) => module,
        _ => return Err(GetModuleDetailsError::NotFound(module_id)),
    };

    Ok(Json(json!(module)))
}
