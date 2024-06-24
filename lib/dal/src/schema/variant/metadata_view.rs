use serde::{Deserialize, Serialize};

use crate::schema::variant::SchemaVariantResult;
use crate::{
    ComponentType, DalContext, Schema, SchemaId, SchemaVariant, SchemaVariantId, Timestamp,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantMetadataView {
    id: SchemaId,
    default_schema_variant_id: SchemaVariantId,
    name: String,
    category: String,
    #[serde(alias = "display_name")]
    display_name: String,
    color: String,
    component_type: ComponentType,
    is_locked: bool,
    link: Option<String>,
    description: Option<String>,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl SchemaVariantMetadataView {
    pub async fn list(ctx: &DalContext) -> SchemaVariantResult<Vec<Self>> {
        let mut views = Vec::new();

        let schemas = Schema::list(ctx).await?;
        for schema in schemas {
            let default_schema_variant =
                SchemaVariant::get_default_for_schema(ctx, schema.id()).await?;
            let name = schema.name.to_owned();
            let display_name = match default_schema_variant.display_name() {
                Some(display_name) => display_name,
                _ => name.clone(),
            };
            views.push(SchemaVariantMetadataView {
                id: schema.id,
                default_schema_variant_id: default_schema_variant.id,
                name,
                category: default_schema_variant.category.to_owned(),
                color: default_schema_variant.get_color(ctx).await?,
                timestamp: default_schema_variant.timestamp.to_owned(),
                component_type: default_schema_variant
                    .get_type(ctx)
                    .await?
                    .unwrap_or(ComponentType::Component),
                link: default_schema_variant.link.to_owned(),
                description: default_schema_variant.description(),
                display_name,
                is_locked: default_schema_variant.is_locked(),
            })
        }

        Ok(views)
    }
}
