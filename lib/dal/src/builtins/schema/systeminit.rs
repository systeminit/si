use crate::builtins::schema::MigrationDriver;
use crate::component::ComponentKind;
use crate::validation::Validation;
use crate::{
    schema::SchemaUiMenu, BuiltinsResult, DalContext, PropKind, SchemaError, SchemaKind,
    StandardModel,
};

const FRAME_NODE_COLOR: i64 = 0xFFFFFF;

impl MigrationDriver {
    pub async fn migrate_systeminit(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        self.migrate_generic_frame(ctx).await?;
        Ok(())
    }

    async fn migrate_generic_frame(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        let (schema, schema_variant, root_prop, _, _, _) = match self
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

        self.create_validation(
            ctx,
            Validation::StringIsHexColor { value: None },
            *color_prop.id(),
            *schema.id(),
            *schema_variant.id(),
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
