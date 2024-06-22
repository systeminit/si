use axum::{
    extract::{OriginalUri, Path},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
    Json, Router,
};
use dal::{
    ChangeSetId, ComponentType, DalContext, FuncId, InputSocket, OutputSocket, Schema, SchemaId,
    SchemaVariant, SchemaVariantId, Timestamp, WorkspacePk,
};
use serde::Serialize;
use thiserror::Error;

use crate::server::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    state::AppState,
    tracking::track,
};

use super::ApiError;

pub fn routes() -> Router<AppState> {
    const PREFIX: &str = "/workspaces/:workspace_id/change-sets/:change_set_id";

    Router::new()
        .route(
            &format!("{PREFIX}/schema-variants"),
            get(list_schema_variants),
        )
        .route(
            &format!("{PREFIX}/schema-variants/:schema_variant_id"),
            get(get_variant),
        )
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchemaVariantAPI {
    schema_id: SchemaId,
    schema_name: String,
    schema_variant_id: SchemaVariantId,
    display_name: Option<String>,
    category: String,
    description: Option<String>,
    link: Option<String>,
    color: String,
    asset_func_id: FuncId,
    func_ids: Vec<FuncId>,
    component_type: ComponentType,
    input_sockets: Vec<InputSocket>,
    output_sockets: Vec<OutputSocket>,
    #[serde(flatten)]
    timestamp: Timestamp,
}

pub async fn list_schema_variants(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> Result<Json<Vec<SchemaVariantAPI>>, ListSchemaVariantsError> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let mut schema_variants = Vec::new();

    for schema_id in Schema::list_ids(&ctx).await? {
        // NOTE(fnichol): Yes there is `SchemaVariant::list_default_ids()`, but shortly we'll be
        // asking for more than only the defaults which reduces us back to looping through schemas
        // to filter appropriate schema variants.
        let schema_variant = SchemaVariant::get_default_for_schema(&ctx, schema_id).await?;
        if !schema_variant.ui_hidden() {
            schema_variants.push(schema_variant_api(&ctx, schema_id, schema_variant).await?);
        }
    }

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "list_schema_variants",
        serde_json::json!({}),
    );

    Ok(Json(schema_variants))
}

pub async fn get_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Path((_workspace_pk, change_set_id, schema_variant_id)): Path<(
        WorkspacePk,
        ChangeSetId,
        SchemaVariantId,
    )>,
) -> Result<Json<SchemaVariantAPI>, ListSchemaVariantsError> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let schema_variant = SchemaVariant::get_by_id(&ctx, schema_variant_id).await?;
    let schema_id = SchemaVariant::schema_id_for_schema_variant_id(&ctx, schema_variant_id).await?;
    let schema_variant = schema_variant_api(&ctx, schema_id, schema_variant).await?;

    // Ported from `lib/sdf-server/src/server/service/variant/get_variant.rs`, so changes may be
    // desired here...
    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "get_variant",
        serde_json::json!({
                    "schema_name": &schema_variant.schema_name,
                    "variant_category": &schema_variant.category,
                    "variant_menu_name": schema_variant.display_name.as_ref(),
                    "variant_id": schema_variant.schema_variant_id,
                    "schema_id": schema_variant.schema_id,
                    "variant_component_type": schema_variant.component_type,
        }),
    );

    Ok(Json(schema_variant))
}

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ListSchemaVariantsError {
    #[error("asset func missing: {0}")]
    AssetFuncMissing(#[from] SchemaVariantMissingAssetFuncId),
    #[error("schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("schema error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
}

impl IntoResponse for ListSchemaVariantsError {
    fn into_response(self) -> Response {
        let status_code = match &self {
            Self::Transactions(dal::TransactionsError::BadWorkspaceAndChangeSet) => {
                StatusCode::FORBIDDEN
            }
            // When a graph node cannot be found for a schema variant, it is not found
            Self::SchemaVariant(dal::SchemaVariantError::NotFound(_)) => StatusCode::NOT_FOUND,
            _ => ApiError::DEFAULT_ERROR_STATUS_CODE,
        };

        ApiError::new(status_code, self).into_response()
    }
}

#[derive(Debug, Error)]
#[error("schema variant missing asset func id; schema_variant_id={0}")]
pub struct SchemaVariantMissingAssetFuncId(SchemaVariantId);

impl SchemaVariantAPI {
    fn new(
        schema_id: SchemaId,
        schema_name: String, // TODO(fnichol): remove when name comes from schema variant
        value: SchemaVariant,
        func_ids: Vec<FuncId>,
        input_sockets: Vec<InputSocket>,
        output_sockets: Vec<OutputSocket>,
    ) -> Result<Self, SchemaVariantMissingAssetFuncId> {
        Ok(Self {
            schema_id,
            schema_name,
            schema_variant_id: value.id,
            display_name: value.display_name,
            category: value.category,
            description: value.description,
            link: value.link,
            color: value.color,
            asset_func_id: value
                .asset_func_id
                .ok_or(SchemaVariantMissingAssetFuncId(value.id))?,
            func_ids,
            component_type: value.component_type,
            input_sockets,
            output_sockets,
            timestamp: value.timestamp,
        })
    }
}

async fn schema_variant_api(
    ctx: &DalContext,
    schema_id: SchemaId,
    schema_variant: SchemaVariant,
) -> Result<SchemaVariantAPI, ListSchemaVariantsError> {
    let (output_sockets, input_sockets) =
        SchemaVariant::list_all_sockets(ctx, schema_variant.id()).await?;
    let func_ids: Vec<_> = SchemaVariant::all_func_ids(ctx, schema_variant.id())
        .await?
        .into_iter()
        .collect();

    // TODO(fnichol): remove when name comes from schema variant
    let schema = Schema::get_by_id(ctx, schema_id).await?;

    SchemaVariantAPI::new(
        schema_id,
        schema.name().to_owned(),
        schema_variant,
        func_ids,
        input_sockets,
        output_sockets,
    )
    .map_err(Into::into)
}
