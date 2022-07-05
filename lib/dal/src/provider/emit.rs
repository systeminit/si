//! This module provides the [`emit()`] function for [`InternalProviders`](crate::InternalProvider)
//! and [`ExternalProviders`](crate::ExternalProvider) to use. This module should be private since
//! it should only be used by the providers.

use telemetry::prelude::*;
use thiserror::Error;

use crate::func::backend::identity::FuncBackendIdentityArgs;
use crate::func::binding::FuncBindingError;
use crate::func::FuncId;
use crate::{
    AttributeContext, AttributeContextError, AttributeValue, DalContext, Func, FuncBinding,
};
use crate::{
    AttributePrototype, AttributePrototypeId, AttributeReadContext, AttributeValueError,
    AttributeView, FuncBackendKind, StandardModel, StandardModelError,
};

#[derive(Error, Debug)]
pub enum EmitError {
    #[error("attribute context error: {0}")]
    AttributeContext(#[from] AttributeContextError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("serde_json error: {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),

    #[error("attribute prototype not found for id: {0}")]
    AttributePrototypeNotFound(AttributePrototypeId),
    #[error("func not found for id: {0}")]
    FuncNotFound(FuncId),
    #[error("missing attribute value")]
    MissingAttributeValue,
}

pub type EmitResult<T> = Result<T, EmitError>;

/// Perform the "emit" for both [`InternalProviders`](crate::InternalProvider) and
/// [`ExternalProviders`](crate::ExternalProvider). Two [`AttributeContexts`](crate::AttributeContext)
/// are required: one to find the [`AttributeValue`](crate::AttributeValue) to "consume" another to
/// find or create an [`AttributeValue`](crate::AttributeValue) to "emit".
///
/// _This function will likely be unused for explicit
/// [`InternalProviders`](crate::InternalProvider)
/// and [`ExternalProviders`](crate::ExternalProvider) until a formal job queue is implemented
/// instead of the work queue for
/// [`AttributeValue::update_dependent_attribute_values()`](crate::AttributeValue::update_dependent_attribute_values())_.
pub async fn emit(
    ctx: &DalContext<'_, '_>,
    attribute_prototype_id: AttributePrototypeId,
    consume_attribute_context: AttributeContext,
    emit_attribute_context: AttributeContext,
) -> EmitResult<AttributeValue> {
    // Assemble the raw value required for the "emit" attribute value. We only need to generate
    // a view if the least specific field set on the consume attribute context is a "PropId".
    let found_attribute_value =
        AttributeValue::find_for_context(ctx, consume_attribute_context.into())
            .await?
            .ok_or(EmitError::MissingAttributeValue)?;

    let payload = if consume_attribute_context.is_least_specific_field_kind_prop()? {
        let found_attribute_view_context = AttributeReadContext {
            prop_id: None,
            ..AttributeReadContext::from(consume_attribute_context)
        };
        let found_attribute_view = AttributeView::new(
            ctx,
            found_attribute_view_context,
            Some(*found_attribute_value.id()),
        )
        .await?;
        found_attribute_view.value().clone()
    } else {
        // TODO(nick): deal with empty value.
        found_attribute_value.get_value(ctx).await?.unwrap()
    };

    // Generate a new func binding return value using the transformation function.
    // Use the found value from the attribute view as the args.
    let attribute_prototype = AttributePrototype::get_by_id(ctx, &attribute_prototype_id)
        .await?
        .ok_or(EmitError::AttributePrototypeNotFound(
            attribute_prototype_id,
        ))?;
    let func_id = attribute_prototype.func_id();
    let func = Func::get_by_id(ctx, &func_id)
        .await?
        .ok_or(EmitError::FuncNotFound(func_id))?;

    // TODO(nick): handle non-identity connections. This will only happen for explicit internal
    // providers and external providers... which do not use emit (maybe will never!).
    let args: serde_json::Value = match func.backend_kind() {
        FuncBackendKind::Identity => {
            serde_json::to_value(FuncBackendIdentityArgs { identity: payload })?
        }
        backend_kind => {
            todo!(
                "emitting for backend kind {:?} not supported yet",
                backend_kind
            )
        }
    };

    let (func_binding, func_binding_return_value) =
        FuncBinding::find_or_create_and_execute(ctx, args, *func.id()).await?;

    // We never want to mutate an emitted AttributeValue in the universal tenancy and we want
    // to ensure the found AttributeValue's context _exactly_ matches the one assembled. In
    // either case, just create a new one!
    if let Some(mut emit_attribute_value) =
        AttributeValue::find_for_context(ctx, emit_attribute_context.into()).await?
    {
        // TODO(nick): we will likely want to replace the "universal" tenancy check with an
        // "is compatible" tenancy check.
        if emit_attribute_value.context == emit_attribute_context
            && (!emit_attribute_value.tenancy().universal() || ctx.write_tenancy().universal())
        {
            emit_attribute_value
                .set_func_binding_id(ctx, *func_binding.id())
                .await?;
            emit_attribute_value
                .set_func_binding_return_value_id(ctx, *func_binding_return_value.id())
                .await?;
            return Ok(emit_attribute_value);
        }
    }
    let new_attribute_value = AttributeValue::new(
        ctx,
        *func_binding.id(),
        *func_binding_return_value.id(),
        emit_attribute_context,
        Option::<String>::None,
    )
    .await?;
    new_attribute_value
        .set_attribute_prototype(ctx, attribute_prototype.id())
        .await?;

    Ok(new_attribute_value)
}
