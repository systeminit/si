//! This module contains "builtin" objects that are included with System Initiative.
//! All submodules are private since the only entrypoint to this module should be the
//! [migrate()](crate::builtins::migrate()) function. However, they may have some functionality
//! exposed for "dev mode" use cases.

use std::collections::HashSet;
use telemetry::prelude::*;
use thiserror::Error;

use crate::func::argument::FuncArgumentError;
use crate::func::binding::FuncBindingError;
use crate::func::binding_return_value::FuncBindingReturnValueError;
use crate::provider::external::ExternalProviderError;
use crate::provider::internal::InternalProviderError;
use crate::schema::variant::definition::SchemaVariantDefinitionError;
use crate::schema::variant::SchemaVariantError;
use crate::socket::SocketError;
use crate::{
    ActionPrototypeError, AttributeContextBuilderError, AttributePrototypeArgumentError,
    AttributePrototypeError, AttributeReadContext, AttributeValueError, AttributeValueId,
    DalContext, ExternalProviderId, FuncError, InternalProviderId, PropError, PropId, SchemaError,
    SchemaVariantId, StandardModelError, TransactionsError, ValidationPrototypeError,
    WorkflowPrototypeError,
};

// Private builtins modules.
mod func;
pub mod schema;
mod workflow;

// Expose the "persist" function for creating and editing builtin funcs while in dev mode.
pub use func::persist as func_persist;

#[derive(Error, Debug)]
pub enum BuiltinsError {
    #[error("attribute context builder error: {0}")]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("attribute value not found by id: {0}")]
    AttributeValueNotFound(AttributeValueId),
    #[error("attribute value not found for attribute read context: {0:?}")]
    AttributeValueNotFoundForContext(AttributeReadContext),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("func binding return value error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("func not found in migration cache {0}")]
    FuncNotFoundInMigrationCache(&'static str),
    #[error("external provider error: {0}")]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("implicit internal provider not found for prop: {0}")]
    ImplicitInternalProviderNotFoundForProp(PropId),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("missing attribute prototype for attribute value")]
    MissingAttributePrototypeForAttributeValue,
    #[error("missing attribute prototype for explicit internal provider: {0}")]
    MissingAttributePrototypeForExplicitInternalProvider(InternalProviderId),
    #[error("missing attribute prototype for external provider: {0}")]
    MissingAttributePrototypeForExternalProvider(ExternalProviderId),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("prop not bound by id: {0}")]
    PropNotFound(PropId),
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("json error {1} at file {0}")]
    FuncJson(String, serde_json::Error),
    #[error("encountered serde json error for func ({0}): {1}")]
    SerdeJsonErrorForFunc(String, serde_json::Error),
    #[error("socket error: {0}")]
    Socket(#[from] SocketError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("validation prototype error: {0}")]
    ValidationPrototype(#[from] ValidationPrototypeError),
    #[error("Filesystem IO error: {0}")]
    FilesystemIO(#[from] std::io::Error),
    #[error("Regex parsing error: {0}")]
    Regex(#[from] regex::Error),
    #[error(transparent)]
    WorkflowPrototype(#[from] WorkflowPrototypeError),
    #[error("Func Metadata error: {0}")]
    FuncMetadata(String),
    #[error("builtin {0} missing func argument {1}")]
    BuiltinMissingFuncArgument(String, String),
    #[error("prop cache not found: {0}")]
    PropCacheNotFound(SchemaVariantId),
    #[error("explicit internal provider not found by name: {0}")]
    ExplicitInternalProviderNotFound(String),
    #[error("external provider not found by name: {0}")]
    ExternalProviderNotFound(String),
    #[error("schema variant definition error")]
    SchemaVariantDefinition(#[from] SchemaVariantDefinitionError),
    #[error("error creating new transactions")]
    Transactions(#[from] TransactionsError),
}

pub type BuiltinsResult<T> = Result<T, BuiltinsError>;

/// This enum drives what builtin [`Schemas`](crate::Schema) to migrate for tests.
///
/// This enum _should not_ be used outside of tests!
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

/// Migrate all "builtins" in a definitive order.
///
/// 1. [`Funcs`](crate::Func)
/// 1. [`WorkflowPrototypes`](crate::workflow_prototype::WorkflowPrototype)
/// 1. [`Schemas`](crate::Schema)
pub async fn migrate(
    ctx: &DalContext,
    selected_test_builtin_schemas: Option<SelectedTestBuiltinSchemas>,
) -> BuiltinsResult<()> {
    info!("migrating functions");
    func::migrate(ctx).await?;

    info!("migrating workflows");
    workflow::migrate(ctx).await?;

    match selected_test_builtin_schemas {
        Some(found_selected_test_builtin_schemas) => {
            schema::migrate_for_tests(ctx, found_selected_test_builtin_schemas).await?;
        }
        None => {
            schema::migrate_for_production(ctx).await?;

            func::migrate_command_prototypes(ctx).await?;
        }
    }

    info!("completed migrating functions, workflows and schemas");
    Ok(())
}
