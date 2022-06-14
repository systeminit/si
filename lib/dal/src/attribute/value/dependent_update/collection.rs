//! This private module provides [`AttributeValueDependentCollectionHarness`] for collecting
//! (finding or creating) [`AttributeValues`](crate::AttributeValue) that are "dependent"
//! on an updated [`AttributeValue`](crate::AttributeValue).

use std::cmp::Ordering;

use crate::attribute::context::{AttributeContextBuilder, AttributeContextLeastSpecificFieldKind};
use crate::attribute::value::dependent_update::AttributeValueDependentUpdateHarness;
use crate::{
    AttributeContext, AttributePrototype, AttributeValue, AttributeValueError,
    AttributeValueResult, DalContext, ExternalProvider, InternalProvider, Prop, StandardModel,
};

/// A field-less struct to that acts as an interface to provide [`Self::collect()`].
pub struct AttributeValueDependentCollectionHarness;

impl AttributeValueDependentCollectionHarness {
    /// Find or create all [`AttributeValues`](crate::AttributeValue) that need to be updated
    /// based on a source [`AttributeContext`](crate::AttributeContext). These are the
    /// [`AttributeValues`](crate::AttributeValue) that are "dependent" on an updated
    /// [`AttributeValue`](crate::AttributeValue) (which is where the source
    /// [`AttributeContext`](crate::AttributeContext) is derived from).
    pub async fn collect(
        ctx: &DalContext<'_, '_>,
        source_attribute_context: AttributeContext,
    ) -> AttributeValueResult<Vec<AttributeValue>> {
        match source_attribute_context.least_specific_field_kind()? {
            AttributeContextLeastSpecificFieldKind::Prop => {
                Self::collect_for_least_specific_kind_prop(ctx, source_attribute_context).await
            }
            AttributeContextLeastSpecificFieldKind::InternalProvider => {
                Self::collect_for_least_specific_kind_internal_provider(
                    ctx,
                    source_attribute_context,
                )
                .await
            }
            AttributeContextLeastSpecificFieldKind::ExternalProvider => {
                Self::collect_for_least_specific_kind_external_provider(
                    ctx,
                    source_attribute_context,
                )
                .await
            }
        }
    }

    /// Collect [`AttributeValues`](crate::AttributeValue) that need to updated based on the
    /// provided source [`AttributeContext`](crate::AttributeContext) whose least specific field
    /// specified is a [`PropId`](crate::Prop).
    async fn collect_for_least_specific_kind_prop(
        ctx: &DalContext<'_, '_>,
        source_attribute_context: AttributeContext,
    ) -> AttributeValueResult<Vec<AttributeValue>> {
        let mut attribute_values_that_need_to_be_updated = Vec::new();
        // First, we need to ensure our corresponding implicit internal provider emits,
        // if one exists.
        if let Some(source_implicit_internal_provider) =
            InternalProvider::get_for_prop(ctx, source_attribute_context.prop_id())
                .await
                .map_err(|e| AttributeValueError::InternalProvider(e.to_string()))?
        {
            source_implicit_internal_provider
                .emit(ctx, source_attribute_context)
                .await
                .map_err(|e| AttributeValueError::InternalProvider(e.to_string()))?;
        }

        // We will start by finding all ancestor props for the prop on the current
        // attribute value that we are processing. We will then find the internal provider
        // corresponding to that prop (side note: if we are "underneath" an array or a map,
        // we will not have an internal provider). From that internal provider, we will find the
        // attribute prototypes who have arguments that specify that internal provider's id.
        // From there, we can find the attribute values that need to be updated.
        for ancestor_prop in
            Prop::all_ancestor_props(ctx, source_attribute_context.prop_id()).await?
        {
            if let Some(ancestor_implicit_internal_provider) =
                InternalProvider::get_for_prop(ctx, *ancestor_prop.id())
                    .await
                    .map_err(|e| AttributeValueError::InternalProvider(e.to_string()))?
            {
                let attribute_prototypes_from_implicit_internal_provider_use =
                    AttributePrototype::list_from_internal_provider_use(
                        ctx,
                        *ancestor_implicit_internal_provider.id(),
                    )
                    .await
                    .map_err(|e| AttributeValueError::AttributePrototype(format!("{e}")))?;
                let attribute_values_in_progress =
                    Self::find_or_create_attribute_values_that_need_to_be_updated(
                        ctx,
                        attribute_prototypes_from_implicit_internal_provider_use,
                        source_attribute_context,
                    )
                    .await?;
                attribute_values_that_need_to_be_updated.extend(attribute_values_in_progress);
            }
        }

        Ok(attribute_values_that_need_to_be_updated)
    }

    /// Collect [`AttributeValues`](crate::AttributeValue) that need to updated based on the
    /// provided source [`AttributeContext`](crate::AttributeContext) whose least specific field
    /// specified is an [`InternalProviderId`](crate::InternalProvider).
    async fn collect_for_least_specific_kind_internal_provider(
        ctx: &DalContext<'_, '_>,
        source_attribute_context: AttributeContext,
    ) -> AttributeValueResult<Vec<AttributeValue>> {
        let mut attribute_values_that_need_to_be_updated = Vec::new();

        // Since the source context's least specific field specified is an internal provider id,
        // not only do we need to ensure that our context's component id is the "head" for potential
        // connections, but we also need to track "consumption" from "explicit internal providers"
        // (i.e. "attribute_prototypes_without_corresponding_external_providers"). Essentially,
        // these initial attribute prototypes collected will either be those that correspond to
        // external providers or those that don't.
        let attribute_prototypes_from_internal_provider_use =
            AttributePrototype::list_from_internal_provider_use(
                ctx,
                source_attribute_context.internal_provider_id(),
            )
            .await
            .map_err(|e| AttributeValueError::AttributePrototype(format!("{e}")))?;

        // These are the attribute prototypes who do not have corresponding external providers
        // (i.e. "consumption" from "explicit internal providers").
        let mut attribute_prototypes_without_corresponding_external_providers = Vec::new();

        for attribute_prototype_from_internal_provider_use in
            attribute_prototypes_from_internal_provider_use
        {
            let external_providers_found =
                ExternalProvider::list_for_attribute_prototype_with_head(
                    ctx,
                    *attribute_prototype_from_internal_provider_use.id(),
                    source_attribute_context.component_id(),
                )
                .await
                .map_err(|e| AttributeValueError::ExternalProvider(format!("{e}")))?;

            for external_provider in &external_providers_found {
                // Use everything from the source context, but change the least specific field to
                // the external provider id.
                let modified_source_attribute_context =
                    AttributeContextBuilder::from(source_attribute_context)
                        .unset_prop_id()
                        .unset_internal_provider_id()
                        .set_external_provider_id(*external_provider.id())
                        .to_context()?;

                attribute_values_that_need_to_be_updated.extend(
                    Self::find_or_create_attribute_values_that_need_to_be_updated(
                        ctx,
                        vec![attribute_prototype_from_internal_provider_use.clone()],
                        modified_source_attribute_context,
                    )
                    .await?,
                );
            }

            if external_providers_found.is_empty() {
                attribute_prototypes_without_corresponding_external_providers
                    .push(attribute_prototype_from_internal_provider_use);
            }
        }

        // Find or create attribute values for prototype's who did not have corresponding
        // external providers (i.e. "consumption" from "explicit internal providers").
        attribute_values_that_need_to_be_updated.extend(
            Self::find_or_create_attribute_values_that_need_to_be_updated(
                ctx,
                attribute_prototypes_without_corresponding_external_providers,
                source_attribute_context,
            )
            .await?,
        );
        Ok(attribute_values_that_need_to_be_updated)
    }

    /// Collect [`AttributeValues`](crate::AttributeValue) that need to updated based on the
    /// provided source [`AttributeContext`](crate::AttributeContext) whose least specific field
    /// specified is an [`ExternalProviderId`](crate::ExternalProvider).
    async fn collect_for_least_specific_kind_external_provider(
        ctx: &DalContext<'_, '_>,
        source_attribute_context: AttributeContext,
    ) -> AttributeValueResult<Vec<AttributeValue>> {
        let mut attribute_values_that_need_to_be_updated = Vec::new();

        // Since the source context's least specific field specified is an external provider id,
        // we need to ensure that our context's component id is the "tail" for potential connections.
        let attribute_prototype_groups =
            AttributePrototype::list_by_head_from_external_provider_use_with_tail(
                ctx,
                source_attribute_context.external_provider_id(),
                source_attribute_context.component_id(),
            )
            .await
            .map_err(|e| AttributeValueError::AttributePrototype(format!("{e}")))?;

        for attribute_prototype_group in attribute_prototype_groups {
            let attribute_prototype = attribute_prototype_group.attribute_prototype;
            let internal_providers_found =
                InternalProvider::list_for_attribute_prototype(ctx, *attribute_prototype.id())
                    .await
                    .map_err(|e| AttributeValueError::InternalProvider(format!("{e}")))?;

            for internal_provider in internal_providers_found {
                // We need to create an ugly ass context that uses the following:
                // - the internal provider for the schema variant field and below
                // - the attribute prototype group's head component id for the component id (the
                //   other side of the "inter component connection")
                // - the attribute prototype's context for everything "more-specific" than the
                //   component id
                let modified_source_attribute_context =
                    AttributeContextBuilder::from(attribute_prototype.context)
                        .unset_prop_id()
                        .unset_external_provider_id()
                        .set_internal_provider_id(*internal_provider.id())
                        .set_schema_id(*internal_provider.schema_id())
                        .set_schema_variant_id(*internal_provider.schema_variant_id())
                        .set_component_id(attribute_prototype_group.head_component_id)
                        .to_context()?;
                attribute_values_that_need_to_be_updated.extend(
                    Self::find_or_create_attribute_values_that_need_to_be_updated(
                        ctx,
                        vec![attribute_prototype.clone()],
                        modified_source_attribute_context,
                    )
                    .await?,
                );
            }
        }

        Ok(attribute_values_that_need_to_be_updated)
    }

    /// Find or create [`AttributeValues`](crate::AttributeValue) that need to be updated based on
    /// the provided [`AttributePrototypes`](crate::AttributePrototype) and the source
    /// [`AttributeContext`](crate::AttributeContext).
    async fn find_or_create_attribute_values_that_need_to_be_updated(
        ctx: &DalContext<'_, '_>,
        attribute_prototypes: Vec<AttributePrototype>,
        source_attribute_context: AttributeContext,
    ) -> AttributeValueResult<Vec<AttributeValue>> {
        let mut attribute_values_that_need_to_be_updated = Vec::new();

        for attribute_prototype in attribute_prototypes {
            let attribute_values_in_context_or_greater_context = Self::merge_attribute_contexts(
                attribute_prototype.context,
                source_attribute_context,
            )?;

            let mut found_exact_context_level = false;
            let attribute_values_in_context_or_greater =
                AttributePrototype::attribute_values_in_context_or_greater(
                    ctx,
                    *attribute_prototype.id(),
                    attribute_values_in_context_or_greater_context.into(),
                )
                .await
                .map_err(|e| AttributeValueError::AttributePrototype(format!("{e}")))?;

            dbg!(
                "OR GREATER CONTEXT",
                &attribute_values_in_context_or_greater_context
            );

            // For each relevant attribute value found corresponding to the attribute
            // prototype, check if its context is at same or greater ("more-specific")
            // level of specificity. If either are true, the attribute value being processed
            // needs to be updated. If the former is true, then we need to create an
            // attribute value in a context whose level of specificity is the same
            // as the context of the "original" attribute value that was updated.
            for attribute_value_in_context_or_greater in attribute_values_in_context_or_greater {
                dbg!(
                    "OR GREATER VALUE",
                    attribute_value_in_context_or_greater.id()
                );
                if attribute_value_in_context_or_greater.context >= source_attribute_context {
                    // We cannot use the "==" operator because we have derived "PartialEq"
                    // in addition to creating our own "partial_cmp" implementation within
                    // our "PartialOrd impl".
                    if attribute_value_in_context_or_greater
                        .context
                        .partial_cmp(&source_attribute_context)
                        == Some(Ordering::Equal)
                    {
                        found_exact_context_level = true;
                    }
                    // if attribute_value_in_context_or_greater.context == source_attribute_context {
                    //     found_exact_context_level = true;
                    // }

                    // If values of a "more-specific" context appear, then they were not
                    // pinned and we need to update them as well.
                    attribute_values_that_need_to_be_updated
                        .push(attribute_value_in_context_or_greater);
                }
            }

            // If this condition passes, we need to create a new attribute value with
            // aforementioned specifications. First, let's find the attribute value
            // corresponding to the attribute prototype that we are currently working
            // with. We will use its data to help create the new attribute value.
            if !found_exact_context_level {
                let attribute_value_for_current_prototype =
                    AttributeValue::find_for_context(ctx, attribute_prototype.context.into())
                        .await?
                        .ok_or(AttributeValueError::Missing)?;

                // The context for creating the new attribute value will use the least specific
                // field from the attribute value corresponding to the attribute prototype
                // that we are currently working with _and_ will use the context
                // from the "source" attribute value updated for all other fields.
                let mut new_attribute_value_context_builder =
                    AttributeContextBuilder::from(source_attribute_context);

                // We need to ensure that we unset all other least specific fields.
                let new_attribute_value_context = match attribute_value_for_current_prototype
                    .context
                    .least_specific_field_kind()?
                {
                    AttributeContextLeastSpecificFieldKind::Prop => {
                        new_attribute_value_context_builder
                            .set_prop_id(attribute_value_for_current_prototype.context.prop_id())
                            .unset_internal_provider_id()
                            .unset_external_provider_id()
                    }
                    AttributeContextLeastSpecificFieldKind::InternalProvider => {
                        new_attribute_value_context_builder
                            .set_internal_provider_id(
                                attribute_value_for_current_prototype
                                    .context
                                    .internal_provider_id(),
                            )
                            .unset_prop_id()
                            .unset_external_provider_id()
                    }
                    AttributeContextLeastSpecificFieldKind::ExternalProvider => {
                        new_attribute_value_context_builder
                            .set_external_provider_id(
                                attribute_value_for_current_prototype
                                    .context
                                    .external_provider_id(),
                            )
                            .unset_prop_id()
                            .unset_internal_provider_id()
                    }
                }
                .to_context()?;

                let mut found_value = false;
                for mut value in
                    AttributeValue::list_for_context(ctx, new_attribute_value_context.into())
                        .await?
                {
                    // dbg!(
                    //     "found",
                    //     &value.context == &new_attribute_value_context,
                    //     &value.context,
                    //     &new_attribute_value_context,
                    //     &value.func_binding_id,
                    //     &attribute_value_for_current_prototype.func_binding_id,
                    //     &value.func_binding_return_value_id,
                    //     &attribute_value_for_current_prototype.func_binding_return_value_id,
                    //     &value.attribute_prototype(ctx).await?,
                    //     &attribute_prototype.id(),
                    // );

                    if value.context == new_attribute_value_context {
                        dbg!("PUSH");
                        // if let Some(mut p) = value.attribute_prototype(ctx).await? {
                        //     p.set_func_id(ctx, attribute_prototype.func_id())
                        //         .await
                        //         .unwrap();
                        // }

                        // TODO(nick): needs to belong to new prototype as new! BUT IT FAILS.

                        // value.unset_attribute_prototype(ctx);
                        // value
                        //     .set_attribute_prototype(ctx, attribute_prototype.id())
                        //     .await?;

                        // TODO: re-enable.
                        // attribute_values_that_need_to_be_updated.push(value);
                        // found_value = true;
                        // break;
                    }
                }
                // let equivalent_value = attribute_prototype
                //     .attribute_values(ctx)
                //     .await
                //     .unwrap()
                //     .pop()
                //     .unwrap();
                // if equivalent_value.context == new_attribute_value_context {
                //     attribute_values_that_need_to_be_updated.push(equivalent_value);
                //     found_value = true;
                // };
                if !found_value {
                    let new_attribute_value = AttributeValue::new(
                        ctx,
                        attribute_value_for_current_prototype.func_binding_id,
                        attribute_value_for_current_prototype.func_binding_return_value_id,
                        new_attribute_value_context,
                        attribute_value_for_current_prototype.key.clone(),
                    )
                    .await?;

                    // Before adding our new attribute value to the list of attribute values
                    // that need to be updated, we need to set its prototype and its parent
                    // attribute value.
                    new_attribute_value
                        .set_attribute_prototype(ctx, attribute_prototype.id())
                        .await?;
                    if let Some(parent_attribute_value) = attribute_value_for_current_prototype
                        .parent_attribute_value(ctx)
                        .await?
                    {
                        let parent_attribute_context =
                            AttributeContextBuilder::from(new_attribute_value.context)
                                .set_prop_id(parent_attribute_value.context.prop_id())
                                .to_context()?;
                        let parent_attribute_value_id =
                            AttributeValue::vivify_value_and_parent_values(
                                ctx,
                                parent_attribute_context,
                                *parent_attribute_value.id(),
                            )
                            .await?;
                        new_attribute_value
                            .set_parent_attribute_value(ctx, &parent_attribute_value_id)
                            .await?;
                    }
                    attribute_values_that_need_to_be_updated.push(new_attribute_value);
                }
            }
        }
        Ok(attribute_values_that_need_to_be_updated)
    }

    /// Merges two [`AttributeContexts`](crate::AttributeContext) with preference to the "base"
    /// context.
    fn merge_attribute_contexts(
        base_attribute_context: AttributeContext,
        override_attribute_context: AttributeContext,
    ) -> AttributeValueResult<AttributeContext> {
        let mut builder = AttributeContextBuilder::from(base_attribute_context);
        let mut skip = false;

        // We do not need to check "skip" at the most specific field.
        if base_attribute_context.is_system_unset() {
            if !override_attribute_context.is_system_unset() {
                builder.set_system_id(override_attribute_context.system_id());
            }
        } else {
            skip = true;
        }
        if base_attribute_context.is_component_unset() && !skip {
            if !override_attribute_context.is_component_unset() {
                builder.set_component_id(override_attribute_context.component_id());
            }
        } else {
            skip = true;
        }

        if base_attribute_context.is_schema_variant_unset() && !skip {
            if !override_attribute_context.is_schema_variant_unset() {
                builder.set_schema_variant_id(override_attribute_context.schema_variant_id());
            }
        } else {
            skip = true;
        }

        // We do not check the least specific fields.
        if base_attribute_context.is_schema_unset()
            && !skip
            && !override_attribute_context.is_schema_unset()
        {
            builder.set_schema_id(override_attribute_context.schema_id());
        }

        Ok(builder.to_context()?)
    }
}
