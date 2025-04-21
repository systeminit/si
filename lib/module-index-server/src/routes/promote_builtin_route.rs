use axum::extract::multipart::MultipartError;
use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{Json, extract::Path};
use chrono::{DateTime, FixedOffset, Offset, Utc};
use module_index_types::ModuleDetailsResponse;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait, QuerySelect};
use telemetry::prelude::info;
use thiserror::Error;

use crate::app_state::AppState;
use crate::models::si_module::make_module_details_response;
use crate::routes::upsert_module_route::UpsertModuleError;
use crate::whoami::{WhoamiError, is_systeminit_auth_token};
use crate::{
    extract::{Authorization, DbConnection},
    models::si_module::{self, ModuleId},
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum PromoteModuleError {
    #[error("db error: {0}")]
    DbErr(#[from] DbErr),
    #[error("multipart decode error: {0}")]
    Multipart(#[from] MultipartError),
    #[error(r#"Module "{0}" not found"#)]
    NotFound(ModuleId),
    #[error("error rejecting module: {0}")]
    RejectModule(#[from] UpsertModuleError),
    #[error("missing username for module rejection")]
    UserSupplied(),
    #[error("whoami error: {0}")]
    Whoami(#[from] WhoamiError),
}

impl IntoResponse for PromoteModuleError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub async fn promote_builtin_route(
    Path(module_id): Path<ModuleId>,
    Authorization {
        user_claim: _user_claim,
        auth_token,
    }: Authorization,
    DbConnection(txn): DbConnection,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Option<ModuleDetailsResponse>>, PromoteModuleError> {
    if !is_systeminit_auth_token(state.auth_api_url(), &auth_token, state.token_emails()).await? {
        return Ok(Json(None));
    }

    info!("Promote to builtin");
    let field = match multipart.next_field().await? {
        Some(f) => f,
        None => return Err(PromoteModuleError::UserSupplied()),
    };
    info!("Found multipart field");
    let data = field.text().await?;
    info!("Got part data");

    let (linked_modules, updated_module) = promote_module(module_id, &txn, data).await?;

    txn.commit().await?;

    let response = make_module_details_response(updated_module, linked_modules);

    Ok(Json(Some(response)))
}

pub async fn promote_module(
    module_id: ModuleId,
    txn: &sea_orm::DatabaseTransaction,
    data: String,
) -> Result<(Vec<si_module::Model>, si_module::Model), PromoteModuleError> {
    let (module, linked_modules) = si_module::Entity::find_by_id(module_id)
        .limit(1)
        .find_with_linked(si_module::SchemaIdReferenceLink)
        .all(txn)
        .await?
        .first()
        .cloned()
        .ok_or(PromoteModuleError::NotFound(module_id))?;
    let active_module = si_module::ActiveModel {
        id: Set(module.id),
        name: Set(module.name),
        description: Set(module.description),
        owner_user_id: Set(module.owner_user_id),
        owner_display_name: Set(module.owner_display_name),
        metadata: Set(module.metadata),
        latest_hash: Set(module.latest_hash),
        latest_hash_created_at: Set(module.latest_hash_created_at),
        created_at: Set(module.created_at),
        rejected_at: Set(None),
        rejected_by_display_name: Set(None),
        kind: Set(module.kind),
        is_builtin_at: Set(Some(DateTime::<FixedOffset>::from_naive_utc_and_offset(
            Utc::now().naive_utc(),
            Utc.fix(),
        ))),
        is_builtin_at_by_display_name: Set(Some(data)),
        schema_id: Set(module.schema_id),
        ..Default::default() // all other attributes are `NotSet`
    };
    let updated_module: si_module::Model = active_module.update(txn).await?;
    Ok((linked_modules, updated_module))
}
