use axum::Json;
use axum::{extract::OriginalUri, http::uri::Uri, response::IntoResponse};
use dal::{ChangeSet, Component, ComponentId, DalContext, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::diagram::DiagramError;
use dal::standard_model::StandardModel;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestoreComponentRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

async fn restore_single_component(
    ctx: &DalContext,
    component_id: ComponentId,
    original_uri: &Uri,
    PosthogClient(posthog_client): &PosthogClient,
) -> DiagramResult<()> {
    Component::restore_and_propagate(ctx, component_id).await?;

    let (component, schema) = {
        let ctx_with_deleted = &ctx.clone_with_delete_visibility();

        let component = Component::get_by_id(ctx_with_deleted, &component_id)
            .await?
            .ok_or(DiagramError::ComponentNotFound)?;

        let schema = component
            .schema(ctx)
            .await?
            .ok_or(DiagramError::SchemaNotFound)?;

        (component, schema)
    };

    track(
        posthog_client,
        ctx,
        original_uri,
        "restore_component",
        serde_json::json!({
                    "component_id": component.id(),
                    "component_schema_name": schema.name(),
        }),
    );

    WsEvent::change_set_written(ctx)
        .await?
        .publish_on_commit(ctx)
        .await?;
    Ok(())
}

/// Restore a [`Component`](dal::Component) via its componentId. Creating change set if on head.
pub async fn restore_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    posthog_client: PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<RestoreComponentRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut force_changeset_pk = None;
    if ctx.visibility().is_head() {
        let change_set = ChangeSet::new(&ctx, ChangeSet::generate_name(), None).await?;

        let new_visibility = Visibility::new(change_set.pk, request.visibility.deleted_at);

        ctx.update_visibility(new_visibility);

        force_changeset_pk = Some(change_set.pk);

        WsEvent::change_set_created(&ctx, change_set.pk)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    };

    restore_single_component(&ctx, request.component_id, &original_uri, &posthog_client).await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestoreComponentsRequest {
    pub component_ids: Vec<ComponentId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

/// Restore a set of [`Component`](dal::Component)s via their componentId. Creating change set if on head.
pub async fn restore_components(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    posthog_client: PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<RestoreComponentsRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut force_changeset_pk = None;
    if ctx.visibility().is_head() {
        let change_set = ChangeSet::new(&ctx, ChangeSet::generate_name(), None).await?;

        let new_visibility = Visibility::new(change_set.pk, request.visibility.deleted_at);

        ctx.update_visibility(new_visibility);

        force_changeset_pk = Some(change_set.pk);

        WsEvent::change_set_created(&ctx, change_set.pk)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    };

    for component_id in request.component_ids {
        restore_single_component(&ctx, component_id, &original_uri, &posthog_client).await?;
        ctx.commit().await?;
    }

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
