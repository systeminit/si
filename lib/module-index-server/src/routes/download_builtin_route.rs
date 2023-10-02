use axum::{
    extract::Path,
    response::{IntoResponse, Redirect, Response},
    Json,
};
use hyper::StatusCode;
use s3::error::S3Error;
use sea_orm::{DbErr, EntityTrait};
use thiserror::Error;

use crate::{
    extract::{DbConnection, ExtractedS3Bucket},
    models::si_module::{self, ModuleId},
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum DownloadBuiltinError {
    #[error("db error: {0}")]
    DbErr(#[from] DbErr),
    #[error(r#"Module "{0}" is not a builtin and requires authentication"#)]
    NotBuiltin(ModuleId),
    #[error(r#"Module "{0}" not found"#)]
    NotFound(ModuleId),
    #[error("s3 error: {0}")]
    S3Error(#[from] S3Error),
}

// TODO: figure out how to not keep this serialization logic here
impl IntoResponse for DownloadBuiltinError {
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

pub async fn download_builtin_route(
    Path(module_id): Path<ModuleId>,
    ExtractedS3Bucket(s3_bucket): ExtractedS3Bucket,
    DbConnection(txn): DbConnection,
) -> Result<Redirect, DownloadBuiltinError> {
    let module = match si_module::Entity::find_by_id(module_id).one(&txn).await? {
        Some(module) => module,
        _ => return Err(DownloadBuiltinError::NotFound(module_id)),
    };

    if module.is_builtin_at.is_none() {
        return Err(DownloadBuiltinError::NotBuiltin(module_id));
    }

    let download_url =
        s3_bucket.presign_get(format!("{}.sipkg", module.latest_hash), 60 * 5, None)?;

    Ok(Redirect::temporary(&download_url))
}
