//! This private module provides [`AttributeValueDependentUpdateHarness`] for updating all
//! [`AttributeValues`](crate::AttributeValue) that are "dependent" on an updated
//! [`AttributeValue`](crate::AttributeValue).

use std::collections::{HashMap, HashSet, VecDeque};

use crate::{
    attribute::context::AttributeContextBuilder,
    attribute::value::dependent_update::collection::AttributeValueDependentCollectionHarness,
    context::JobContent, AttributeContext, AttributePrototypeArgument, AttributeValue,
    AttributeValueError, AttributeValueId, AttributeValueResult, Component, ComponentAsyncTasks,
    ComponentId, DalContext, FuncBinding, InternalProvider, Prop, PropKind, StandardModel,
    SystemId,
};

use super::DependentValuesAsyncTasks;

pub mod collection;

/// A field-less struct to that acts as an interface to provide [`Self::update_dependent_values()`].
pub struct AttributeValueDependentUpdateHarness;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct AlmostComponentAsyncTask {
    component_id: ComponentId,
    system_id: SystemId,
}

impl AttributeValueDependentUpdateHarness {
    /// Update dependent_update [`AttributeValues`](crate::AttributeValue) for an updated
    /// [`AttributeValueId`](crate::AttributeValue).
    pub async fn update_dependent_values(
        ctx: &DalContext<'_, '_>,
        attribute_value_id_to_update: AttributeValueId,
    ) -> AttributeValueResult<Vec<ComponentAsyncTasks>> {
        let mut attribute_value_that_needs_to_be_updated =
            AttributeValue::get_by_id(ctx, &attribute_value_id_to_update)
                .await?
                .ok_or(AttributeValueError::Missing)?;

        let attribute_prototype = attribute_value_that_needs_to_be_updated
            .attribute_prototype(ctx)
            .await?
            .ok_or(AttributeValueError::MissingAttributePrototype)?;

        // Iterate over each group of attribute prototype arguments (grouped by argument
        // name) to assemble our func binding arguments. For each group, if the arguments
        // length is greater than one, then we have more than one argument with the same
        // name.
        //
        // Examples:
        // - If one argument in group --> FuncBinding arg --> { name: value }
        // - If two arguments in group --> FuncBinding arg --> { name: [ value1, value2 ] }
        let mut func_binding_args: HashMap<String, Option<serde_json::Value>> = HashMap::new();
        for mut argument_group in AttributePrototypeArgument::list_by_name_for_attribute_prototype(
            ctx,
            *attribute_prototype.id(),
        )
        .await?
        {
            #[allow(clippy::comparison_chain)]
            if argument_group.arguments.len() == 1 {
                // This error should be impossible to hit since we have one argument.
                let argument = argument_group.arguments.pop().ok_or_else(|| {
                    AttributeValueError::EmptyAttributePrototypeArgumentsForGroup(
                        argument_group.name.clone(),
                    )
                })?;
                func_binding_args.insert(
                    argument_group.name,
                    Self::build_func_binding_argument_value_from_attribute_prototype_argument(
                        ctx,
                        argument,
                        attribute_value_that_needs_to_be_updated.context,
                    )
                    .await?,
                );
            } else if argument_group.arguments.len() > 1 {
                let mut assembled_values = Vec::new();
                for argument in argument_group.arguments {
                    assembled_values.push(
                        Self::build_func_binding_argument_value_from_attribute_prototype_argument(
                            ctx,
                            argument,
                            attribute_value_that_needs_to_be_updated.context,
                        )
                        .await?,
                    );
                }
                func_binding_args.insert(
                    argument_group.name,
                    Some(serde_json::to_value(assembled_values)?),
                );
            } else {
                // This should not be possible, but we will check just in case the query
                // (or something else) regresses.
                return Err(
                    AttributeValueError::EmptyAttributePrototypeArgumentsForGroup(
                        argument_group.name,
                    ),
                );
            }
        }

        // Generate a new func binding return value with our arguments assembled.
        let (func_binding, mut func_binding_return_value) =
            FuncBinding::find_or_create_and_execute(
                ctx,
                serde_json::to_value(func_binding_args)?,
                attribute_prototype.func_id(),
            )
            .await?;

        // Update the attribute value with the new func binding and func binding return value.
        attribute_value_that_needs_to_be_updated
            .set_func_binding_id(ctx, *func_binding.id())
            .await?;
        attribute_value_that_needs_to_be_updated
            .set_func_binding_return_value_id(ctx, *func_binding_return_value.id())
            .await?;

        // If the value we just updated was for a Prop, we might have run a function that
        // generates a deep data structure. If the Prop is an Array/Map/Object, then the
        // value should be an empty Array/Map/Object, while the unprocessed value contains
        // the deep data structure.
        if attribute_value_that_needs_to_be_updated
            .context
            .is_least_specific_field_kind_prop()?
        {
            let processed_value = match func_binding_return_value.unprocessed_value().cloned() {
                Some(unprocessed_value) => {
                    let prop = Prop::get_by_id(
                        ctx,
                        &attribute_value_that_needs_to_be_updated.context.prop_id(),
                    )
                    .await?
                    .ok_or_else(|| {
                        AttributeValueError::PropNotFound(
                            attribute_value_that_needs_to_be_updated.context.prop_id(),
                        )
                    })?;

                    match prop.kind() {
                        PropKind::Object | PropKind::Map => Some(serde_json::json!({})),
                        PropKind::Array => Some(serde_json::json!([])),
                        _ => Some(unprocessed_value),
                    }
                }
                None => None,
            };
            func_binding_return_value
                .set_value(ctx, processed_value)
                .await?;
        };
        // The value will be different from the unprocessed value if we updated it above
        // for an Array/Map/Value. If they are different from each other, then we know
        // that we need to fully process the deep data structure, populating
        // AttributeValues for the child Props.
        if func_binding_return_value.unprocessed_value() != func_binding_return_value.value() {
            if let Some(unprocessed_value) = func_binding_return_value.unprocessed_value().cloned()
            {
                AttributeValue::populate_nested_values(
                    ctx,
                    *attribute_value_that_needs_to_be_updated.id(),
                    attribute_value_that_needs_to_be_updated.context,
                    unprocessed_value,
                )
                .await?;
            }
        }

        if attribute_value_that_needs_to_be_updated
            .context
            .component_id()
            .is_some()
        {
            if let Some(component) = Component::get_by_id(
                ctx,
                &attribute_value_that_needs_to_be_updated
                    .context
                    .component_id(),
            )
            .await?
            {
                component
                    .check_validations(
                        ctx,
                        *attribute_value_that_needs_to_be_updated.id(),
                        &func_binding_return_value.value().cloned(),
                    )
                    .await
                    .map_err(|err| AttributeValueError::Component(err.to_string()))?;

                // We only want to enqueue a job to check the qualifications if the AttributeValue
                // is for the implicit InternalProvider of the Root Prop of the Component.
                if attribute_value_that_needs_to_be_updated
                    .context
                    .is_least_specific_field_kind_internal_provider()?
                {
                    let internal_provider = InternalProvider::get_by_id(
                        ctx,
                        &attribute_value_that_needs_to_be_updated
                            .context
                            .internal_provider_id(),
                    )
                    .await?
                    .ok_or_else(|| {
                        AttributeValueError::InternalProviderNotFound(
                            attribute_value_that_needs_to_be_updated
                                .context
                                .internal_provider_id(),
                        )
                    })?;
                    let provider_prop = Prop::get_by_id(ctx, internal_provider.prop_id())
                        .await?
                        .ok_or_else(|| {
                            AttributeValueError::PropNotFound(*internal_provider.prop_id())
                        })?;

                    // The Root Prop won't have a parent Prop.
                    if provider_prop.parent_prop(ctx).await?.is_none() {
                        let task = component
                            .build_async_tasks(
                                ctx,
                                attribute_value_that_needs_to_be_updated.context.system_id(),
                            )
                            .await
                            .map_err(|err| AttributeValueError::Component(err.to_string()))?;

                        ctx.enqueue_job(JobContent::DependentValuesUpdate(
                            DependentValuesAsyncTasks::new(Some(task), None),
                        ))
                        .await;
                    }
                }
            }
        }

        let dependent_attribute_values = AttributeValueDependentCollectionHarness::collect(
            ctx,
            attribute_value_that_needs_to_be_updated.context,
        )
        .await?;
        for dependent_attribute_value in dependent_attribute_values {
            ctx.enqueue_job(JobContent::DependentValuesUpdate(
                DependentValuesAsyncTasks::new(None, Some(*dependent_attribute_value.id())),
            ))
            .await;
        }

        Ok(vec![])
    }

    /// Build a [`FuncBinding`](crate::FuncBinding) argument from a provided
    /// [`AttributePrototypeArgument`](crate::AttributePrototypeArgument) and context of the
    /// [`AttributeValue`] that needs to be updated.
    async fn build_func_binding_argument_value_from_attribute_prototype_argument(
        ctx: &DalContext<'_, '_>,
        attribute_prototype_argument: AttributePrototypeArgument,
        attribute_value_context: AttributeContext,
    ) -> AttributeValueResult<Option<serde_json::Value>> {
        let value = if attribute_prototype_argument.is_internal_provider_unset() {
            // Collect the tail component values we need for our external provider context.
            let tail_component_id = attribute_prototype_argument.tail_component_id();
            let tail_component = Component::get_by_id(ctx, &tail_component_id)
                .await?
                .ok_or(AttributeValueError::ComponentNotFound(tail_component_id))?;
            let tail_schema = tail_component
                .schema(ctx)
                .await
                .map_err(|e| AttributeValueError::Component(format!("{e}")))?
                .ok_or(AttributeValueError::SchemaNotFoundForComponent(
                    tail_component_id,
                ))?;
            let tail_schema_variant = tail_component
                .schema_variant(ctx)
                .await
                .map_err(|e| AttributeValueError::Component(format!("{e}")))?
                .ok_or(AttributeValueError::SchemaVariantNotFoundForComponent(
                    tail_component_id,
                ))?;

            // Our external provider context will use the following:
            // - the external provider id derived from the attribute prototype argument
            // - the schema id, schema variant id, and component id derived from the tail component
            // - everything "more-specific" from the provided attribute value context
            let external_provider_context = AttributeContextBuilder::from(attribute_value_context)
                .unset_internal_provider_id()
                .unset_prop_id()
                .set_external_provider_id(attribute_prototype_argument.external_provider_id())
                .set_schema_id(*tail_schema.id())
                .set_schema_variant_id(*tail_schema_variant.id())
                .set_component_id(tail_component_id)
                .to_context()?;
            let external_provider_attribute_value =
                AttributeValue::find_for_context(ctx, external_provider_context.into())
                    .await?
                    .ok_or(AttributeValueError::NotFoundForExternalProviderContext(
                        external_provider_context,
                    ))?;
            external_provider_attribute_value.get_value(ctx).await?
        } else {
            let internal_provider_context = AttributeContextBuilder::from(attribute_value_context)
                .unset_external_provider_id()
                .unset_prop_id()
                .set_internal_provider_id(attribute_prototype_argument.internal_provider_id())
                .to_context()?;
            let internal_provider_attribute_value =
                AttributeValue::find_for_context(ctx, internal_provider_context.into())
                    .await?
                    .ok_or(AttributeValueError::NotFoundForInternalProviderContext(
                        internal_provider_context,
                    ))?;
            internal_provider_attribute_value.get_value(ctx).await?
        };

        Ok(value)
    }
}
