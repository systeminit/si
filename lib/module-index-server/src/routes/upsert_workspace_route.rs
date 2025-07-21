use axum::{
    Json,
    extract::{
        Multipart,
        multipart::MultipartError,
    },
    response::{
        IntoResponse,
        Response,
    },
};
use chrono::{
    DateTime,
    FixedOffset,
    Offset,
    Utc,
};
use hyper::StatusCode;
use module_index_types::ExtraMetadata;
use s3::error::S3Error;
use sea_orm::{
    ActiveModelTrait,
    DbErr,
    Set,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_hash::Hash;
use si_pkg::{
    SiPkgError,
    WorkspaceExport,
};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    extract::{
        Authorization,
        DbConnection,
        ExtractedS3Bucket,
    },
    models::{
        si_module,
        si_module::ModuleKind,
    },
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpsertModuleRequest {
    pub foo: Option<bool>,
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum UpsertWorkspaceError {
    #[error("db error: {0}")]
    DbErr(#[from] DbErr),
    #[error("file upload error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("multipart decode error: {0}")]
    Multipart(#[from] MultipartError),
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
impl IntoResponse for UpsertWorkspaceError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        error!("upsert error: {}", &error_message);

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub async fn upsert_workspace_route(
    Authorization { user_claim, .. }: Authorization,
    ExtractedS3Bucket { s3_bucket, .. }: ExtractedS3Bucket,
    DbConnection(txn): DbConnection,
    mut multipart: Multipart,
) -> Result<(), UpsertWorkspaceError> {
    let field = match multipart.next_field().await.unwrap() {
        Some(f) => f,
        None => return Err(UpsertWorkspaceError::UploadRequiredError),
    };
    let data = field.bytes().await?;

    let content: WorkspaceExport = serde_json::from_slice(&data)?;

    let export_metadata = &content.into_latest().metadata;
    let hash = Hash::new(&data).to_string();
    let version = export_metadata.version.to_owned();

    info!("upserting workspace: {:?}", &export_metadata);

    let new_module = si_module::ActiveModel {
        name: Set(export_metadata.name.to_owned()),
        description: Set(Some(export_metadata.description.to_owned())),
        owner_user_id: Set(user_claim.user_id().to_string()),
        owner_display_name: Set(Some(export_metadata.created_by.to_owned())),
        latest_hash: Set(hash.to_string()),
        latest_hash_created_at: Set(DateTime::<FixedOffset>::from_naive_utc_and_offset(
            Utc::now().naive_utc(),
            Utc.fix(),
        )),
        kind: Set(ModuleKind::WorkspaceBackup),
        metadata: Set(serde_json::to_value(ExtraMetadata {
            version,
            schemas: vec![],
            funcs: vec![],
        })?),

        ..Default::default() // all other attributes are `NotSet`
    };

    s3_bucket
        .put_object(format!("{hash}.workspace_export"), &data)
        .await?;

    let _new_module: si_module::Model = dbg!(new_module.insert(&txn).await)?;

    txn.commit().await?;

    Ok(())
}
