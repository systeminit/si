use std::str::FromStr;

use axum::{
    Json,
    extract::{
        Multipart,
        State,
        multipart::MultipartError,
    },
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
};
use sea_orm::DbErr;
use si_id::SchemaId;
use thiserror::Error;

use super::{
    get_module_details_route::GetModuleDetailsError,
    promote_builtin_route::{
        PromoteModuleError,
        promote_module,
    },
    reject_module_route::{
        RejectModuleError,
        reject_other_modules_of_a_schema_id,
    },
    upsert_module_route::upsert_module,
};
use crate::{
    app_state::AppState,
    extract::{
        Authorization,
        DbConnection,
        ExtractedS3Bucket,
    },
    routes::upsert_module_route::{
        UpsertModuleError,
        extract_multiparts,
    },
    whoami::{
        WhoamiError,
        is_systeminit_auth_token,
    },
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum UpsertBuiltinError {
    #[error("db error: {0}")]
    DbErr(#[from] DbErr),
    #[error("getting module details error: {0}")]
    GetModuleDetails(#[from] GetModuleDetailsError),
    #[error("multipart decode error: {0}")]
    Multipart(#[from] MultipartError),
    #[error("error promoting module: {0}")]
    PromoteModule(#[from] PromoteModuleError),
    #[error("error rejecting module: {0}")]
    RejectModule(#[from] RejectModuleError),
    #[error("Ulid decode error: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("error upserting module: {0}")]
    UpsertModule(#[from] UpsertModuleError),
    #[error("whoami error: {0}")]
    Whoami(#[from] WhoamiError),
}

impl IntoResponse for UpsertBuiltinError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub async fn upsert_builtin_route(
    Authorization {
        user_claim,
        auth_token,
    }: Authorization,
    ExtractedS3Bucket { s3_bucket, .. }: ExtractedS3Bucket,
    DbConnection(txn): DbConnection,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<Json<bool>, UpsertBuiltinError> {
    if !is_systeminit_auth_token(state.auth_api_url(), &auth_token, state.token_emails()).await? {
        return Ok(Json(false));
    }

    let multiparts = extract_multiparts(&mut multipart).await?;
    if let Some(schema_id) = multiparts.schema_id.clone() {
        let existing_schema_id = SchemaId::from_str(&schema_id)?;
        reject_other_modules_of_a_schema_id("Clover".to_string(), existing_schema_id, &txn).await?;
    }

    // Upload the new module
    let new_module = upsert_module(multiparts, &txn, user_claim, s3_bucket).await?;

    // Promote the new module to be a builtin
    promote_module(new_module.id, &txn, "Clover".to_string()).await?;

    txn.commit().await?;

    Ok(Json(true))
}
