use crate::{
    extract::{Authorization, DbConnection, UserClaim},
    models::si_module,
};
use axum::{
    debug_handler,
    extract::Multipart,
    extract::Query,
    response::{IntoResponse, Response},
    Json,
};
use chrono::{DateTime, FixedOffset, Local, Offset, TimeZone, Utc};
use hyper::StatusCode;
use sea_orm::{ActiveModelTrait, ActiveValue::NotSet, DbErr, InsertResult, Set};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use si_pkg::{SiPkg, SiPkgError};
use thiserror::Error;
use tokio::fs;
use ulid::Ulid;

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
    #[error("module parsing error: {0}")]
    SiPkgError(#[from] SiPkgError),
    #[error("upload is required")]
    UploadRequiredError,
}

// TODO: figure out how to not keep this serialization logic here
impl IntoResponse for UpsertModuleError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        let body = Json(
            serde_json::json!({ "error": { "message": error_message, "code": 42, "statusCode": status.as_u16() } }),
        );

        (status, body).into_response()
    }
}

// #[debug_handler]
pub async fn upsert_module_route(
    // Authorization(claim): Authorization,
    DbConnection(txn): DbConnection,
    mut multipart: Multipart,
) -> Result<Json<Value>, UpsertModuleError> {
    let field = match multipart.next_field().await.unwrap() {
        Some(f) => f,
        None => return Err(UpsertModuleError::UploadRequiredError),
    };
    let name = field.name().unwrap().to_string();
    let data = field.bytes().await.unwrap();

    let temp_path = std::env::temp_dir().join("tmp-package.sipkg");
    fs::write(&temp_path, data).await?;

    let pkg_info = SiPkg::load_from_file(temp_path).await?;
    let pkg_metadata = pkg_info.metadata()?;

    let new_module = si_module::ActiveModel {
        name: Set(pkg_metadata.name().to_owned()),
        description: Set(Some(pkg_metadata.description().to_owned())),
        // owner_user_id: Set(claim.user_pk.to_string()),
        owner_user_id: Set(Ulid::new().to_string()),
        owner_display_name: Set(Some(pkg_metadata.created_by().to_owned())),
        latest_hash: Set(pkg_metadata.hash().to_string()),
        // maybe use db's `CLOCK_TIMESTAMP()`?
        latest_hash_created_at: Set(DateTime::<FixedOffset>::from_utc(
            Utc::now().naive_utc(),
            Utc.fix(),
        )),

        ..Default::default() // all other attributes are `NotSet`
    };

    let new_module: si_module::Model = new_module.insert(&txn).await?;

    txn.commit().await?;

    Ok(Json(json!({ "metadata": format!("{:?}", new_module) })))
}
