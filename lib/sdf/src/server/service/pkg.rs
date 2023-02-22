use crate::server::{
    extract::{AccessBuilder, HandlerContext},
    impl_default_error_into_response,
};
use axum::{extract::Query, response::Response, routing::get, Json, Router};
use dal::{StandardModelError, TenancyError, TransactionsError, Visibility};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs::read_dir;

#[derive(Error, Debug)]
pub enum PkgError {
    #[error(transparent)]
    Pg(#[from] si_data_pg::PgError),
    #[error(transparent)]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error("tenancy error: {0}")]
    Tenancy(#[from] TenancyError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    ContextTransaction(#[from] TransactionsError),
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("No packages path provided")]
    NoPackagesPath,
}

type PkgResult<T> = Result<T, PkgError>;

impl_default_error_into_response!(PkgError);

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PkgView {
    name: String,
    // other metadata to come next
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PkgListRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PkgListResponse {
    pub pkgs: Vec<PkgView>,
}

pub async fn list_pkg_dir_entries(pkgs_path: &PathBuf) -> PkgResult<Vec<String>> {
    let mut result = vec![];
    let mut entries = read_dir(pkgs_path).await?;
    while let Some(entry) = entries.next_entry().await? {
        result.push(entry.file_name().to_string_lossy().to_string())
    }

    Ok(result)
}

pub async fn list_pkgs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(_request_ctx): AccessBuilder,
    Query(_request): Query<PkgListRequest>,
) -> PkgResult<Json<PkgListResponse>> {
    let pkgs_path = match builder.pkgs_path().await {
        None => return Err(PkgError::NoPackagesPath),
        Some(path) => path,
    };

    let pkgs = list_pkg_dir_entries(pkgs_path)
        .await?
        .iter()
        .map(|name| PkgView {
            name: name.to_owned(),
        })
        .collect();

    Ok(Json(PkgListResponse { pkgs }))
}

pub fn routes() -> Router {
    Router::new().route("/list_pkgs", get(list_pkgs))
}
