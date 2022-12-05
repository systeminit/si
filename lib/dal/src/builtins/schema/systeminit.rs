use crate::builtins::schema::MigrationDriver;
use crate::component::ComponentKind;
use crate::validation::Validation;
use crate::{
    schema::SchemaUiMenu, BuiltinsError, BuiltinsResult, DalContext, DiagramKind, InternalProvider,
    PropKind, SchemaError, SchemaKind, SocketArity, StandardModel,
};

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

        // Sockets
        let identity_func_item = self
            .get_func_item("si:identity")
            .ok_or(BuiltinsError::FuncNotFoundInMigrationCache("si:identity"))?;

        let (_docker_hub_credential_explicit_internal_provider, _input_socket) =
            InternalProvider::new_explicit_with_socket(
                ctx,
                *schema_variant.id(),
                "Frame",
                identity_func_item.func_id,
                identity_func_item.func_binding_id,
                identity_func_item.func_binding_return_value_id,
                SocketArity::Many,
                DiagramKind::Configuration,
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

        schema_variant.finalize(ctx).await?;

        Ok(())
    }
}
