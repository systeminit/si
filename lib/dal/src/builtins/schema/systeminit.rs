use crate::builtins::schema::MigrationDriver;
use crate::component::ComponentKind;
use crate::func::argument::FuncArgument;
use crate::schema::variant::leaves::LeafKind;
use crate::{
    schema::SchemaUiMenu, BuiltinsResult, DalContext, PropKind, SchemaError, SchemaKind,
    SchemaVariant, StandardModel,
};
use crate::{BuiltinsError, Func, InternalProvider};

const FRAME_NODE_COLOR: i64 = 0xFFFFFF;

impl MigrationDriver {
    pub async fn migrate_systeminit(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        self.migrate_generic_frame(ctx).await?;
        Ok(())
    }

    async fn migrate_generic_frame(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        let (schema, schema_variant, root_prop, _) = match self
            .create_schema_and_variant(
                ctx,
                "Generic Frame",
                SchemaKind::Configuration,
                ComponentKind::Standard,
                Some(FRAME_NODE_COLOR),
                None,
            )
            .await?
        {
            Some(tuple) => tuple,
            None => return Ok(()),
        };

        // Diagram and UI Menu
        let diagram_kind = schema
            .diagram_kind()
            .ok_or_else(|| SchemaError::NoDiagramKindForSchemaKind(*schema.kind()))?;
        let ui_menu = SchemaUiMenu::new(ctx, "Generic Frame", "Frames", &diagram_kind).await?;
        ui_menu.set_schema(ctx, schema.id()).await?;

        // Prop and validation creation
        let color_prop = self
            .create_prop(
                ctx,
                "Color",
                PropKind::String,
                None,
                Some(root_prop.si_prop_id),
                None,
            )
            .await?;

        schema_variant.finalize(ctx).await?;
        let color_ip = InternalProvider::find_for_prop(ctx, *color_prop.id())
            .await?
            .ok_or_else(|| {
                BuiltinsError::ImplicitInternalProviderNotFoundForProp(*color_prop.id())
            })?;

        let validation_func_name = "si:validationIsValidHexColor".to_string();
        let validation_func = Func::find_by_attr(ctx, "name", &validation_func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(validation_func_name.to_string()))?;
        let validation_func_arg =
            FuncArgument::find_by_name_for_func(ctx, "value", *validation_func.id())
                .await?
                .ok_or_else(|| {
                    BuiltinsError::BuiltinMissingFuncArgument(
                        validation_func_name.clone(),
                        "value".to_string(),
                    )
                })?;

        SchemaVariant::add_leaf(
            ctx,
            *validation_func.id(),
            *validation_func_arg.id(),
            *color_ip.id(),
            *schema_variant.id(),
            LeafKind::Validation,
        )
        .await?;

        self.finalize_schema_variant(ctx, &schema_variant, &root_prop)
            .await?;

        // set the component as a configuration frame
        let si_type_prop = self
            .find_child_prop_by_name(ctx, root_prop.si_prop_id, "type")
            .await?;

        self.set_default_value_for_prop(
            ctx,
            *si_type_prop.id(),
            serde_json::json!["configurationFrame"],
        )
        .await?;

        // TODO - PAUL/VICTOR:
        // As this is an actual frame and has no alternative functionality
        // we want to disable the ability that a user can change the node type

        Ok(())
    }
}
