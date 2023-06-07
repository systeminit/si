use super::{SchemaVariantDefinitionError, SchemaVariantDefinitionResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::func::list_funcs::ListedFuncView;
use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use dal::SchemaVariant;
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
    pub funcs: Vec<ListedFuncView>,
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
            funcs: vec![],
            variant_exists: false, // This requires a database call, so this is a dummy value
            component_type: *def.component_type(),
        }
    }
}

pub async fn get_variant_def(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Query(request): Query<GetVariantDefRequest>,
) -> SchemaVariantDefinitionResult<Json<GetVariantDefResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let variant_def = SchemaVariantDefinition::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(SchemaVariantDefinitionError::VariantDefinitionNotFound(
            request.id,
        ))?;

    let variant = variant_def.existing_default_schema_variant_id(&ctx).await?;

    let mut response: GetVariantDefResponse = variant_def.clone().into();
    response.variant_exists = variant.is_some();

    if let Some(variant_id) = variant {
        response.funcs = SchemaVariant::all_funcs(&ctx, variant_id)
            .await?
            .iter()
            .filter_map(|func| match func.try_into() {
                Ok(variant) => Some(ListedFuncView {
                    id: func.id().to_owned(),
                    handler: func.handler().map(|handler| handler.to_owned()),
                    variant,
                    name: func.name().into(),
                    display_name: func.display_name().map(Into::into),
                    is_builtin: func.builtin(),
                }),
                Err(_) => None,
            })
            .collect();
    }

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "get_variant_def",
        serde_json::json!({
                    "variant_def_name": variant_def.name(),
                    "variant_def_category": variant_def.category(),
                    "variant_def_menu_name": variant_def.menu_name(),
                    "variant_def_id": variant_def.id(),
                    "variant_def_component_type": variant_def.component_type(),
        }),
    );

    Ok(Json(response))
}
