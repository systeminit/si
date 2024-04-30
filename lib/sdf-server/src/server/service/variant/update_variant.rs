use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use hyper::Uri;
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use dal::schema::variant::authoring::VariantAuthoringClient;
use dal::{ChangeSet, DalContext, Func, SchemaVariant, SchemaVariantId, WsEvent};
use dal::{ComponentType, SchemaId, Visibility};
use telemetry::prelude::*;

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::service::async_route::handle_error;
use crate::service::variant::SchemaVariantResult;

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExecVariantRequest {
    pub id: SchemaId,
    pub default_schema_variant_id: SchemaVariantId,
    pub name: String,
    pub menu_name: Option<String>,
    pub category: String,
    pub color: String,
    pub link: Option<String>,
    pub code: String,
    pub description: Option<String>,
    pub component_type: ComponentType,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecVariantResponse {
    pub task_id: Ulid,
    pub success: bool,
}

pub async fn update_variant(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<ExecVariantRequest>,
) -> SchemaVariantResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let task_id = Ulid::new();

    let request_span = Span::current();

    tokio::task::spawn(async move {
        let new_schema_variant_id = match update_variant_inner(
            &ctx,
            &request,
            &original_uri,
            PosthogClient(posthog_client),
            request_span,
        )
        .await
        {
            Ok(values) => values,
            Err(err) => {
                return handle_error(&ctx, original_uri, task_id, err).await;
            }
        };

        let event = match WsEvent::schema_variant_update_finished(&ctx, new_schema_variant_id).await
        {
            Ok(event) => event,
            Err(err) => {
                return handle_error(&ctx, original_uri, task_id, err).await;
            }
        };

        if let Err(err) = event.publish_on_commit(&ctx).await {
            return handle_error(&ctx, original_uri, task_id, err).await;
        };

        if let Err(err) = ctx.commit().await {
            handle_error(&ctx, original_uri, task_id, err).await;
        }
    });

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }

    Ok(response.body(serde_json::to_string(&ExecVariantResponse {
        task_id,
        success: true,
    })?)?)
}

#[instrument(name = "async_task.update_variant", level = "info", skip_all)]
pub async fn update_variant_inner(
    ctx: &DalContext,
    request: &ExecVariantRequest,
    _original_uri: &Uri,
    PosthogClient(_posthog_client): PosthogClient,
    _request_span: Span,
) -> SchemaVariantResult<SchemaVariantId> {
    let sv = SchemaVariant::get_by_id(ctx, request.default_schema_variant_id).await?;
    let components_in_use = sv.get_components_on_graph(ctx).await?;

    if let Some(asset_func_id) = sv.asset_func_id() {
        let asset_func = Func::get_by_id_or_error(ctx, asset_func_id).await?;
        if !components_in_use.is_empty() {
            // If we have components_in_use
            // We should create a new version of the schema_variant
            // and we will set that new version to be the default for the schema
            let new_variant_id =
                VariantAuthoringClient::update_and_generate_variant_with_new_version(
                    ctx,
                    &asset_func,
                    sv.id(),
                    request.name.clone(),
                    request.menu_name.clone(),
                    request.category.clone(),
                    request.color.clone(),
                    request.link.clone(),
                    request.code.clone(),
                    request.description.clone(),
                    request.component_type,
                )
                .await?;
            return Ok(new_variant_id);
        } else {
            // We are going to work on the current version of the schema variant
            // 1. update the current Asset Func
            // 2. Execute the func
            // 3. Build a new package spec
            // 4. Run the work to import the spec but ignore the func and the schema_Variant_id
            dbg!("I would edit the existing schema variant");
            // let updated_sv = VariantAuthoringClient::edit_existing_variant(&ctx).await?;
            todo!()
        }
    } else {
        return Err(
            crate::service::variant::SchemaVariantError::SchemaVariantAssetNotFound(sv.id()),
        );
    }
}
