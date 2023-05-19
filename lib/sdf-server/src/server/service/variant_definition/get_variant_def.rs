use super::{SchemaVariantDefinitionError, SchemaVariantDefinitionResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{
    schema::variant::definition::{SchemaVariantDefinition, SchemaVariantDefinitionId},
    ComponentType, StandardModel, Timestamp, Visibility,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetVariantDefRequest {
    pub id: SchemaVariantDefinitionId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVariantDefResponse {
    pub id: SchemaVariantDefinitionId,
    pub name: String,
    pub menu_name: Option<String>,
    pub category: String,
    pub color: String,
    pub link: Option<String>,
    pub description: Option<String>,
    pub definition: String,
    pub variant_exists: bool,
    pub component_type: ComponentType,
    #[serde(flatten)]
    pub timestamp: Timestamp,
}

impl From<SchemaVariantDefinition> for GetVariantDefResponse {
    fn from(def: SchemaVariantDefinition) -> Self {
        GetVariantDefResponse {
            id: *def.id(),
            name: def.name().to_string(),
            menu_name: def.menu_name().map(|menu_name| menu_name.to_string()),
            category: def.category().to_string(),
            color: def.color().to_string(),
            link: def.link().map(|link| link.to_string()),
            definition: def.definition().to_string(),
            description: def.description().map(|d| d.to_string()),
            timestamp: def.timestamp().to_owned(),
            variant_exists: false, // This requires a database call, so this is a dummy value
            component_type: *def.component_type(),
        }
    }
}

pub async fn get_variant_def(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetVariantDefRequest>,
) -> SchemaVariantDefinitionResult<Json<GetVariantDefResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let variant_def = SchemaVariantDefinition::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(SchemaVariantDefinitionError::VariantDefinitionNotFound(
            request.id,
        ))?;

    let variant_exists = variant_def
        .existing_default_schema_variant_id(&ctx)
        .await?
        .is_some();
    let mut response: GetVariantDefResponse = variant_def.into();
    response.variant_exists = variant_exists;

    Ok(Json(response))
}
