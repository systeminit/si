//! This module contains "builtin" objects that are included with System Initiative.
//! The submodules are private since the only entrypoint to this module should be the
//! [migrate()](crate::builtins::migrate()) function.

use thiserror::Error;

use crate::func::binding::FuncBindingError;
use crate::func::binding_return_value::FuncBindingReturnValueError;
use crate::provider::external::ExternalProviderError;
use crate::provider::internal::InternalProviderError;
use crate::schema::variant::SchemaVariantError;
use crate::socket::SocketError;
use crate::{
    AttributeContextBuilderError, AttributePrototypeArgumentError, AttributePrototypeError,
    AttributeValueError, CodeGenerationPrototypeError, DalContext, FuncError, PropError, PropId,
    QualificationPrototypeError, ResourcePrototypeError, SchemaError, StandardModelError,
    ValidationPrototypeError,
};

mod func;
mod schema;

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
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("qualification prototype error: {0}")]
    QualificationPrototype(#[from] QualificationPrototypeError),
    #[error("resource prototype error: {0}")]
    ResourcePrototype(#[from] ResourcePrototypeError),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("socket error: {0}")]
    Socket(#[from] SocketError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("validation prototype error: {0}")]
    ValidationPrototype(#[from] ValidationPrototypeError),
}

pub type BuiltinsResult<T> = Result<T, BuiltinsError>;

/// Migrate all "builtin" [`Funcs`](crate::Func) and [`Schemas`](crate::Schema) (in that order).
pub async fn migrate(ctx: &DalContext<'_, '_>) -> BuiltinsResult<()> {
    func::migrate(ctx).await?;
    schema::migrate(ctx).await?;
    Ok(())
}
