use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use dal::{ComponentType, Func, SchemaVariant, SchemaVariantId, Timestamp, Visibility};
use serde::{Deserialize, Serialize};

// use super::{is_variant_def_locked, SchemaVariantDefinitionError, SchemaVariantDefinitionResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
// use crate::server::tracking::track;
use crate::service::func::compile_return_types;
use crate::service::func::list_funcs::ListedFuncView;
use crate::service::variant::{SchemaVariantError, SchemaVariantResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetVariantRequest {
    pub id: SchemaVariantId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVariantResponse {
    pub id: SchemaVariantId,
    pub name: String,
    pub menu_name: Option<String>,
    pub category: String,
    pub color: String,
    pub link: Option<String>,
    pub description: Option<String>,
    pub code: String,
    pub component_type: ComponentType,
    pub funcs: Vec<ListedFuncView>,
    pub types: String,
    pub has_components: bool,
    #[serde(flatten)]
    pub timestamp: Timestamp,
}

pub async fn get_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Query(request): Query<GetVariantRequest>,
) -> SchemaVariantResult<Json<GetVariantResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let variant = SchemaVariant::get_by_id(&ctx, request.id).await?;
    let schema = variant.schema(&ctx).await?;

    let mut response: GetVariantResponse = GetVariantResponse {
        id: request.id,
        name: schema.name().into(),
        menu_name: variant.display_name(),
        category: variant.category().into(),
        color: variant.get_color(&ctx).await?,
        link: variant.link(),
        description: variant.description(),
        component_type: variant.component_type(),
        timestamp: variant.timestamp(),
        // Will be set elsewhere
        code: "".to_string(),
        funcs: vec![],
        types: "".to_string(),
        has_components: false,
    };

    if let Some(authoring_func) = variant.asset_func_id() {
        let asset_func = Func::get_by_id(&ctx, authoring_func).await?;

        response.code = asset_func
            .code_plaintext()?
            .ok_or(SchemaVariantError::FuncIsEmpty(asset_func.id))?;

        response.types =
            compile_return_types(asset_func.backend_response_type, asset_func.backend_kind)
                .to_string();
    }

    // let has_components = is_variant_def_locked(&ctx, &variant_def).await?;
    // response.has_components = has_components;

    response.funcs = SchemaVariant::all_funcs(&ctx, request.id)
        .await?
        .iter()
        .filter_map(|func| match func.try_into() {
            Ok(func_variant) => Some(ListedFuncView {
                id: func.id,
                handler: func.handler.clone().map(|handler| handler.to_owned()),
                variant: func_variant,
                name: (*func.name).to_string(),
                display_name: func
                    .display_name
                    .as_ref()
                    .map(Into::into)
                    .or_else(|| Some(func.name.to_string())),
                is_builtin: func.builtin,
            }),
            Err(_) => None,
        })
        .collect();

    // track(
    //     &posthog_client,
    //     &ctx,
    //     &original_uri,
    //     "get_variant",
    //     serde_json::json!({
    //                 "variant_name": variant.name(),
    //                 "variant_category": variant.category(),
    //                 "variant_menu_name": variant.menu_name(),
    //                 "variant_id": variant.id(),
    //                 "variant_component_type": variant.component_type(),
    //     }),
    // );

    Ok(Json(response))
}
