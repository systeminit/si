use axum::{
    Json,
    extract::{
        Path,
        Query,
    },
    response::{
        IntoResponse,
        Response,
    },
};
use hyper::StatusCode;
use module_index_types::ModuleDetailsResponse;
use sea_orm::{
    ColumnTrait,
    DatabaseTransaction,
    DbErr,
    EntityTrait,
    QueryFilter,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::{
    Value,
    json,
};
use thiserror::Error;

use crate::{
    extract::{
        Authorization,
        DbConnection,
    },
    models::si_module::{
        self,
        ModuleId,
        SchemaIdReferenceLink,
        make_module_details_response,
    },
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
    let module = get_module_by_id_opt(module_id, &txn)
        .await?
        .ok_or(GetModuleDetailsError::NotFound(module_id))?;

    Ok(Json(json!(module)))
}

pub async fn get_module_by_id_opt(
    module_id: ModuleId,
    txn: &DatabaseTransaction,
) -> Result<Option<ModuleDetailsResponse>, GetModuleDetailsError> {
    let query = si_module::Entity::find()
        .find_with_linked(SchemaIdReferenceLink)
        .filter(si_module::Column::Id.eq(module_id));

    let modules: Vec<ModuleDetailsResponse> = query
        .all(txn)
        .await?
        .into_iter()
        .map(|(module, linked_modules)| make_module_details_response(module, linked_modules))
        .collect();

    if modules.len() > 1 {
        return Err(GetModuleDetailsError::MoreThanOneMatchingModuleFor(
            module_id,
        ));
    }

    let module = modules.first().cloned();

    Ok(module)
}
