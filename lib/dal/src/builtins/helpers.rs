//! This module contains helpers that can that can be used in builtins migrations as well as
//! integration tests. It should be the only public builtins module.

use serde_json::Value;

use crate::attribute::context::AttributeContextBuilder;
use crate::func::binding::FuncBindingId;
use crate::func::binding_return_value::FuncBindingReturnValueId;
use crate::{
    func::binding::FuncBinding, AttributeReadContext, AttributeValue, AttributeValueId,
    BuiltinsError, BuiltinsResult, DalContext, Func, FuncError, FuncId, Prop, PropError, PropId,
    StandardModel,
};

/// Get the "si:identity" [`Func`](crate::Func) and execute (if necessary).
pub async fn setup_identity_func(
    ctx: &DalContext<'_, '_>,
) -> BuiltinsResult<(FuncId, FuncBindingId, FuncBindingReturnValueId)> {
    let identity_func_name = "si:identity".to_string();
    let identity_func: Func = Func::find_by_attr(ctx, "name", &identity_func_name)
        .await?
        .pop()
        .ok_or(FuncError::NotFoundByName(identity_func_name))?;
    let (identity_func_binding, identity_func_binding_return_value) =
        FuncBinding::find_or_create_and_execute(
            ctx,
            serde_json::json![{ "identity": null }],
            *identity_func.id(),
        )
        .await?;
    Ok((
        *identity_func.id(),
        *identity_func_binding.id(),
        *identity_func_binding_return_value.id(),
    ))
}

/// Find the child of a [`Prop`](crate::Prop) by name. _Use with caution!_
pub async fn find_child_prop_by_name(
    ctx: &DalContext<'_, '_>,
    prop_id: PropId,
    child_prop_name: &str,
) -> BuiltinsResult<Prop> {
    let prop = Prop::get_by_id(ctx, &prop_id)
        .await?
        .ok_or_else(|| PropError::NotFound(prop_id, *ctx.visibility()))?;
    for current in prop.child_props(ctx).await? {
        if current.name() == child_prop_name {
            return Ok(current);
        }
    }
    Err(PropError::ExpectedChildNotFound(child_prop_name.to_string()).into())
}

/// Update an [`AttributeValue`](crate::AttributeValue) for a provided [`Prop`](crate::Prop) and
/// base [`AttributeReadContext`](crate::AttributeReadContext). This only works if the parent
/// [`AttributeValue`](crate::AttributeValue) corresponds to an _"object"_ [`Prop`](crate::Prop)
/// that's already been "initialized".
pub async fn update_attribute_value_for_prop_and_context(
    ctx: &DalContext<'_, '_>,
    prop_id: PropId,
    value: Option<Value>,
    base_attribute_read_context: AttributeReadContext,
) -> BuiltinsResult<AttributeValueId> {
    let attribute_value_context = AttributeReadContext {
        prop_id: Some(prop_id),
        ..base_attribute_read_context
    };
    let attribute_value = AttributeValue::find_for_context(ctx, attribute_value_context)
        .await?
        .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
            attribute_value_context,
        ))?;

    let parent_prop_id = parent_prop(ctx, prop_id).await?;
    let parent_attribute_value_context = AttributeReadContext {
        prop_id: Some(parent_prop_id),
        ..base_attribute_read_context
    };
    let parent_attribute_value =
        AttributeValue::find_for_context(ctx, parent_attribute_value_context)
            .await?
            .ok_or(BuiltinsError::AttributeValueNotFoundForContext(
                parent_attribute_value_context,
            ))?;

    let prop_attribute_context = AttributeContextBuilder::from(base_attribute_read_context)
        .set_prop_id(prop_id)
        .to_context()?;

    let (_, updated_attribute_value_id, _) = AttributeValue::update_for_context(
        ctx,
        *attribute_value.id(),
        Some(*parent_attribute_value.id()),
        prop_attribute_context,
        value,
        None,
    )
    .await?;

    // Return the updated attribute value id.
    Ok(updated_attribute_value_id)
}

/// Find the parent for a given [`PropId`](crate::Prop).
pub async fn parent_prop(ctx: &DalContext<'_, '_>, prop_id: PropId) -> BuiltinsResult<PropId> {
    let parent_prop = Prop::get_by_id(ctx, &prop_id)
        .await?
        .ok_or(BuiltinsError::PropNotFound(prop_id))?
        .parent_prop(ctx)
        .await?
        .ok_or(BuiltinsError::PropParentNotFoundOrEmpty(prop_id))?;
    Ok(*parent_prop.id())
}
