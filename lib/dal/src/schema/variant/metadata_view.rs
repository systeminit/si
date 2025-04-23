use serde::{
    Deserialize,
    Serialize,
};

use crate::{
    ComponentType,
    DalContext,
    Schema,
    SchemaId,
    SchemaVariant,
    SchemaVariantId,
    Timestamp,
    schema::variant::SchemaVariantResult,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantMetadataView {
    id: SchemaId,
    default_schema_variant_id: SchemaVariantId,
    schema_name: String,
    // TODO rename this to version without breaking the frontend
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
                SchemaVariant::default_for_schema(ctx, schema.id()).await?;
            views.push(SchemaVariantMetadataView {
                id: schema.id,
                default_schema_variant_id: default_schema_variant.id,
                schema_name: schema.name.to_owned(),
                name: default_schema_variant.version.to_owned(),
                category: default_schema_variant.category.to_owned(),
                color: default_schema_variant.get_color(ctx).await?,
                timestamp: default_schema_variant.timestamp.to_owned(),
                component_type: default_schema_variant
                    .get_type(ctx)
                    .await?
                    .unwrap_or(ComponentType::Component),
                link: default_schema_variant.link.to_owned(),
                description: default_schema_variant.description.clone(),
                display_name: default_schema_variant.display_name().to_string(),
                is_locked: default_schema_variant.is_locked(),
            })
        }

        Ok(views)
    }
}
