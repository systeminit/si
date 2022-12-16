//! This module contains (and is oriented around) the [`RootProp`]. This object is not persisted
//! to the database.

use telemetry::prelude::*;

use crate::{
    schema::variant::{leaves::LeafKind, SchemaVariantResult},
    AttributeContext, AttributeValue, AttributeValueError, DalContext, Prop, PropId, PropKind,
    SchemaVariant, SchemaVariantId, StandardModel,
};

/// Contains the root [`PropId`](crate::Prop) and its immediate children for a
/// [`SchemaVariant`](crate::SchemaVariant). These [`Props`](crate::Prop) are also those that
/// correspond to the "root" [`Props`](crate::Prop) on the [`ComponentView`](crate::ComponentView)
/// "properties" field.
#[derive(Debug, Copy, Clone)]
pub struct RootProp {
    /// The parent of the other [`Props`](crate::Prop) on [`self`](Self).
    pub prop_id: PropId,
    /// Contains the tree of [`Props`](crate::Prop) corresponding to System Initiative metadata.
    pub si_prop_id: PropId,
    /// Contains the tree of [`Props`](crate::Prop) corresponding to the real world _model_.
    pub domain_prop_id: PropId,
    /// Contains the tree of [`Props`](crate::Prop) corresponding to the real world _resource_.
    /// All information needed to populate the _model_ should be derived from this tree.
    pub resource_prop_id: PropId,
    /// Contains the tree of [`Props`](crate::Prop) corresponding to code generation
    /// [`Funcs`](crate::Func).
    pub code_prop_id: PropId,
    /// Contains the tree of [`Props`](crate::Prop) corresponding to qualification
    /// [`Funcs`](crate::Func).
    pub qualification_prop_id: PropId,
    /// The [`Props`](crate::Prop) corresponding to the map of validation
    /// [`Funcs`](crate::Func) results.
    pub validation_prop_id: PropId,
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
        let root_prop_id = *root_prop.id();

        let si_specific_prop = Prop::new(ctx, "si", PropKind::Object, None).await?;
        si_specific_prop.set_parent_prop(ctx, root_prop_id).await?;

        let si_name_prop = Prop::new(ctx, "name", PropKind::String, None).await?;
        si_name_prop
            .set_parent_prop(ctx, *si_specific_prop.id())
            .await?;

        let domain_specific_prop = Prop::new(ctx, "domain", PropKind::Object, None).await?;
        domain_specific_prop
            .set_parent_prop(ctx, root_prop_id)
            .await?;

        let resource_specific_prop_id = Self::setup_resource(ctx, root_prop_id).await?;
        let code_specific_prop_id = Self::setup_code(ctx, root_prop_id).await?;
        let validation_prop_id = Self::setup_validation(ctx, root_prop_id).await?;
        let qualification_specific_prop_id = Self::setup_qualification(ctx, root_prop_id).await?;

        // Now that the structure is set up, we can populate default
        // AttributePrototypes & AttributeValues to be updated appropriately below.
        SchemaVariant::create_default_prototypes_and_values(ctx, schema_variant_id).await?;

        let root_context = AttributeContext::builder()
            .set_prop_id(root_prop_id)
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
            prop_id: root_prop_id,
            si_prop_id: *si_specific_prop.id(),
            domain_prop_id: *domain_specific_prop.id(),
            resource_prop_id: resource_specific_prop_id,
            code_prop_id: code_specific_prop_id,
            qualification_prop_id: qualification_specific_prop_id,
            validation_prop_id,
        })
    }

    async fn setup_resource(ctx: &DalContext, root_prop_id: PropId) -> SchemaVariantResult<PropId> {
        let mut resource_prop = Prop::new(ctx, "resource", PropKind::Object, None).await?;
        resource_prop.set_hidden(ctx, true).await?;
        resource_prop.set_parent_prop(ctx, root_prop_id).await?;
        let resource_prop_id = *resource_prop.id();

        let mut resource_status_prop = Prop::new(ctx, "status", PropKind::String, None).await?;
        resource_status_prop.set_hidden(ctx, true).await?;
        resource_status_prop
            .set_parent_prop(ctx, resource_prop_id)
            .await?;

        let mut resource_message_prop = Prop::new(ctx, "message", PropKind::String, None).await?;
        resource_message_prop.set_hidden(ctx, true).await?;
        resource_message_prop
            .set_parent_prop(ctx, resource_prop_id)
            .await?;

        let mut resource_logs_prop = Prop::new(ctx, "logs", PropKind::Array, None).await?;
        resource_logs_prop.set_hidden(ctx, true).await?;
        resource_logs_prop
            .set_parent_prop(ctx, resource_prop_id)
            .await?;

        let mut resource_logs_log_prop = Prop::new(ctx, "log", PropKind::String, None).await?;
        resource_logs_log_prop.set_hidden(ctx, true).await?;
        resource_logs_log_prop
            .set_parent_prop(ctx, *resource_logs_prop.id())
            .await?;

        let mut resource_value_prop = Prop::new(ctx, "value", PropKind::String, None).await?;
        resource_value_prop.set_hidden(ctx, true).await?;
        resource_value_prop
            .set_parent_prop(ctx, resource_prop_id)
            .await?;

        Ok(resource_prop_id)
    }

    async fn setup_code(ctx: &DalContext, root_prop_id: PropId) -> SchemaVariantResult<PropId> {
        let (code_map_prop_id, code_map_item_prop_id) =
            RootProp::insert_leaf_props(ctx, LeafKind::CodeGeneration, root_prop_id).await?;

        let mut child_code_prop = Prop::new(ctx, "code", PropKind::String, None).await?;
        child_code_prop.set_hidden(ctx, true).await?;
        child_code_prop
            .set_parent_prop(ctx, code_map_item_prop_id)
            .await?;

        let mut child_format_prop = Prop::new(ctx, "format", PropKind::String, None).await?;
        child_format_prop.set_hidden(ctx, true).await?;
        child_format_prop
            .set_parent_prop(ctx, code_map_item_prop_id)
            .await?;

        Ok(code_map_prop_id)
    }

    async fn setup_qualification(
        ctx: &DalContext,
        root_prop_id: PropId,
    ) -> SchemaVariantResult<PropId> {
        let (qualification_map_prop_id, qualification_map_item_prop_id) =
            RootProp::insert_leaf_props(ctx, LeafKind::Qualification, root_prop_id).await?;

        let mut child_qualified_prop = Prop::new(ctx, "qualified", PropKind::Boolean, None).await?;
        child_qualified_prop.set_hidden(ctx, true).await?;
        child_qualified_prop
            .set_parent_prop(ctx, qualification_map_item_prop_id)
            .await?;

        let mut child_message_prop = Prop::new(ctx, "message", PropKind::String, None).await?;
        child_message_prop.set_hidden(ctx, true).await?;
        child_message_prop
            .set_parent_prop(ctx, qualification_map_item_prop_id)
            .await?;

        Ok(qualification_map_prop_id)
    }

    async fn setup_validation(
        ctx: &DalContext,
        root_prop_id: PropId,
    ) -> SchemaVariantResult<PropId> {
        let (validation_map_prop_id, validation_map_item_prop_id) =
            RootProp::insert_leaf_props(ctx, LeafKind::Validation, root_prop_id).await?;

        let mut child_valid_prop = Prop::new(ctx, "valid", PropKind::Boolean, None).await?;
        child_valid_prop.set_hidden(ctx, true).await?;
        child_valid_prop
            .set_parent_prop(ctx, validation_map_item_prop_id)
            .await?;

        let mut child_messages = Prop::new(ctx, "message", PropKind::String, None).await?;
        child_messages.set_hidden(ctx, true).await?;
        child_messages
            .set_parent_prop(ctx, validation_map_item_prop_id)
            .await?;

        Ok(validation_map_prop_id)
    }

    async fn insert_leaf_props(
        ctx: &DalContext,
        leaf_kind: LeafKind,
        root_prop_id: PropId,
    ) -> SchemaVariantResult<(PropId, PropId)> {
        let (leaf_prop_name, leaf_item_prop_name) = leaf_kind.prop_names();

        let mut leaf_prop = Prop::new(ctx, leaf_prop_name, PropKind::Map, None).await?;
        leaf_prop.set_hidden(ctx, true).await?;
        leaf_prop.set_parent_prop(ctx, root_prop_id).await?;

        let mut leaf_item_prop =
            Prop::new(ctx, leaf_item_prop_name, PropKind::Object, None).await?;
        leaf_item_prop.set_hidden(ctx, true).await?;
        leaf_item_prop.set_parent_prop(ctx, *leaf_prop.id()).await?;

        Ok((*leaf_prop.id(), *leaf_item_prop.id()))
    }
}
