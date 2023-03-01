use crate::server::impl_default_error_into_response;
use axum::{
    response::Response,
    routing::{get, post},
    Json, Router,
};
use dal::{
    installed_pkg::InstalledPkgError, pkg::PkgError as DalPkgError, DalContextBuilder,
    StandardModelError, TenancyError, TransactionsError, WsEventError,
};
use serde::{Deserialize, Serialize};
use si_pkg::SiPkgError;
use si_settings::{safe_canonically_join, CanonicalFileError};
use std::path::{Path, PathBuf};
use thiserror::Error;
use tokio::fs::read_dir;

pub mod install_pkg;
pub mod list_pkgs;

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
    #[error("could not publish websocket event: {0}")]
    WsEvent(#[from] WsEventError),
    #[error("No packages path provided")]
    NoPackagesPath,
    #[error("Could not canononicalize path: {0}")]
    Canononicalize(#[from] CanonicalFileError),
    #[error("Package could not be found: {0}")]
    PackageNotFound(String),
    #[error("Package with that name already installed: {0}")]
    PackageAlreadyInstalled(String),
    // add error for matching hash
    #[error(transparent)]
    InstalledPkg(#[from] InstalledPkgError),
    #[error(transparent)]
    SiPkg(#[from] SiPkgError),
    #[error(transparent)]
    DalPkg(#[from] DalPkgError),
}

pub type PkgResult<T> = Result<T, PkgError>;

impl_default_error_into_response!(PkgError);

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PkgView {
    name: String,
    installed: bool,
}

pub async fn get_pkgs_path(builder: &DalContextBuilder) -> PkgResult<&PathBuf> {
    match builder.pkgs_path().await {
        None => Err(PkgError::NoPackagesPath),
        Some(path) => Ok(path),
    }
}

pub async fn list_pkg_dir_entries(pkgs_path: &Path) -> PkgResult<Vec<String>> {
    let mut result = vec![];
    let mut entries = read_dir(pkgs_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        result.push(entry.file_name().to_string_lossy().to_string())
    }

    Ok(result)
}

pub async fn pkg_lookup(pkgs_path: &Path, name: &str) -> PkgResult<Option<PathBuf>> {
    let real_pkg_path = safe_canonically_join(pkgs_path, name)?;
    let file_name = real_pkg_path
        .file_name()
        .map(|file_name| file_name.to_string_lossy().to_string());

    Ok((real_pkg_path.is_file() && file_name.is_some()).then_some(real_pkg_path))
}

pub fn routes() -> Router {
    Router::new()
        .route("/list_pkgs", get(list_pkgs::list_pkgs))
        .route("/install_pkg", post(install_pkg::install_pkg))
}
