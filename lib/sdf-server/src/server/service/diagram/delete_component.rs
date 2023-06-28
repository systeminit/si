use axum::Json;
use axum::{extract::OriginalUri, http::uri::Uri};
use dal::{
    ChangeSet, ChangeSetPk, Component, ComponentId, DalContext, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

use super::{DiagramError, DiagramResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteComponentRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ForceChangeSet {
    #[serde(rename = "_forceChangesetPk")] // TODO(victor) find a way to return this as a header
    pub force_changeset_pk: Option<ChangeSetPk>,
}

async fn delete_single_component(
    ctx: &DalContext,
    component_id: ComponentId,
    original_uri: &Uri,
    PosthogClient(posthog_client): &PosthogClient,
) -> DiagramResult<()> {
    let mut comp = Component::get_by_id(ctx, &component_id)
        .await?
        .ok_or(DiagramError::ComponentNotFound)?;

    let comp_schema = comp
        .schema(ctx)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;

    comp.delete_and_propagate(ctx).await?;

    track(
        posthog_client,
        ctx,
        original_uri,
        "delete_component",
        serde_json::json!({
            "component_id": comp.id(),
            "component_schema_name": comp_schema.name(),
        }),
    );

    WsEvent::change_set_written(ctx)
        .await?
        .publish_on_commit(ctx)
        .await?;

    Ok(())
}

/// Delete a [`Component`](dal::Component) via its componentId.
pub async fn delete_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    posthog_client: PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<DeleteComponentRequest>,
) -> DiagramResult<Json<ForceChangeSet>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    delete_single_component(&ctx, request.component_id, &original_uri, &posthog_client).await?;

    ctx.commit().await?;

    Ok(Json(ForceChangeSet {
        force_changeset_pk: None,
    }))
}

/// Delete a [`Component`](dal::Component) via its componentId. Creates change-set if on head
pub async fn delete_component2(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    posthog_client: PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<DeleteComponentRequest>,
) -> DiagramResult<Json<ForceChangeSet>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut force_changeset_pk = None;
    if ctx.visibility().is_head() {
        let change_set = ChangeSet::new(&ctx, ChangeSet::generate_name(&ctx).await?, None).await?;

        let new_visibility = Visibility::new(change_set.pk, request.visibility.deleted_at);

        ctx.update_visibility(new_visibility);

        force_changeset_pk = Some(change_set.pk);

        WsEvent::change_set_created(&ctx, change_set.pk)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    };

    delete_single_component(&ctx, request.component_id, &original_uri, &posthog_client).await?;

    ctx.commit().await?;

    Ok(Json(ForceChangeSet { force_changeset_pk }))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteComponentsRequest {
    pub component_ids: Vec<ComponentId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

/// Delete a set of [`Component`](dal::Component)s via their componentId. Creates change-set if on head
pub async fn delete_components(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    posthog_client: PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<DeleteComponentsRequest>,
) -> DiagramResult<Json<ForceChangeSet>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut force_changeset_pk = None;
    if ctx.visibility().is_head() {
        let change_set = ChangeSet::new(&ctx, ChangeSet::generate_name(&ctx).await?, None).await?;

        let new_visibility = Visibility::new(change_set.pk, request.visibility.deleted_at);

        ctx.update_visibility(new_visibility);

        force_changeset_pk = Some(change_set.pk);

        WsEvent::change_set_created(&ctx, change_set.pk)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    };

    for component_id in request.component_ids {
        delete_single_component(&ctx, component_id, &original_uri, &posthog_client).await?;
        ctx.commit().await?;
    }

    ctx.commit().await?;

    Ok(Json(ForceChangeSet { force_changeset_pk }))
}
