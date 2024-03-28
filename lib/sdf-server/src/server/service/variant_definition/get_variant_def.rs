use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use dal::{ComponentType, Func, SchemaVariant, SchemaVariantId, Timestamp, Visibility};
use serde::{Deserialize, Serialize};

// use super::{is_variant_def_locked, SchemaVariantDefinitionError, SchemaVariantDefinitionResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
// use crate::server::tracking::track;
use crate::service::func::compile_return_types;
use crate::service::func::list_funcs::ListedFuncView;
use crate::service::variant_definition::{SchemaVariantError, SchemaVariantResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetVariantDefRequest {
    pub id: SchemaVariantId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVariantDefResponse {
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

pub async fn get_variant_def(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Query(request): Query<GetVariantDefRequest>,
) -> SchemaVariantResult<Json<GetVariantDefResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let variant_def = SchemaVariant::get_by_id(&ctx, request.id).await?;
    let schema = variant_def.schema(&ctx).await?;

    let mut response: GetVariantDefResponse = GetVariantDefResponse {
        id: request.id,
        name: schema.name().into(),
        menu_name: variant_def.display_name(),
        category: variant_def.category().into(),
        color: variant_def.get_color(&ctx).await?,
        link: variant_def.link(),
        description: variant_def.description(),
        component_type: variant_def.component_type(),
        timestamp: variant_def.timestamp(),
        // Will be set elsewhere
        code: "".to_string(),
        funcs: vec![],
        types: "".to_string(),
        has_components: false,
    };

    if let Some(authoring_func) = variant_def.asset_func_id() {
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
    //     "get_variant_def",
    //     serde_json::json!({
    //                 "variant_def_name": variant_def.name(),
    //                 "variant_def_category": variant_def.category(),
    //                 "variant_def_menu_name": variant_def.menu_name(),
    //                 "variant_def_id": variant_def.id(),
    //                 "variant_def_component_type": variant_def.component_type(),
    //     }),
    // );

    Ok(Json(response))
}
