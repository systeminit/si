use axum::{
    extract::Multipart,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, FixedOffset, Offset, Utc};
use hyper::StatusCode;
use module_index_client::{FuncMetadata, ModuleDetailsResponse};
use s3::error::S3Error;
use sea_orm::{ActiveModelTrait, DbErr, Set};
use serde::{Deserialize, Serialize};
use si_pkg::{SiPkg, SiPkgError, SiPkgKind};
use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::{
    extract::{Authorization, DbConnection, ExtractedS3Bucket},
    models::si_module,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpsertModuleRequest {
    pub foo: Option<bool>,
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum UpsertModuleError {
    #[error("db error: {0}")]
    DbErr(#[from] DbErr),
    #[error("file upload error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("s3 error: {0}")]
    S3Error(#[from] S3Error),
    #[error("JSON serialization/deserialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("module parsing error: {0}")]
    SiPkgError(#[from] SiPkgError),
    #[error("upload is required")]
    UploadRequiredError,
}

// TODO: figure out how to not keep this serialization logic here
impl IntoResponse for UpsertModuleError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

// #[debug_handler]
pub async fn upsert_module_route(
    Authorization { .. }: Authorization,
    ExtractedS3Bucket(s3_bucket): ExtractedS3Bucket,
    DbConnection(txn): DbConnection,
    mut multipart: Multipart,
) -> Result<Json<ModuleDetailsResponse>, UpsertModuleError> {
    info!("Upsert module");
    let field = match multipart.next_field().await.unwrap() {
        Some(f) => f,
        None => return Err(UpsertModuleError::UploadRequiredError),
    };
    info!("Found multipart field");
    let data = field.bytes().await.unwrap();
    info!("Got part data");

    // SiPkg using old term "package" but we are dealing with a "module"
    let loaded_module = dbg!(SiPkg::load_from_bytes(data.to_vec()))?;
    let module_metadata = dbg!(loaded_module.metadata())?;

    let version = module_metadata.version().to_owned();
    let module_kind = match module_metadata.kind() {
        SiPkgKind::WorkspaceBackup => si_module::ModuleKind::WorkspaceBackup,
        SiPkgKind::Module => si_module::ModuleKind::Module,
    };

    let schemas: Vec<String> = loaded_module
        .schemas()?
        .iter()
        .map(|s| s.name().to_owned())
        .collect();
    let funcs: Vec<FuncMetadata> = loaded_module
        .funcs()?
        .iter()
        .map(|f| FuncMetadata {
            name: f.name().to_owned(),
            display_name: f.display_name().map(|d| d.to_owned()),
            description: f.description().map(|d| d.to_owned()),
        })
        .collect();

    let new_module = si_module::ActiveModel {
        name: Set(module_metadata.name().to_owned()),
        description: Set(Some(module_metadata.description().to_owned())),
        // owner_user_id: Set(claim.user_pk.to_string()),
        owner_user_id: Set(Ulid::new().to_string()),
        owner_display_name: Set(Some(module_metadata.created_by().to_owned())),
        latest_hash: Set(module_metadata.hash().to_string()),
        // maybe use db's `CLOCK_TIMESTAMP()`?
        latest_hash_created_at: Set(DateTime::<FixedOffset>::from_utc(
            Utc::now().naive_utc(),
            Utc.fix(),
        )),
        metadata: Set(serde_json::to_value(ExtraMetadata {
            version,
            schemas,
            funcs,
        })?),
        kind: Set(module_kind),
        ..Default::default() // all other attributes are `NotSet`
    };

    // TODO: put below
    // upload to s3
    s3_bucket
        .put_object(format!("{}.sipkg", module_metadata.hash()), &data)
        .await?;

    let new_module: si_module::Model = dbg!(new_module.insert(&txn).await)?;

    txn.commit().await?;

    Ok(dbg!(Json(new_module.try_into()?)))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtraMetadata {
    pub version: String,
    pub schemas: Vec<String>,
    pub funcs: Vec<FuncMetadata>,
}
