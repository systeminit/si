//! This module contains (and is oriented around) the [`RootProp`]. This object is not persisted
//! to the database.

use telemetry::prelude::*;

use crate::schema::variant::{SchemaVariantError, SchemaVariantResult};
use crate::{
    func::binding::FuncBinding, Func, Prop, PropId, PropKind, SchemaVariantId, StandardModel,
};
use crate::{
    AttributeContext, AttributeReadContext, AttributeValue, AttributeValueError, DalContext,
    SchemaId,
};

/// Contains the the si-specific [`PropId`](crate::Prop) and domain-specific [`PropId`](crate::Prop)
/// corresponding to a [`SchemaVariant`](crate::SchemaVariant). In addition, these correspond to the
/// "root" [`Props`](crate::Prop) on the [`ComponentView`](crate::ComponentView) "properties" field.
pub struct RootProp {
    pub si_prop_id: PropId,
    pub domain_prop_id: PropId,
}

impl RootProp {
    /// Creates and returns a [`RootProp`] for a [`SchemaVariant`](crate::SchemaVariant).
    #[instrument(skip_all)]
    pub async fn new(
        ctx: &DalContext<'_, '_>,
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
    ) -> SchemaVariantResult<Self> {
        let mut base_context_builder = AttributeContext::builder();
        base_context_builder
            .set_schema_id(schema_id)
            .set_schema_variant_id(schema_variant_id);

        let func_name = "si:setPropObject".to_string();
        let mut funcs = Func::find_by_attr(ctx, "name", &func_name).await?;
        let func = funcs
            .pop()
            .ok_or(SchemaVariantError::MissingFunc(func_name))?;

        let (func_binding, created) = FuncBinding::find_or_create(
            ctx,
            // Shortcut to creating the FuncBackendPropObjectArgs.
            serde_json::json!({ "value": {} }),
            *func.id(),
            *func.backend_kind(),
        )
        .await?;

        if created {
            func_binding.execute(ctx).await?;
        }

        let root_prop = Prop::new(ctx, "root", PropKind::Object).await?;
        root_prop
            .add_schema_variant(ctx, &schema_variant_id)
            .await?;

        let root_context = base_context_builder
            .clone()
            .set_prop_id(*root_prop.id())
            .to_context()?;
        let (_, root_value_id) = AttributeValue::update_for_context(
            ctx,
            *AttributeValue::find_for_context(ctx, root_context.into())
                .await?
                .pop()
                .ok_or(AttributeValueError::Missing)?
                .id(),
            None,
            root_context,
            Some(serde_json::json![{}]),
            None,
        )
        .await?;

        let base_attribute_read_context = AttributeReadContext {
            schema_id: Some(schema_id),
            schema_variant_id: Some(schema_variant_id),
            ..AttributeReadContext::default()
        };

        let si_specific_prop = Prop::new(ctx, "si", PropKind::Object).await?;
        si_specific_prop
            .set_parent_prop(ctx, *root_prop.id(), base_attribute_read_context)
            .await?;

        let si_context = base_context_builder
            .clone()
            .set_prop_id(*si_specific_prop.id())
            .to_context()?;
        let (_, si_value_id) = AttributeValue::update_for_context(
            ctx,
            *AttributeValue::find_for_context(ctx, si_context.into())
                .await?
                .pop()
                .ok_or(AttributeValueError::Missing)?
                .id(),
            Some(root_value_id),
            si_context,
            Some(serde_json::json![{}]),
            None,
        )
        .await?;

        let si_name_prop = Prop::new(ctx, "name", PropKind::String).await?;
        si_name_prop
            .set_parent_prop(ctx, *si_specific_prop.id(), base_attribute_read_context)
            .await?;

        let si_name_context = base_context_builder
            .clone()
            .set_prop_id(*si_name_prop.id())
            .to_context()?;
        AttributeValue::update_for_context(
            ctx,
            *AttributeValue::find_for_context(ctx, si_name_context.into())
                .await?
                .pop()
                .ok_or(AttributeValueError::Missing)?
                .id(),
            Some(si_value_id),
            si_name_context,
            None,
            None,
        )
        .await?;

        let domain_specific_prop = Prop::new(ctx, "domain", PropKind::Object).await?;
        domain_specific_prop
            .set_parent_prop(ctx, *root_prop.id(), base_attribute_read_context)
            .await?;

        let domain_context = base_context_builder
            .clone()
            .set_prop_id(*domain_specific_prop.id())
            .to_context()?;
        AttributeValue::update_for_context(
            ctx,
            *AttributeValue::find_for_context(ctx, domain_context.into())
                .await?
                .pop()
                .ok_or(AttributeValueError::Missing)?
                .id(),
            Some(root_value_id),
            domain_context,
            Some(serde_json::json![{}]),
            None,
        )
        .await?;

        Ok(Self {
            si_prop_id: *si_specific_prop.id(),
            domain_prop_id: *domain_specific_prop.id(),
        })
    }
}
