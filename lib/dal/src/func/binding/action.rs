use serde::{
    Deserialize,
    Serialize,
};
use telemetry::prelude::*;

use super::{
    EventualParent,
    FuncBinding,
    FuncBindingResult,
};
use crate::{
    ActionPrototypeId,
    DalContext,
    Func,
    FuncId,
    Prop,
    SchemaVariant,
    SchemaVariantError,
    SchemaVariantId,
    action::prototype::{
        ActionKind,
        ActionPrototype,
        ActionPrototypeParent,
    },
    func::binding::FuncBindingError,
    prop::PropPath,
};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
pub struct ActionBinding {
    // unique ids
    pub schema_variant_id: SchemaVariantId,
    pub action_prototype_id: ActionPrototypeId,
    pub func_id: FuncId,
    //thing that can be updated
    pub kind: ActionKind,
}

impl ActionBinding {
    pub async fn assemble_action_bindings(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        let mut bindings = vec![];
        for (schema_variant_id, action_prototype_id) in
            SchemaVariant::list_with_action_prototypes_for_action_func(ctx, func_id).await?
        {
            let action_prototype = ActionPrototype::get_by_id(ctx, action_prototype_id).await?;
            bindings.push(FuncBinding::Action(ActionBinding {
                schema_variant_id,
                action_prototype_id,
                func_id,
                kind: action_prototype.kind,
            }));
        }
        Ok(bindings)
    }

    /// Updates the [`ActionKind`] for a given [`ActionPrototypeId`] by removing the existing [`ActionPrototype`]
    /// and creating a new one in its place
    #[instrument(
        level = "info",
        skip(ctx),
        name = "func.binding.action.update_action_binding"
    )]
    pub async fn update_action_binding(
        ctx: &DalContext,
        action_prototype_id: ActionPrototypeId,
        kind: ActionKind,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        let prototype_parent = ActionPrototype::parentage(ctx, action_prototype_id).await?;
        let func_id = ActionPrototype::func_id(ctx, action_prototype_id).await?;
        let func = Func::get_by_id(ctx, func_id).await?; // delete and recreate the prototype

        ActionPrototype::remove(ctx, action_prototype_id).await?;
        ActionPrototype::new(
            ctx,
            kind,
            func.name.to_owned(),
            func.description.to_owned(),
            prototype_parent,
            func_id,
        )
        .await?;

        FuncBinding::for_func_id(ctx, func_id).await
    }

    /// Creates an [`ActionPrototype`] with the specified [`ActionKind`] for a given [`SchemaVariantId`]
    /// Checks to ensure there isn't already an Action with that Kind in the case of Create/Delete/Refresh
    #[instrument(
        level = "info",
        skip(ctx),
        name = "func.binding.action.create_action_binding"
    )]
    pub async fn create_action_binding(
        ctx: &DalContext,
        func_id: FuncId,
        action_kind: ActionKind,
        schema_variant_id: SchemaVariantId,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        // don't add binding if parent is locked
        SchemaVariant::error_if_locked(ctx, schema_variant_id).await?;

        // If not manual, don't create duplicate prototypes for variant AND actionKind
        if action_kind != ActionKind::Manual {
            let existing_action_prototypes_for_variant =
                ActionPrototype::for_variant(ctx, schema_variant_id).await?;
            if existing_action_prototypes_for_variant
                .iter()
                .any(|p| p.kind == action_kind)
            {
                return Err(FuncBindingError::ActionKindAlreadyExists(
                    action_kind,
                    schema_variant_id,
                ));
            }
        }

        let func = Func::get_by_id(ctx, func_id).await?;
        ActionPrototype::new(
            ctx,
            action_kind,
            func.name.to_owned(),
            func.description.to_owned(),
            ActionPrototypeParent::SchemaVariant(schema_variant_id),
            func.id,
        )
        .await?;

        FuncBinding::for_func_id(ctx, func_id).await
    }

    /// Deletes an [`ActionPrototype`] by the [`ActionPrototypeId`]
    #[instrument(
        level = "info",
        skip(ctx),
        name = "func.binding.action.delete_action_binding"
    )]
    pub async fn delete_action_binding(
        ctx: &DalContext,
        action_prototype_id: ActionPrototypeId,
    ) -> FuncBindingResult<EventualParent> {
        // don't delete binding if parent is locked
        let parent_id = ActionPrototype::parentage(ctx, action_prototype_id).await?;

        if let ActionPrototypeParent::SchemaVariant(schema_variant_id) = parent_id {
            SchemaVariant::error_if_locked(ctx, schema_variant_id).await?;
        }

        ActionPrototype::remove(ctx, action_prototype_id).await?;

        Ok(parent_id.into())
    }

    pub(crate) async fn compile_action_types(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingResult<String> {
        let schema_variant_ids =
            SchemaVariant::list_with_action_prototypes_for_action_func(ctx, func_id).await?;
        let mut ts_types = vec![];
        for (variant_id, _) in schema_variant_ids {
            let path = "root";

            let prop = Prop::find_prop_by_path(ctx, variant_id, &PropPath::new([path]))
                .await
                .map_err(|_| {
                    SchemaVariantError::PropNotFoundAtPath(variant_id, path.to_string())
                })?;
            ts_types.push(Prop::ts_type(ctx, prop.id()).await?)
        }
        Ok(format!(
            "type Input {{
            kind: 'standard';
            properties: {};
        }}",
            ts_types.join(" | "),
        ))
    }

    pub(crate) async fn port_binding_to_new_func(
        &self,
        ctx: &DalContext,
        new_func_id: FuncId,
    ) -> FuncBindingResult<Vec<FuncBinding>> {
        // cache existing config info
        let schema_variant_id = self.schema_variant_id;
        let action_kind = self.kind;
        // remove the existing action prototype and recreate it for the new func id

        ActionPrototype::remove(ctx, self.action_prototype_id).await?;

        Self::create_action_binding(ctx, new_func_id, action_kind, schema_variant_id).await?;

        FuncBinding::for_func_id(ctx, new_func_id).await
    }
}
