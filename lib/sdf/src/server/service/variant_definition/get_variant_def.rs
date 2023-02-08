use super::{SchemaVariantDefinitionError, SchemaVariantDefinitionResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{
    component::ComponentKind,
    schema::variant::definition::{SchemaVariantDefinition, SchemaVariantDefinitionId},
    StandardModel, Visibility,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetVariantDefRequest {
    pub id: SchemaVariantDefinitionId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetVariantDefResponse {
    pub id: SchemaVariantDefinitionId,
    pub name: String,
    pub menu_name: Option<String>,
    pub category: String,
    pub color: String,
    pub component_kind: ComponentKind,
    pub link: Option<String>,
    pub definition: String,
}

impl From<SchemaVariantDefinition> for GetVariantDefResponse {
    fn from(variant: SchemaVariantDefinition) -> Self {
        GetVariantDefResponse {
            id: *variant.id(),
            name: variant.name().to_string(),
            menu_name: variant.menu_name().map(|menu_name| menu_name.to_string()),
            category: variant.category().to_string(),
            color: variant.color().to_string(),
            component_kind: *variant.component_kind(),
            link: variant.link().map(|link| link.to_string()),
            definition: variant.definition().to_string(),
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
        .ok_or(SchemaVariantDefinitionError::VariantDefnitionNotFound(
            request.id,
        ))?;

    Ok(Json(variant_def.into()))
}
