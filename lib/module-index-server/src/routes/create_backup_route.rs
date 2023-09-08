use axum::{
    extract::Multipart,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, FixedOffset, Offset, Utc};
use hyper::StatusCode;
use module_index_client::ModuleDetailsResponse;
use s3::error::S3Error;
use sea_orm::{ActiveModelTrait, DbErr, Set};
use serde::{Deserialize, Serialize};
use si_pkg::{SiPkg, SiPkgError};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    extract::{Authorization, DbConnection, ExtractedS3Bucket},
    models::si_module::{self, ModuleResponseError},
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum CreateBackupError {
    #[error("db error: {0}")]
    DbErr(#[from] DbErr),
    #[error("file upload error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("module response error: {0}")]
    ModuleResponseError(#[from] ModuleResponseError),
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
impl IntoResponse for CreateBackupError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());
        dbg!(self);

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

// #[debug_handler]
pub async fn create_backup_route(
    Authorization {
        user_claim,
        auth_token: _auth_token,
    }: Authorization,
    ExtractedS3Bucket(s3_bucket): ExtractedS3Bucket,
    DbConnection(txn): DbConnection,
    mut multipart: Multipart,
) -> Result<Json<ModuleDetailsResponse>, CreateBackupError> {
    let field = match multipart.next_field().await.unwrap() {
        Some(f) => f,
        None => return Err(CreateBackupError::UploadRequiredError),
    };
    let data = field.bytes().await.unwrap();

    let loaded_package = dbg!(SiPkg::load_from_bytes(data.to_vec()))?;
    let package_metadata = dbg!(loaded_package.metadata())?;

    let s3_path = format!("backups/{}.sipkg", package_metadata.hash());

    let new_backup = si_module::ActiveModel {
        is_backup: Set(true),
        // TODO: do we want workspace ID as first class data?

        // name + description should probably be something like "Backup of Adam's Dev Worksapce"
        // we can set them here or count on it being set already in the package metadata?
        name: Set(package_metadata.name().to_owned()),
        description: Set(Some(package_metadata.description().to_owned())),
        owner_user_id: Set(user_claim.user_pk.to_string()),
        owner_display_name: Set(Some(package_metadata.created_by().to_owned())),
        latest_hash: Set(package_metadata.hash().to_string()),
        // maybe use db's `CLOCK_TIMESTAMP()`?
        latest_hash_created_at: Set(DateTime::<FixedOffset>::from_utc(
            Utc::now().naive_utc(),
            Utc.fix(),
        )),
        metadata: Set(serde_json::to_value(ExtraBackupMetadata {
            s3_path: s3_path.to_owned(),
        })?),
        ..Default::default() // all other attributes are `NotSet`
    };
    dbg!(&new_backup);

    // TODO: where do we want to put these? backups/{workspaceId}/{timestamp?}
    // upload to s3
    s3_bucket.put_object(s3_path, &data).await?;

    let new_backup: si_module::Model = new_backup.insert(&txn).await?;

    txn.commit().await?;

    Ok(dbg!(Json(new_backup.try_into()?)))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtraBackupMetadata {
    s3_path: String,
}
