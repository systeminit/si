use axum::{
    extract::{Path, Query},
    response::{IntoResponse, Response},
    Json,
};
use hyper::StatusCode;
use module_index_types::ModuleDetailsResponse;
use sea_orm::{ColumnTrait, DbErr, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use thiserror::Error;

use crate::{
    extract::{Authorization, DbConnection},
    models::si_module::{self, make_module_details_response, ModuleId, SchemaIdReferenceLink},
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum GetModuleDetailsError {
    #[error("db error: {0}")]
    DbErr(#[from] DbErr),
    #[error(r#"More than one matching module found for id: "{0}""#)]
    MoreThanOneMatchingModuleFor(ModuleId),
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
    let query = si_module::Entity::find();

    // filters
    let query = query
        .find_with_linked(SchemaIdReferenceLink)
        .filter(si_module::Column::Id.eq(module_id));

    let modules: Vec<ModuleDetailsResponse> = query
        .all(&txn)
        .await?
        .into_iter()
        .map(|(module, linked_modules)| make_module_details_response(module, linked_modules))
        .collect();

    if modules.len() > 1 {
        return Err(GetModuleDetailsError::MoreThanOneMatchingModuleFor(
            module_id,
        ));
    }

    if modules.is_empty() {
        return Err(GetModuleDetailsError::NotFound(module_id));
    }

    let module = modules
        .first()
        .expect("We have already checked a module exists here");

    Ok(Json(json!(module)))
}
