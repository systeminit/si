use si_pkg::{FuncSpecBackendKind, FuncSpecBackendResponseType, SiPkgError, SpecError};
use std::convert::TryFrom;
use thiserror::Error;
use url::ParseError;

mod export;
mod import;

pub use export::export_pkg;
pub use import::{import_pkg, import_pkg_from_pkg};

use crate::{
    func::argument::FuncArgumentError, installed_pkg::InstalledPkgError, prop_tree::PropTreeError,
    schema::variant::definition::SchemaVariantDefinitionError, FuncBackendKind,
    FuncBackendResponseType, FuncError, FuncId, PropError, SchemaError, SchemaId,
    SchemaVariantError, SchemaVariantId, StandardModelError, ValidationPrototypeError,
};

#[derive(Debug, Error)]
pub enum PkgError {
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    Pkg(#[from] SiPkgError),
    #[error(transparent)]
    Prop(#[from] PropError),
    #[error(transparent)]
    Schema(#[from] SchemaError),
    #[error(transparent)]
    Func(#[from] FuncError),
    #[error(transparent)]
    FuncArgument(#[from] FuncArgumentError),
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
    #[error("Installed func id {0} does not exist")]
    InstalledFuncMissing(FuncId),
    #[error("Installed schema variant {0} does not exist")]
    InstalledSchemaVariantMissing(SchemaVariantId),
    #[error("standard model relationship {0} missing belongs_to for {1} with id {2}")]
    StandardModelMissingBelongsTo(&'static str, &'static str, String),
    #[error("standard model relationship {0} found multiple belongs_to for {1} with id {2}")]
    StandardModelMultipleBelongsTo(&'static str, &'static str, String),
    #[error(transparent)]
    UrlParse(#[from] ParseError),
    #[error("Cannot package func with backend kind of {0}")]
    InvalidFuncBackendKind(FuncBackendKind),
    #[error("Cannot package func with backend response type of {0}")]
    InvalidFuncBackendResponseType(FuncBackendResponseType),
    #[error("Package asked for a function with the unique id {0} but none could be found")]
    MissingFuncUniqueId(String),
    #[error("Func {0} missing from exported funcs")]
    MissingExportedFunc(FuncId),
    #[error("Leaf Function {0} has invalid argument {1}")]
    InvalidLeafArgument(FuncId, String),
    #[error("Validation Creation Error: {0}")]
    Validation(#[from] ValidationPrototypeError),
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

impl TryFrom<FuncBackendKind> for FuncSpecBackendKind {
    type Error = PkgError;

    fn try_from(value: FuncBackendKind) -> Result<Self, Self::Error> {
        Ok(match value {
            FuncBackendKind::JsAttribute => FuncSpecBackendKind::JsAttribute,
            FuncBackendKind::JsCommand => FuncSpecBackendKind::JsCommand,
            FuncBackendKind::Json => FuncSpecBackendKind::Json,
            FuncBackendKind::JsValidation => FuncSpecBackendKind::JsValidation,
            FuncBackendKind::JsWorkflow => FuncSpecBackendKind::JsWorkflow,
            _ => return Err(PkgError::InvalidFuncBackendKind(value)),
        })
    }
}

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

impl TryFrom<FuncBackendResponseType> for FuncSpecBackendResponseType {
    type Error = PkgError;

    fn try_from(value: FuncBackendResponseType) -> Result<Self, Self::Error> {
        Ok(match value {
            FuncBackendResponseType::Array => FuncSpecBackendResponseType::Array,
            FuncBackendResponseType::Boolean => FuncSpecBackendResponseType::Boolean,
            FuncBackendResponseType::CodeGeneration => FuncSpecBackendResponseType::CodeGeneration,
            FuncBackendResponseType::Command => FuncSpecBackendResponseType::Command,
            FuncBackendResponseType::Confirmation => FuncSpecBackendResponseType::Confirmation,
            FuncBackendResponseType::Integer => FuncSpecBackendResponseType::Integer,
            FuncBackendResponseType::Json => FuncSpecBackendResponseType::Json,
            FuncBackendResponseType::Map => FuncSpecBackendResponseType::Map,
            FuncBackendResponseType::Object => FuncSpecBackendResponseType::Object,
            FuncBackendResponseType::Qualification => FuncSpecBackendResponseType::Qualification,
            FuncBackendResponseType::String => FuncSpecBackendResponseType::String,
            FuncBackendResponseType::Validation => FuncSpecBackendResponseType::Validation,
            FuncBackendResponseType::Workflow => FuncSpecBackendResponseType::Workflow,

            _ => return Err(PkgError::InvalidFuncBackendResponseType(value)),
        })
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
