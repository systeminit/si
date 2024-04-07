//! This module contains "builtin" objects that are included with System Initiative.
//! All submodules are private since the only entrypoint to this module should be the
//! [migrate()](crate::builtins::migrate_local()) function. However, they may have some functionality
//! exposed for "dev mode" use cases.

use std::collections::HashSet;
use telemetry::prelude::*;
use thiserror::Error;

use si_pkg::{SiPkgError, SpecError};

use crate::func::FuncError;
use crate::module::ModuleError;
use crate::pkg::PkgError;
use crate::{AttributeValueId, PropId, SchemaVariantId, StandardModelError, TransactionsError};

// Private builtins modules.
pub mod func;
pub mod schema;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum BuiltinsError {
    #[error("attribute value not found by id: {0}")]
    AttributeValueNotFound(AttributeValueId),
    #[error("builtin {0} missing func argument {1}")]
    BuiltinMissingFuncArgument(String, String),
    #[error("Filesystem IO error: {0}")]
    FilesystemIO(#[from] std::io::Error),
    #[error(transparent)]
    Func(#[from] FuncError),
    #[error("json error {1} at file {0}")]
    FuncJson(String, serde_json::Error),
    #[error("Func Metadata error: {0}")]
    FuncMetadata(String),
    #[error("func not found in migration cache {0}")]
    FuncNotFoundInMigrationCache(&'static str),
    #[error("missing attribute prototype for attribute value")]
    MissingAttributePrototypeForAttributeValue,
    #[error("no packages path configured")]
    MissingPkgsPath,
    #[error(transparent)]
    Module(#[from] ModuleError),
    #[error(transparent)]
    Pkg(#[from] PkgError),
    #[error("prop cache not found: {0}")]
    PropCacheNotFound(SchemaVariantId),
    #[error("prop not bound by id: {0}")]
    PropNotFound(PropId),
    #[error("Regex parsing error: {0}")]
    Regex(#[from] regex::Error),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("encountered serde json error for func ({0}): {1}")]
    SerdeJsonErrorForFunc(String, serde_json::Error),
    #[error(transparent)]
    SiPkg(#[from] SiPkgError),
    #[error(transparent)]
    Spec(#[from] SpecError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("error creating new transactions")]
    Transactions(#[from] TransactionsError),
}

pub type BuiltinsResult<T> = Result<T, BuiltinsError>;

/// This enum drives what builtin [`Schemas`](crate::Schema) to migrate for tests.
///
/// This enum _should not_ be used outside of tests!
#[remain::sorted]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SelectedTestBuiltinSchemas {
    /// Migrate everything (default behavior).
    All,
    /// Migrate nothing.
    None,
    /// Migrate _some_ [`Schema(s)`](crate::Schema) based on user input.
    Some(HashSet<String>),
    /// Migrate _only_ test-exclusive [`Schemas`](crate::Schema).
    Test,
}
