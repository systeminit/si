//! This module contains "builtin" objects that are included with System Initiative.
//! All submodules are private since the only entrypoint to this module should be the
//! [migrate()](crate::builtins::migrate()) function. However, they may have some functionality
//! exposed for "dev mode" use cases.

use thiserror::Error;

use crate::func::binding::FuncBindingError;
use crate::func::binding_return_value::FuncBindingReturnValueError;
use crate::provider::external::ExternalProviderError;
use crate::provider::internal::InternalProviderError;
use crate::schema::variant::SchemaVariantError;
use crate::socket::SocketError;
use crate::{
    AttributeContextBuilderError, AttributePrototypeArgumentError, AttributePrototypeError,
    AttributeReadContext, AttributeValueError, AttributeValueId, CodeGenerationPrototypeError,
    DalContext, ExternalProviderId, FuncError, PropError, PropId, PropKind,
    QualificationPrototypeError, SchemaError, StandardModelError, ValidationPrototypeError,
    WorkflowPrototypeError,
};

mod func;
mod schema;
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
    #[error("attribute value not found for attribute read context: {0:?}")]
    AttributeValueNotFoundForContext(AttributeReadContext),
    #[error("no parent found for attribute value: {0}")]
    AttributeValueDoesNotHaveParent(AttributeValueId),
    #[error("code generation prototype error: {0}")]
    CodeGenerationPrototype(#[from] CodeGenerationPrototypeError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("func binding return value error: {0}")]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error("external provider error: {0}")]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("implicit internal provider not found for prop: {0}")]
    ImplicitInternalProviderNotFoundForProp(PropId),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("missing attribute prototype for attribute value")]
    MissingAttributePrototypeForAttributeValue,
    #[error("missing attribute prototype for external provider id: {0}")]
    MissingAttributePrototypeForExternalProvider(ExternalProviderId),
    #[error("expected primitive prop kind (string, boolean, integer), found {0}")]
    NonPrimitivePropKind(PropKind),
    #[error("parent prop kind is not \"Object\", which is required for setting default values on props (found {0})")]
    ParentPropIsNotObjectForPropWithDefaultValue(PropKind),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("prop not bound by id: {0}")]
    PropNotFound(PropId),
    #[error("qualification prototype error: {0}")]
    QualificationPrototype(#[from] QualificationPrototypeError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
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
}

pub type BuiltinsResult<T> = Result<T, BuiltinsError>;

/// Migrate all "builtins" in a definitive order.
///
/// 1. [`Funcs`](crate::Func)
/// 1. [`WorkflowPrototypes`](crate::workflow_prototype::WorkflowPrototype)
/// 1. [`Schemas`](crate::Schema)
pub async fn migrate(ctx: &DalContext) -> BuiltinsResult<()> {
    func::migrate(ctx).await?;
    workflow::migrate(ctx).await?;
    schema::migrate(ctx).await?;
    Ok(())
}
