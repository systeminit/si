use si_pkg::{FuncSpecBackendKind, FuncSpecBackendResponseType, SiPkgError, SpecError};
use thiserror::Error;
use url::ParseError;

mod export;
mod import;

pub use export::export_pkg;
pub use import::{import_pkg, import_pkg_from_pkg};

use crate::{
    installed_pkg::InstalledPkgError, prop_tree::PropTreeError,
    schema::variant::definition::SchemaVariantDefinitionError, FuncBackendKind,
    FuncBackendResponseType, FuncError, PropError, SchemaError, SchemaId, SchemaVariantError,
    SchemaVariantId, StandardModelError,
};

#[derive(Debug, Error)]
pub enum PkgError {
    #[error(transparent)]
    Pkg(#[from] SiPkgError),
    #[error(transparent)]
    Prop(#[from] PropError),
    #[error(transparent)]
    Schema(#[from] SchemaError),
    #[error(transparent)]
    Func(#[from] FuncError),
    #[error(transparent)]
    SchemaVariant(#[from] SchemaVariantError),
    #[error(transparent)]
    SchemaVariantDefinition(#[from] SchemaVariantDefinitionError),
    #[error(transparent)]
    PkgSpec(#[from] SpecError),
    #[error("prop spec structure is invalid: {0}")]
    PropSpecChildrenInvalid(String),
    #[error(transparent)]
    PropTree(#[from] PropTreeError),
    #[error("prop tree structure is invalid: {0}")]
    PropTreeInvalid(String),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("Package with that hash already installed: {0}")]
    PackageAlreadyInstalled(String),
    #[error(transparent)]
    InstalledPkg(#[from] InstalledPkgError),
    #[error("Installed schema id {0} does not exist")]
    InstalledSchemaMissing(SchemaId),
    #[error("Installed schema variant {0} does not exist")]
    InstalledSchemaVariantMissing(SchemaVariantId),
    #[error("standard model relationship {0} missing belongs_to for {1} with id {2}")]
    StandardModelMissingBelongsTo(&'static str, &'static str, String),
    #[error("standard model relationship {0} found multiple belongs_to for {1} with id {2}")]
    StandardModelMultipleBelongsTo(&'static str, &'static str, String),
    #[error(transparent)]
    UrlParse(#[from] ParseError),
}

impl PkgError {
    fn prop_tree_invalid(message: impl Into<String>) -> Self {
        Self::PropTreeInvalid(message.into())
    }

    fn prop_spec_children_invalid(message: impl Into<String>) -> Self {
        Self::PropSpecChildrenInvalid(message.into())
    }
}

pub type PkgResult<T> = Result<T, PkgError>;

impl From<FuncSpecBackendKind> for FuncBackendKind {
    fn from(value: FuncSpecBackendKind) -> Self {
        match value {
            FuncSpecBackendKind::JsAttribute => FuncBackendKind::JsAttribute,
            FuncSpecBackendKind::JsCommand => FuncBackendKind::JsCommand,
            FuncSpecBackendKind::Json => FuncBackendKind::Json,
            FuncSpecBackendKind::JsValidation => FuncBackendKind::JsValidation,
            FuncSpecBackendKind::JsWorkflow => FuncBackendKind::JsWorkflow,
        }
    }
}

impl From<FuncSpecBackendResponseType> for FuncBackendResponseType {
    fn from(value: FuncSpecBackendResponseType) -> Self {
        match value {
            FuncSpecBackendResponseType::Array => FuncBackendResponseType::Array,
            FuncSpecBackendResponseType::Boolean => FuncBackendResponseType::Boolean,
            FuncSpecBackendResponseType::CodeGeneration => FuncBackendResponseType::CodeGeneration,
            FuncSpecBackendResponseType::Command => FuncBackendResponseType::Command,
            FuncSpecBackendResponseType::Confirmation => FuncBackendResponseType::Confirmation,
            FuncSpecBackendResponseType::Integer => FuncBackendResponseType::Integer,
            FuncSpecBackendResponseType::Json => FuncBackendResponseType::Json,
            FuncSpecBackendResponseType::Map => FuncBackendResponseType::Map,
            FuncSpecBackendResponseType::Object => FuncBackendResponseType::Object,
            FuncSpecBackendResponseType::Qualification => FuncBackendResponseType::Qualification,
            FuncSpecBackendResponseType::String => FuncBackendResponseType::String,
            FuncSpecBackendResponseType::Validation => FuncBackendResponseType::Validation,
            FuncSpecBackendResponseType::Workflow => FuncBackendResponseType::Workflow,
        }
    }
}
