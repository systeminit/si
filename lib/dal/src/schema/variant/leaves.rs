//! This module contains all "leaves" that can be created underneath [`RootProp`](crate::RootProp)
//! subtrees for a [`SchemaVariant`](crate::SchemaVariant). In this domain, a "leaf" is considered
//! to an entry of a immediate child [`map`](crate::PropKind::Map) underneath "/root".

use crate::func::argument::FuncArgumentId;
use crate::schema::variant::{SchemaVariantError, SchemaVariantResult};
use crate::{
    AttributeContext, AttributePrototypeArgument, AttributeReadContext, AttributeValue,
    AttributeValueError, DalContext, Func, FuncBackendKind, FuncBackendResponseType, FuncError,
    FuncId, InternalProvider, InternalProviderError, InternalProviderId, PropId, SchemaVariant,
    SchemaVariantId, StandardModel,
};

/// This enum provides options for creating leaves underneath compatible subtrees of "/root" within
/// a [`SchemaVariant`](crate::SchemaVariant). Each compatible subtree starts with a
/// [`map`](crate::PropKind::Map) [`Prop`](crate::Prop) that can contain zero to many
/// [`object`](crate::PropKind::Object) entries. Each entry must leverage the same kind of
/// [`Func`](crate::Func) within the same [`map`](crate::PropKind::Map). The kind of
/// [`Func`](crate::Func) allowed corresponds to the [`LeafKind`].
#[derive(Clone, Copy, Debug)]
pub enum LeafKind {
    /// This variant corresponds to the "/root/code" subtree whose leaves leverage code generation
    /// [`Funcs`](crate::Func).
    CodeGeneration,
    /// This variant corresponds to the "/root/qualification" subtree whose leaves leverage
    /// qualification [`Funcs`](crate::Func).
    Qualification,
    /// This variant corresponds to the "/root/validation" subtree whose leaves store
    /// validation [`Func`](crate::Func) output
    Validation,
}

impl LeafKind {
    pub fn prop_names(&self) -> (&'static str, &'static str) {
        match self {
            LeafKind::CodeGeneration => ("code", "codeItem"),
            LeafKind::Qualification => ("qualification", "qualificationItem"),
            LeafKind::Validation => ("validation", "validationItem"),
        }
    }
}

impl From<LeafKind> for FuncBackendResponseType {
    fn from(leaf_kind: LeafKind) -> Self {
        match leaf_kind {
            LeafKind::CodeGeneration => FuncBackendResponseType::CodeGeneration,
            LeafKind::Qualification => FuncBackendResponseType::Qualification,
            LeafKind::Validation => FuncBackendResponseType::Validation,
        }
    }
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
        func_argument_internal_provider_id: InternalProviderId,
        schema_variant_id: SchemaVariantId,
        leaf_kind: LeafKind,
    ) -> SchemaVariantResult<(PropId, String)> {
        if schema_variant_id.is_none() {
            return Err(SchemaVariantError::InvalidSchemaVariant);
        }

        let func = Func::get_by_id(ctx, &func_id)
            .await?
            .ok_or(FuncError::NotFound(func_id))?;

        if func.backend_kind() != &FuncBackendKind::JsAttribute {
            return Err(SchemaVariantError::LeafFunctionMustBeJsAttribute(
                *func.id(),
            ));
        }

        if func.backend_response_type() != &leaf_kind.into() {
            return Err(SchemaVariantError::LeafFunctionMismatch(
                *func.backend_response_type(),
                leaf_kind,
            ));
        }

        // NOTE(nick): perhaps, considering only finalizing once and outside of this method. We only
        // need to finalize once if multiple leaves are added.
        SchemaVariant::finalize_for_id(ctx, schema_variant_id).await?;

        // Assemble the values we need to insert an object into the map.
        let item_prop =
            SchemaVariant::find_leaf_item_prop(ctx, schema_variant_id, leaf_kind).await?;

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

        // Insert an item into the map and setup its function. For most leaves, we'll just use the
        // func name. But the same validation might be defined for many props on one schema
        // variant, so we pair the prop id with the validation func name.
        let map_key = match leaf_kind {
            LeafKind::Validation => {
                let internal_provider =
                    InternalProvider::get_by_id(ctx, &func_argument_internal_provider_id)
                        .await?
                        .ok_or_else(|| {
                            InternalProviderError::NotFound(func_argument_internal_provider_id)
                        })?;

                format!("{};{}", internal_provider.prop_id(), func.name())
            }
            _ => func.name().to_owned(),
        };

        let inserted_attribute_value_id = AttributeValue::insert_for_context(
            ctx,
            insert_attribute_context,
            *map_attribute_value.id(),
            Some(serde_json::json![{}]),
            Some(map_key.clone()),
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

        AttributePrototypeArgument::new_for_intra_component(
            ctx,
            *inserted_attribute_prototype.id(),
            func_argument_id,
            func_argument_internal_provider_id,
        )
        .await?;

        // Return the prop id for the entire map so that its implicit internal provider can be
        // used for intelligence functions.
        Ok((*map_prop.id(), map_key))
    }

    /// Finds the implicit [`InternalProvider`](crate::InternalProvider)
    /// corresponding to the
    /// [`LeafKind`](crate::schema::variant::leaves::LeafKind), which is of kind
    /// [`map`](crate::PropKind::Map).
    ///
    /// _Note: the [`SchemaVariant`](crate::SchemaVariant) must be
    /// [`finalized`](crate::SchemaVariant::finalize()) (or [`finalized for
    /// id`](crate::SchemaVariant::finalize_for_id())) before running this
    /// query.
    pub async fn find_leaf_implicit_internal_provider(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        leaf_kind: LeafKind,
    ) -> SchemaVariantResult<InternalProvider> {
        let (prop_name, _) = leaf_kind.prop_names();

        Self::find_root_child_implicit_internal_provider(ctx, schema_variant_id, prop_name).await
    }
}
