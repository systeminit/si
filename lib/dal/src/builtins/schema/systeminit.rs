use crate::builtins::schema::MigrationDriver;
use crate::component::ComponentKind;
use crate::validation::Validation;
use crate::{BuiltinsResult, ComponentType, DalContext, PropKind, StandardModel};

const FRAME_NODE_COLOR: i64 = 0xFFFFFF;

impl MigrationDriver {
    pub async fn migrate_systeminit(&self, ctx: &DalContext) -> BuiltinsResult<()> {
        self.migrate_generic_frame(ctx, "Frames").await?;
        Ok(())
    }

    async fn migrate_generic_frame(
        &self,
        ctx: &DalContext,
        ui_menu_category: &str,
    ) -> BuiltinsResult<()> {
        let (schema, mut schema_variant, root_prop, _, _, _) = match self
            .create_schema_and_variant(
                ctx,
                "Generic Frame",
                None,
                ui_menu_category,
                ComponentKind::Standard,
                Some(FRAME_NODE_COLOR),
                None,
            )
            .await?
        {
            Some(tuple) => tuple,
            None => return Ok(()),
        };

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

        schema_variant
            .finalize(ctx, Some(ComponentType::ConfigurationFrame))
            .await?;

        // TODO - PAUL/VICTOR:
        // As this is an actual frame and has no alternative functionality
        // we want to disable the ability that a user can change the node type

        Ok(())
    }
}
