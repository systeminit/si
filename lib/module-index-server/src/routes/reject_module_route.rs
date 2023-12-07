use axum::extract::{Multipart, State};
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::{extract::Path, Json};
use chrono::{DateTime, Offset, Utc};
use module_index_client::ModuleDetailsResponse;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, DbErr, EntityTrait};
use telemetry::prelude::info;
use thiserror::Error;

use crate::app_state::AppState;
use crate::routes::upsert_module_route::UpsertModuleError;
use crate::whoami::{is_systeminit_auth_token, WhoamiError};
use crate::{
    extract::{Authorization, DbConnection, ExtractedS3Bucket},
    models::si_module::{self, ModuleId},
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum RejectModuleError {
    #[error("db error: {0}")]
    DbErr(#[from] DbErr),
    #[error(r#"Module "{0}" not found"#)]
    NotFound(ModuleId),
    #[error("error rejecting module: {0}")]
    RejectModule(#[from] UpsertModuleError),
    #[error("missing username for module rejection")]
    UserSupplied(),
    #[error("whoami error: {0}")]
    Whoami(#[from] WhoamiError),
}

impl IntoResponse for RejectModuleError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub async fn reject_module(
    Path(module_id): Path<ModuleId>,
    Authorization {
        user_claim: _user_claim,
        auth_token,
    }: Authorization,
    ExtractedS3Bucket(_s3_bucket): ExtractedS3Bucket,
    DbConnection(txn): DbConnection,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<Option<ModuleDetailsResponse>>, RejectModuleError> {
    if !is_systeminit_auth_token(&auth_token, state.token_emails()).await? {
        return Ok(Json(None));
    }

    info!("Reject module");
    let field = match multipart.next_field().await.unwrap() {
        Some(f) => f,
        None => return Err(RejectModuleError::UserSupplied()),
    };
    info!("Found multipart field");
    let data = dbg!(field.text().await.unwrap());
    info!("Got part data");

    let module = match si_module::Entity::find_by_id(module_id).one(&txn).await? {
        Some(module) => module,
        _ => return Err(RejectModuleError::NotFound(module_id)),
    };

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
        rejected_at: Set(Some(DateTime::from_naive_utc_and_offset(
            Utc::now().naive_utc(),
            Utc.fix(),
        ))),
        rejected_by_display_name: Set(Some(data)),
        kind: Set(module.kind),
        is_builtin_at: Set(module.is_builtin_at),
        is_builtin_at_by_display_name: Set(module.is_builtin_at_by_display_name),
    };

    let updated_module: si_module::Model = dbg!(active_module.update(&txn).await)?;

    txn.commit().await?;

    Ok(Json(Some(updated_module.try_into()?)))
}
