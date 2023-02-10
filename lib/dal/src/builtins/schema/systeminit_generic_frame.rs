use crate::builtins::schema::MigrationDriver;
use crate::component::ComponentKind;
use crate::schema::variant::definition::SchemaVariantDefinitionMetadataJson;
use crate::validation::Validation;
use crate::{BuiltinsResult, ComponentType, DalContext, PropKind, StandardModel};

impl MigrationDriver {
    pub async fn migrate_systeminit_generic_frame(
        &self,
        ctx: &DalContext,
        ui_menu_category: &str,
        node_color: &str,
    ) -> BuiltinsResult<()> {
        let (schema, mut schema_variant, root_prop, _, _, _) = match self
            .create_schema_and_variant(
                ctx,
                SchemaVariantDefinitionMetadataJson::new(
                    "Generic Frame",
                    None::<&str>,
                    ui_menu_category,
                    node_color,
                    ComponentKind::Standard,
                    None,
                    None,
                ),
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
