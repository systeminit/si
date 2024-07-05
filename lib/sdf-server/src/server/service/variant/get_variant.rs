use axum::extract::OriginalUri;
use axum::{extract::Query, Json};
use dal::func::authoring::FuncAuthoringClient;
use dal::func::summary::FuncSummary;
use dal::{
    ComponentType, Func, Schema, SchemaId, SchemaVariant, SchemaVariantId, Timestamp, Visibility,
};
use serde::{Deserialize, Serialize};

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::variant::{SchemaVariantError, SchemaVariantResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetVariantRequest {
    pub id: SchemaId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetVariantResponse {
    pub id: SchemaId,
    pub default_schema_variant_id: SchemaVariantId,
    pub schema_name: String,
    pub name: String,
    pub display_name: String,
    pub category: String,
    pub color: String,
    pub link: Option<String>,
    pub description: Option<String>,
    pub code: String,
    pub component_type: ComponentType,
    pub funcs: Vec<FuncSummary>,
    pub types: String,
    pub has_components: bool,
    pub is_locked: bool,
    #[serde(flatten)]
    pub timestamp: Timestamp,
}

pub async fn get_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Query(request): Query<GetVariantRequest>,
) -> SchemaVariantResult<Json<GetVariantResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let schema = Schema::get_by_id(&ctx, request.id).await?;
    let default_schema_variant_id = schema.get_default_schema_variant_id(&ctx).await?.ok_or(
        SchemaVariantError::NoDefaultSchemaVariantFoundForSchema(schema.id()),
    )?;
    let variant = SchemaVariant::get_by_id_or_error(&ctx, default_schema_variant_id).await?;

    // Collect the funcs for all schema variants corresponding to the schema. We want this because
    // while asset editing generally corresponds to the current default schema variant id, you are
    // unable to access functions that belong to components using older schema variants on the
    // diagram unless you collect all functions for all schema variants. Function access is
    // important because it provides not only the ability to edit a function, but also its bindings.
    let funcs = FuncSummary::list_for_schema_id(&ctx, schema.id()).await?;

    let mut response: GetVariantResponse = GetVariantResponse {
        id: request.id,
        default_schema_variant_id,
        schema_name: schema.name().into(),
        name: variant.version().into(),
        display_name: variant.display_name().to_string(),
        category: variant.category().into(),
        color: variant.get_color(&ctx).await?,
        link: variant.link(),
        description: variant.description(),
        component_type: variant.component_type(),
        timestamp: variant.timestamp(),
        // Will be set elsewhere
        code: "".to_string(),
        funcs,
        types: "".to_string(),
        has_components: false,
        is_locked: variant.is_locked(),
    };

    if let Some(authoring_func) = variant.asset_func_id() {
        let asset_func = Func::get_by_id_or_error(&ctx, authoring_func).await?;

        response.code = asset_func
            .code_plaintext()?
            .ok_or(SchemaVariantError::FuncIsEmpty(asset_func.id))?;

        response.types = FuncAuthoringClient::compile_return_types(
            asset_func.backend_response_type,
            asset_func.backend_kind,
        )
        .to_string();
    }

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "get_variant",
        serde_json::json!({
                    "variant_version": variant.version(),
                    "variant_category": variant.category(),
                    "variant_display_name": variant.display_name(),
                    "variant_id": variant.id(),
                    "schema_id": schema.id(),
                    "variant_component_type": variant.component_type(),
        }),
    );

    Ok(Json(response))
}
