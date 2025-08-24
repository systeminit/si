use std::path::{
    Path,
    PathBuf,
};

use axum::{
    Router,
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    routing::post,
};
use convert_case::{
    Case,
    Casing,
};
use dal::{
    ChangeSetError,
    DalContextBuilder,
    FuncError,
    SchemaError,
    SchemaId,
    SchemaVariantError,
    SchemaVariantId,
    TransactionsError,
    WorkspaceError,
    WorkspacePk,
    WorkspaceSnapshotError,
    WsEventError,
    pkg::PkgError as DalPkgError,
};
use sdf_core::api_error::ApiError;
use si_layer_cache::LayerDbError;
use si_pkg::{
    SiPkg,
    SiPkgError,
};
use si_std::{
    CanonicalFileError,
    canonical_file::safe_canonically_join,
};
use telemetry::prelude::*;
use thiserror::Error;
use tokio::fs::read_dir;
use ulid::Ulid;

pub mod approval_process;
pub mod import_workspace_vote;
pub mod install_module;
pub mod upgrade_modules;

const PKG_EXTENSION: &str = "sipkg";
const MAX_NAME_SEARCH_ATTEMPTS: usize = 100;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ModuleError {
    #[error("Could not canonicalize path: {0}")]
    Canonicalize(#[from] CanonicalFileError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("dal cached module error: {0}")]
    DalCachedModule(#[from] dal::cached_module::CachedModuleError),
    #[error("dal pkg error: {0}")]
    DalPkg(#[from] DalPkgError),
    #[error("Trying to export from/import into root tenancy")]
    ExportingImportingWithRootTenancy,
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("hyper http error: {0}")]
    Hyper(#[from] hyper::http::Error),
    #[error("Invalid package file name: {0}")]
    InvalidPackageFileName(String),
    #[error("invalid user system init")]
    InvalidUserSystemInit,
    #[error("IO Error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("LayerDb error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("dal module error: {0}")]
    Module(#[from] dal::module::ModuleError),
    #[error("Module hash not be found: {0}")]
    ModuleHashNotFound(String),
    #[error("Module index: {0}")]
    ModuleIndex(#[from] module_index_client::ModuleIndexClientError),
    #[error("Module index not configured")]
    ModuleIndexNotConfigured,
    #[error("No packages path provided")]
    NoPackagesPath,
    #[error("Package with that name already installed: {0}")]
    PackageAlreadyInstalled(String),
    #[error("That package already exists: {0}")]
    PackageAlreadyOnDisk(String),
    #[error("No schema variants added to package export")]
    PackageExportEmpty,
    #[error("Package name required")]
    PackageNameEmpty,
    #[error("Package could not be found: {0}")]
    PackageNotFound(String),
    #[error("Package version required")]
    PackageVersionEmpty,
    #[error("pg error: {0}")]
    Pg(#[from] si_data_pg::PgError),
    #[error("pg pool error: {0}")]
    PgPool(#[from] si_data_pg::PgPoolError),
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema not found for variant {0}")]
    SchemaNotFoundForVariant(SchemaVariantId),
    #[error("schema install pkg result empty: {0}")]
    SchemaNotFoundFromInstall(Ulid),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("schema variant not found {0}")]
    SchemaVariantNotFound(SchemaVariantId),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error("si pkg error: {0}")]
    SiPkg(#[from] SiPkgError),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("ulid decode error: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
    #[error(
        "found an unlocked schema variant (schema: {1}) for a module to be installed (module: {0})"
    )]
    UnlockedSchemaVariantForModuleToInstall(String, SchemaId),
    #[error("Unable to parse URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("Could not find current workspace {0}")]
    WorkspaceNotFound(WorkspacePk),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error("could not publish websocket event: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type ModuleResult<T> = Result<T, ModuleError>;

impl IntoResponse for ModuleError {
    fn into_response(self) -> Response {
        let (status_code, error_message) = match self {
            ModuleError::ModuleHashNotFound(_)
            | ModuleError::PackageNotFound(_)
            | ModuleError::SchemaNotFoundForVariant(_)
            | ModuleError::SchemaVariantNotFound(_)
            | ModuleError::WorkspaceNotFound(_) => (StatusCode::NOT_FOUND, self.to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()),
        };

        ApiError::new(status_code, error_message).into_response()
    }
}

#[derive(serde::Deserialize, serde::Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PkgView {
    name: String,
    installed: bool,
    hash: Option<String>,
}

pub async fn get_pkgs_path(builder: &DalContextBuilder) -> ModuleResult<&PathBuf> {
    match builder.pkgs_path().await {
        None => Err(ModuleError::NoPackagesPath),
        Some(path) => Ok(path),
    }
}

pub async fn list_pkg_dir_entries(pkgs_path: &Path) -> ModuleResult<Vec<String>> {
    let mut result = vec![];
    let mut entries = read_dir(pkgs_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        result.push(entry.file_name().to_string_lossy().to_string())
    }

    Ok(result)
}

pub async fn pkg_lookup(
    pkgs_path: &Path,
    name: &str,
) -> ModuleResult<(Option<PathBuf>, Option<String>)> {
    let real_pkg_path = safe_canonically_join(pkgs_path, name)?;
    let file_name = real_pkg_path
        .file_name()
        .map(|file_name| file_name.to_string_lossy().to_string());

    Ok((real_pkg_path.is_file().then_some(real_pkg_path), file_name))
}

fn add_pkg_extension(to: &str, version: &str, attempts: usize) -> String {
    match attempts {
        0 => format!("{to}-{version}.{PKG_EXTENSION}"),
        more_than_zero => format!("{to}-{version}-{more_than_zero}.{PKG_EXTENSION}"),
    }
}

/// Generate a file name automatically based on the package name, appending numbers to the name if
/// it conflicts with a file already on disk.
pub async fn get_new_pkg_path(
    builder: &DalContextBuilder,
    name: &str,
    version: &str,
) -> ModuleResult<PathBuf> {
    let name_kebabed = name.to_case(Case::Kebab);
    let version_kebabed = version.to_case(Case::Kebab);

    let mut attempts = 0;
    loop {
        let file_name = add_pkg_extension(&name_kebabed, &version_kebabed, attempts);

        let real_pkg_path = match Path::new(&file_name).file_name() {
            None => return Err(ModuleError::InvalidPackageFileName(file_name)),
            Some(file_name) => Path::join(get_pkgs_path(builder).await?, file_name),
        };

        if attempts > MAX_NAME_SEARCH_ATTEMPTS {
            return Err(ModuleError::PackageAlreadyOnDisk(
                real_pkg_path.to_string_lossy().to_string(),
            ));
        } else if real_pkg_path.is_file() {
            attempts = 1;
            continue;
        }

        return Ok(real_pkg_path);
    }
}

pub async fn pkg_open(builder: &DalContextBuilder, file_name: &str) -> ModuleResult<SiPkg> {
    let pkg_tuple = pkg_lookup(get_pkgs_path(builder).await?, file_name).await?;

    let real_pkg_path = match pkg_tuple {
        (None, _) => return Err(ModuleError::PackageNotFound(file_name.to_string())),
        (Some(real_pkg_path), _) => real_pkg_path,
    };

    Ok(SiPkg::load_from_file(&real_pkg_path).await?)
}

pub fn routes() -> Router<sdf_core::app_state::AppState> {
    Router::new()
        .route("/install_module", post(install_module::install_module)) // USED IN CUSTOMIZE SCREEN
        .route("/upgrade_modules", post(upgrade_modules::upgrade_modules)) // USED IN CUSTOMIZE SCREEN
        .route(
            "/begin_approval_process", // USED IN CUSTOMIZE SCREEN
            post(approval_process::begin_approval_process),
        )
        .route(
            "/cancel_approval_process", // USED IN CUSTOMIZE SCREEN
            post(approval_process::cancel_approval_process),
        )
        .route(
            "/import_workspace_vote", // USED IN CUSTOMIZE SCREEN
            post(import_workspace_vote::import_workspace_vote),
        )
}
