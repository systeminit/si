use crate::builtins::schema::MigrationDriver;
use crate::component::ComponentKind;
use crate::schema::variant::definition::SchemaVariantDefinitionMetadataJson;
use crate::{BuiltinsResult, ComponentType, DalContext};

impl MigrationDriver {
    pub async fn migrate_systeminit_generic_frame(
        &self,
        ctx: &DalContext,
        ui_menu_category: &str,
        node_color: &str,
    ) -> BuiltinsResult<()> {
        let (_schema, mut schema_variant, root_prop, _, _, _) = match self
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

        let mut type_prop = self
            .find_child_prop_by_name(ctx, root_prop.si_prop_id, "type")
            .await?;
        type_prop.set_hidden(ctx, true).await?;

        schema_variant
            .finalize(ctx, Some(ComponentType::ConfigurationFrame))
            .await?;

        Ok(())
    }
}
