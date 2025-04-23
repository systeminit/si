//! This module contains "builtin" objects that are included with System Initiative.
//! All submodules are private since the only entrypoint to this module should be the
//! [migrate()](crate::builtins::migrate_local()) function. However, they may have some functionality
//! exposed for "dev mode" use cases.

use si_pkg::{
    SiPkgError,
    SpecError,
};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    AttributeValueId,
    PropId,
    SchemaVariantError,
    SchemaVariantId,
    StandardModelError,
    TransactionsError,
    action::prototype::ActionPrototypeError,
    func::{
        FuncError,
        argument::FuncArgumentError,
    },
    module::ModuleError,
    pkg::PkgError,
};

pub mod func;
pub mod schema;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum BuiltinsError {
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("attribute value not found by id: {0}")]
    AttributeValueNotFound(AttributeValueId),
    #[error("builtin {0} missing func argument {1}")]
    BuiltinMissingFuncArgument(String, String),
    #[error("func error")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("json error {1} at file {0}")]
    FuncJson(String, serde_json::Error),
    #[error("func metadata error: {0}")]
    FuncMetadata(String),
    #[error("func not found in migration cache {0}")]
    FuncNotFoundInMigrationCache(&'static str),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("missing attribute prototype for attribute value")]
    MissingAttributePrototypeForAttributeValue,
    #[error("no packages path configured")]
    MissingPkgsPath,
    #[error("module error: {0}")]
    Module(#[from] ModuleError),
    #[error("pkg error: {0}")]
    Pkg(#[from] PkgError),
    #[error("prop cache not found: {0}")]
    PropCacheNotFound(SchemaVariantId),
    #[error("prop not bound by id: {0}")]
    PropNotFound(PropId),
    #[error("regex parsing error: {0}")]
    Regex(#[from] regex::Error),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("encountered serde json error for func ({0}): {1}")]
    SerdeJsonErrorForFunc(String, serde_json::Error),
    #[error("si pkg error: {0}")]
    SiPkg(#[from] SiPkgError),
    #[error("spec error: {0}")]
    Spec(#[from] SpecError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("error creating new transactions")]
    Transactions(#[from] TransactionsError),
}

pub type BuiltinsResult<T> = Result<T, BuiltinsError>;
