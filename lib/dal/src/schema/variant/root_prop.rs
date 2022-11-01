//! This module contains (and is oriented around) the [`RootProp`]. This object is not persisted
//! to the database.

use telemetry::prelude::*;

use crate::{
    schema::variant::SchemaVariantResult, AttributeContext, AttributeValue, AttributeValueError,
    DalContext, Prop, PropId, PropKind, SchemaVariant, SchemaVariantId, StandardModel,
};

/// Contains the si-specific [`PropId`](crate::Prop), the domain-specific [`PropId`](crate::Prop),
/// and the root [`PropId`](crate::Prop) corresponding to a [`SchemaVariant`](crate::SchemaVariant).
/// In addition, these correspond to the "root" [`Props`](crate::Prop) on the
/// [`ComponentView`](crate::ComponentView) "properties" field.
#[derive(Debug, Copy, Clone)]
pub struct RootProp {
    pub prop_id: PropId,
    pub si_prop_id: PropId,
    pub domain_prop_id: PropId,
    pub resource_prop_id: PropId,
}

impl RootProp {
    /// Creates and returns a [`RootProp`] for a [`SchemaVariant`](crate::SchemaVariant).
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<Self> {
        let root_prop = Prop::new(ctx, "root", PropKind::Object, None).await?;
        root_prop
            .add_schema_variant(ctx, &schema_variant_id)
            .await?;

        let si_specific_prop = Prop::new(ctx, "si", PropKind::Object, None).await?;
        si_specific_prop
            .set_parent_prop(ctx, *root_prop.id())
            .await?;

        let si_name_prop = Prop::new(ctx, "name", PropKind::String, None).await?;
        si_name_prop
            .set_parent_prop(ctx, *si_specific_prop.id())
            .await?;

        let domain_specific_prop = Prop::new(ctx, "domain", PropKind::Object, None).await?;
        domain_specific_prop
            .set_parent_prop(ctx, *root_prop.id())
            .await?;

        let resource_specific_prop = Prop::new(ctx, "resource", PropKind::String, None).await?;
        resource_specific_prop
            .set_parent_prop(ctx, *root_prop.id())
            .await?;

        // Now that the structure is set up, we can populate default
        // AttributePrototypes & AttributeValues to be updated appropriately below.
        SchemaVariant::create_default_prototypes_and_values(ctx, schema_variant_id).await?;

        let root_context = AttributeContext::builder()
            .set_prop_id(*root_prop.id())
            .to_context()?;
        let (_, root_value_id) = AttributeValue::update_for_context(
            ctx,
            *AttributeValue::find_for_context(ctx, root_context.into())
                .await?
                .ok_or(AttributeValueError::Missing)?
                .id(),
            None,
            root_context,
            Some(serde_json::json![{}]),
            None,
        )
        .await?;

        let si_context = AttributeContext::builder()
            .set_prop_id(*si_specific_prop.id())
            .to_context()?;
        let (_, si_value_id) = AttributeValue::update_for_context(
            ctx,
            *AttributeValue::find_for_context(ctx, si_context.into())
                .await?
                .ok_or(AttributeValueError::Missing)?
                .id(),
            Some(root_value_id),
            si_context,
            Some(serde_json::json![{}]),
            None,
        )
        .await?;

        let si_name_context = AttributeContext::builder()
            .set_prop_id(*si_name_prop.id())
            .to_context()?;
        let (_, _) = AttributeValue::update_for_context(
            ctx,
            *AttributeValue::find_for_context(ctx, si_name_context.into())
                .await?
                .ok_or(AttributeValueError::Missing)?
                .id(),
            Some(si_value_id),
            si_name_context,
            None,
            None,
        )
        .await?;

        let domain_context = AttributeContext::builder()
            .set_prop_id(*domain_specific_prop.id())
            .to_context()?;
        let (_, _) = AttributeValue::update_for_context(
            ctx,
            *AttributeValue::find_for_context(ctx, domain_context.into())
                .await?
                .ok_or(AttributeValueError::Missing)?
                .id(),
            Some(root_value_id),
            domain_context,
            Some(serde_json::json![{}]),
            None,
        )
        .await?;

        Ok(Self {
            prop_id: *root_prop.id(),
            si_prop_id: *si_specific_prop.id(),
            domain_prop_id: *domain_specific_prop.id(),
            resource_prop_id: *resource_specific_prop.id(),
        })
    }
}
