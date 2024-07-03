use telemetry::prelude::*;

use crate::{
    action::prototype::{ActionKind, ActionPrototype},
    func::binding::FuncBindingsError,
    prop::PropPath,
    ActionPrototypeId, DalContext, Func, FuncId, Prop, SchemaVariant, SchemaVariantError,
    SchemaVariantId,
};

use super::{FuncBinding, FuncBindings, FuncBindingsResult};
pub struct ActionBinding;

impl ActionBinding {
    pub(crate) async fn assemble_action_bindings(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingsResult<Vec<FuncBinding>> {
        let mut bindings = Vec::new();
        for (schema_variant_id, action_prototype_id) in
            SchemaVariant::list_with_action_prototypes_for_action_func(ctx, func_id).await?
        {
            let action_prototype = ActionPrototype::get_by_id(ctx, action_prototype_id).await?;
            bindings.push(FuncBinding::Action {
                kind: action_prototype.kind,
                schema_variant_id,
                action_prototype_id,
                func_id,
            });
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
    ) -> FuncBindingsResult<FuncBindings> {
        let schema_variant_id =
            ActionPrototype::schema_variant_id(ctx, action_prototype_id).await?;
        let func_id = ActionPrototype::func_id(ctx, action_prototype_id).await?;
        let func = Func::get_by_id_or_error(ctx, func_id).await?;
        // delete and recreate the prototype
        //brit todo: there might be existing actions enqueued, we should find them and reassociate the prototype
        ActionPrototype::remove(ctx, action_prototype_id).await?;
        ActionPrototype::new(
            ctx,
            kind,
            func.name.to_owned(),
            func.description.to_owned(),
            schema_variant_id,
            func_id,
        )
        .await?;
        let new_binding = FuncBindings::from_func_id(ctx, func_id).await?;
        Ok(new_binding)
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
    ) -> FuncBindingsResult<FuncBindings> {
        if action_kind != ActionKind::Manual {
            let existing_action_prototypes_for_variant =
                ActionPrototype::for_variant(ctx, schema_variant_id).await?;
            if existing_action_prototypes_for_variant
                .iter()
                .any(|p| p.kind == action_kind)
            {
                return Err(FuncBindingsError::ActionKindAlreadyExists(
                    action_kind,
                    schema_variant_id,
                ));
            }
        }

        let func = Func::get_by_id_or_error(ctx, func_id).await?;
        ActionPrototype::new(
            ctx,
            action_kind,
            func.name.to_owned(),
            func.description.to_owned(),
            schema_variant_id,
            func.id,
        )
        .await?;

        let new_binding = FuncBindings::from_func_id(ctx, func_id).await?;
        Ok(new_binding)
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
    ) -> FuncBindingsResult<FuncBindings> {
        let func_id = ActionPrototype::func_id(ctx, action_prototype_id).await?;

        ActionPrototype::remove(ctx, action_prototype_id).await?;

        FuncBindings::from_func_id(ctx, func_id).await
    }

    pub(crate) async fn compile_action_types(
        ctx: &DalContext,
        func_id: FuncId,
    ) -> FuncBindingsResult<String> {
        let schema_variant_ids =
            SchemaVariant::list_with_action_prototypes_for_action_func(ctx, func_id).await?;
        let mut ts_types = vec![];
        for (variant_id, _) in schema_variant_ids {
            let path = "root";
            let prop = match Prop::find_prop_by_path(ctx, variant_id, &PropPath::new([path])).await
            {
                Ok(prop_id) => prop_id,
                Err(_) => Err(SchemaVariantError::PropNotFoundAtPath(
                    variant_id,
                    path.to_string(),
                ))?,
            };
            ts_types.push(prop.ts_type(ctx).await?)
        }
        Ok(format!(
            "type Input {{
            kind: 'standard';
            properties: {};
        }}",
            ts_types.join(" | "),
        ))
    }
}
