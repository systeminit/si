//! This module contains all "leaves" that can be created underneath [`RootProp`](crate::RootProp)
//! subtrees for a [`SchemaVariant`](crate::SchemaVariant). In this domain, a "leaf" is considered
//! to an entry of a immediate child [`map`](crate::PropKind::Map) underneath "/root".

use crate::func::argument::FuncArgumentId;
use crate::schema::variant::{SchemaVariantError, SchemaVariantResult};
use crate::{
    AttributeContext, AttributePrototypeArgument, AttributeReadContext, AttributeValue,
    AttributeValueError, DalContext, Func, FuncError, FuncId, PropId, SchemaVariant,
    SchemaVariantId, StandardModel,
};

/// This enum provides options for creating leaves underneath compatible subtrees of "/root" within
/// a [`SchemaVariant`](crate::SchemaVariant). Each compatible subtree starts with a
/// [`map`](crate::PropKind::Map) [`Prop`](crate::Prop) that can contain zero to many
/// [`object`](crate::PropKind::Object) entries. Each entry must leverage the same kind of
/// [`Func`](crate::Func) within the same [`map`](crate::PropKind::Map). The kind of
/// [`Func`](crate::Func) allowed corresponds to the [`LeafKind`].
pub enum LeafKind {
    /// This variant corresponds to the "/root/code" subtree whose leaves leverage code generation
    /// [`Funcs`](crate::Func).
    CodeGeneration,
    /// This variant corresponds to the "/root/qualification" subtree whose leaves leverage
    /// qualification [`Funcs`](crate::Func).
    Qualification,
}

impl SchemaVariant {
    /// Insert an [`object`](crate::PropKind::Object) entry into a "/root" subtree of
    /// [`map`](crate::PropKind::Map) with a [`Func`](crate::Func) that matches the provided
    /// [`LeafKind`] in order to populate the subtree entry.
    ///
    /// The [`PropId`](crate::Prop) for the child [`map`](crate::PropKind::Map) of "/root"
    /// corresponding to the [`LeafKind`] is returned.
    pub async fn add_leaf(
        ctx: &DalContext,
        func_id: FuncId,
        func_argument_id: FuncArgumentId,
        schema_variant_id: SchemaVariantId,
        leaf_kind: LeafKind,
    ) -> SchemaVariantResult<PropId> {
        if schema_variant_id.is_none() {
            return Err(SchemaVariantError::InvalidSchemaVariant);
        }

        // TODO(nick): check if the func is valid for the leaf kind. We "get" the func here
        // since we not only need it for the name later, but we want to perform that check upfront
        // to save time if the func kind is invalid for the leaf.
        let func = Func::get_by_id(ctx, &func_id)
            .await?
            .ok_or(FuncError::NotFound(func_id))?;

        // NOTE(nick): perhaps, considering only finalizing once and outside of this method. We only
        // need to finalize once if multiple leaves are added.
        SchemaVariant::finalize_for_id(ctx, schema_variant_id).await?;

        // Assemble the values we need to insert an object into the map.
        let item_prop = match leaf_kind {
            LeafKind::CodeGeneration => {
                SchemaVariant::find_code_item_prop(ctx, schema_variant_id).await?
            }
            LeafKind::Qualification => {
                SchemaVariant::find_qualification_item_prop(ctx, schema_variant_id).await?
            }
        };

        // NOTE(nick): we should consider getting the parent and the item at the same time.
        let map_prop = item_prop
            .parent_prop(ctx)
            .await?
            .ok_or_else(|| SchemaVariantError::ParentPropNotFound(*item_prop.id()))?;
        let map_attribute_read_context = AttributeReadContext::default_with_prop(*map_prop.id());
        let map_attribute_value = AttributeValue::find_for_context(ctx, map_attribute_read_context)
            .await?
            .ok_or(AttributeValueError::NotFoundForReadContext(
                map_attribute_read_context,
            ))?;
        let insert_attribute_context = AttributeContext::builder()
            .set_prop_id(*item_prop.id())
            .to_context()?;

        // Insert an item into the map and setup its function. The new entry is named after the func
        // name since func names must be unique for a given tenancy and visibility. If that changes,
        // then this will break.
        let inserted_attribute_value_id = AttributeValue::insert_for_context(
            ctx,
            insert_attribute_context,
            *map_attribute_value.id(),
            Some(serde_json::json![{}]),
            Some(func.name().to_string()),
        )
        .await?;
        let inserted_attribute_value = AttributeValue::get_by_id(ctx, &inserted_attribute_value_id)
            .await?
            .ok_or_else(|| {
                AttributeValueError::NotFound(inserted_attribute_value_id, *ctx.visibility())
            })?;
        let mut inserted_attribute_prototype = inserted_attribute_value
            .attribute_prototype(ctx)
            .await?
            .ok_or(AttributeValueError::MissingAttributePrototype)?;
        inserted_attribute_prototype
            .set_func_id(ctx, func_id)
            .await?;

        // NOTE(nick): there will likely need to be divergent behavior here for validations.
        // Code generation and qualification rely on "/root/domain".
        let domain_implicit_internal_provider =
            SchemaVariant::find_domain_implicit_internal_provider(ctx, schema_variant_id).await?;
        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *inserted_attribute_prototype.id(),
            func_argument_id,
            *domain_implicit_internal_provider.id(),
        )
        .await?;

        // Return the prop id for the entire map so that its implicit internal provider can be
        // used for intelligence functions.
        Ok(*map_prop.id())
    }
}
