use super::{is_variant_def_locked, SchemaVariantDefinitionError, SchemaVariantDefinitionResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::func::list_funcs::ListedFuncView;
use crate::service::func::{compile_return_types, compile_return_types_2};
use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use dal::{
    schema::variant::definition::{SchemaVariantDefinition, SchemaVariantDefinitionId},
    ComponentType, Func, SchemaVariant, SchemaVariantError, SchemaVariantId, StandardModel,
    Timestamp, Visibility,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetVariantDefRequest {
    pub id: SchemaVariantDefinitionId,
    pub has_secrets_enabled: bool,
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
    pub code: String,
    pub schema_variant_id: Option<SchemaVariantId>,
    pub component_type: ComponentType,
    pub funcs: Vec<ListedFuncView>,
    pub types: String,
    pub has_components: bool,
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
            description: def.description().map(|d| d.to_string()),
            code: "".to_string(), //TODO @stack72
            timestamp: def.timestamp().to_owned(),
            funcs: vec![],
            schema_variant_id: None,
            component_type: *def.component_type(),
            types: "".to_string(),
            has_components: false,
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

    let variant_id = variant_def.schema_variant_id().copied();

    let has_components = is_variant_def_locked(&ctx, &variant_def).await?;

    let asset_func = Func::get_by_id(&ctx, &variant_def.func_id()).await?.ok_or(
        SchemaVariantDefinitionError::FuncNotFound(variant_def.func_id()),
    )?;

    let mut response: GetVariantDefResponse = variant_def.clone().into();
    response.schema_variant_id = variant_id;
    response.has_components = has_components;

    response.code =
        asset_func
            .code_plaintext()?
            .ok_or(SchemaVariantDefinitionError::FuncIsEmpty(
                variant_def.func_id(),
            ))?;

    if let Some(variant_id) = variant_id {
        let deleted_ctx = ctx.clone_with_delete_visibility();
        if let Some(schema_variant) = SchemaVariant::get_by_id(&deleted_ctx, &variant_id).await? {
            if schema_variant.visibility().deleted_at.is_some() {
                return Err(SchemaVariantError::NotFound(*schema_variant.id()))?;
            }
        }

        response.funcs = SchemaVariant::all_funcs(&ctx, variant_id)
            .await?
            .iter()
            .filter_map(|func| match func.try_into() {
                Ok(func_variant) => Some(ListedFuncView {
                    id: func.id().to_owned(),
                    handler: func.handler().map(|handler| handler.to_owned()),
                    variant: func_variant,
                    name: func.name().into(),
                    display_name: func
                        .display_name()
                        .map(Into::into)
                        .or_else(|| Some(func.name().to_string())),
                    is_builtin: func.builtin(),
                }),
                Err(_) => None,
            })
            .collect();
    }

    let types = if request.has_secrets_enabled {
        compile_return_types_2(
            *asset_func.backend_response_type(),
            *asset_func.backend_kind(),
        )
    } else {
        compile_return_types(
            *asset_func.backend_response_type(),
            *asset_func.backend_kind(),
        )
    };

    response.types = types.to_string();

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
