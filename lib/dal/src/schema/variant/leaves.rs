//! This module contains all "leaves" that can be created underneath [`RootProp`](crate::RootProp)
//! subtrees for a [`SchemaVariant`](crate::SchemaVariant). In this domain, a "leaf" is considered
//! to an entry of a immediate child [`map`](crate::PropKind::Map) underneath "/root".

use telemetry::prelude::*;

use super::{
    SchemaVariantError,
    SchemaVariantResult,
};
use crate::{
    AttributePrototype,
    AttributePrototypeId,
    DalContext,
    Func,
    FuncBackendKind,
    FuncId,
    Prop,
    PropId,
    SchemaVariant,
    SchemaVariantId,
    attribute::prototype::argument::AttributePrototypeArgument,
    func::leaf::{
        LeafInput,
        LeafKind,
    },
    workspace_snapshot::edge_weight::EdgeWeightKind,
};

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
        schema_variant_id: SchemaVariantId,
        leaf_kind: LeafKind,
        inputs: Vec<LeafInput>,
    ) -> SchemaVariantResult<(PropId, AttributePrototypeId)> {
        let func = Func::get_by_id(ctx, func_id).await?;

        // Ensure the func matches what we need.
        if func.backend_kind != FuncBackendKind::JsAttribute {
            return Err(SchemaVariantError::LeafFunctionMustBeJsAttribute(func.id));
        }
        if func.backend_response_type != leaf_kind.into() {
            return Err(SchemaVariantError::LeafFunctionMismatch(
                func_id,
                func.backend_response_type,
                leaf_kind,
            ));
        }

        // The key is the name of the func. This assume func names are unique.
        let key = Some(func.name.to_owned());

        // Gather the item and map props.
        let item_prop_id =
            SchemaVariant::find_leaf_item_prop(ctx, schema_variant_id, leaf_kind).await?;
        let map_prop_id = Prop::parent_prop_id_by_id(ctx, item_prop_id)
            .await?
            .ok_or_else(|| SchemaVariantError::LeafMapPropNotFound(item_prop_id))?;

        // Clear existing prototypes as needed.
        if let Some(prototype_id) = AttributePrototype::find_for_prop(ctx, item_prop_id, &None)
            .await
            .map_err(Box::new)?
        {
            debug!(%prototype_id, %item_prop_id, "removing attribute prototype without key for leaf item prop");
            AttributePrototype::remove(ctx, prototype_id)
                .await
                .map_err(Box::new)?;
        }
        if let Some(prototype_id) = AttributePrototype::find_for_prop(ctx, item_prop_id, &key)
            .await
            .map_err(Box::new)?
        {
            debug!(%prototype_id, %item_prop_id, ?key, "removing attribute prototype for leaf item prop and key that already exists");
            AttributePrototype::remove(ctx, prototype_id)
                .await
                .map_err(Box::new)?;
        }

        // Create the new prototype and add an edge to the item prop using a populated key.
        let attribute_prototype_id = AttributePrototype::new(ctx, func_id)
            .await
            .map_err(Box::new)?
            .id();
        Prop::add_edge_to_attribute_prototype(
            ctx,
            item_prop_id,
            attribute_prototype_id,
            EdgeWeightKind::Prototype(key),
        )
        .await?;

        // Now that we have the prototype, we can process all inputs and create an attribute prototype argument that
        // sources its value from the input prop.
        for input in inputs {
            let input_prop_id = SchemaVariant::find_root_child_prop_id(
                ctx,
                schema_variant_id,
                input.location.into(),
            )
            .await?;

            AttributePrototypeArgument::new(
                ctx,
                attribute_prototype_id,
                input.func_argument_id,
                input_prop_id,
            )
            .await?;
        }

        Ok((map_prop_id, attribute_prototype_id))
    }
}
