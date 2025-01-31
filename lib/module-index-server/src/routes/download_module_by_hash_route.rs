use axum::{
    extract::Path,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use hyper::StatusCode;
use s3::error::S3Error;
use sea_orm::DbErr;
use thiserror::Error;

use crate::extract::{Authorization, ExtractedS3Bucket};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum DownloadModuleError {
    #[error("db error: {0}")]
    DbErr(#[from] DbErr),
    #[error("s3 error: {0}")]
    S3Error(#[from] S3Error),
}

// TODO: figure out how to not keep this serialization logic here
impl IntoResponse for DownloadModuleError {
    fn into_response(self) -> Response {
        let (status, error_message) = (StatusCode::INTERNAL_SERVER_ERROR, self.to_string());

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

pub async fn download_module_by_hash_route(
    Path(hash): Path<String>,
    Authorization { .. }: Authorization,
    ExtractedS3Bucket(s3_bucket): ExtractedS3Bucket,
) -> Result<Redirect, DownloadModuleError> {
    let download_url = s3_bucket
        .presign_get(format!("{}.sipkg", hash), 60 * 5, None)
        .await?;

    Ok(Redirect::temporary(&download_url))
}
