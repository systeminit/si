use axum::{extract::OriginalUri, http::uri::Uri};
use axum::{response::IntoResponse, Json};
use dal::{
    action_prototype::ActionPrototypeContextField, Action, ActionKind, ActionPrototype,
    ActionPrototypeContext, ChangeSet, Component, ComponentId, DalContext, StandardModel,
    Visibility, WsEvent,
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
    let comp_schema_variant = comp
        .schema_variant(ctx)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;

    let resource = comp.resource(ctx).await?;

    // TODO: this is tricky, we don't want to delete all actions, but we don't need to be perfect
    // right now, let's see how the usage plays with users
    let actions = Action::find_for_change_set(ctx).await?;
    for mut action in actions {
        if action.component_id() == comp.id() {
            action.delete_by_id(ctx).await?;
        }
    }

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

    if resource.payload.is_some() {
        if let Some(prototype) = ActionPrototype::find_for_context_and_kind(
            ctx,
            ActionKind::Delete,
            ActionPrototypeContext::new_for_context_field(
                ActionPrototypeContextField::SchemaVariant(*comp_schema_variant.id()),
            ),
        )
        .await?
        .first()
        {
            Action::new(ctx, *prototype.id(), *comp.id()).await?;

            let change_set = ChangeSet::get_by_pk(ctx, &ctx.visibility().change_set_pk)
                .await?
                .ok_or(DiagramError::ChangeSetNotFound)?;
            change_set.sort_actions(ctx).await?;
        }
    }

    WsEvent::change_set_written(ctx)
        .await?
        .publish_on_commit(ctx)
        .await?;

    Ok(())
}

/// Delete a [`Component`](dal::Component) via its componentId. Creates change-set if on head
pub async fn delete_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    posthog_client: PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<DeleteComponentRequest>,
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

    delete_single_component(&ctx, request.component_id, &original_uri, &posthog_client).await?;

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
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
        delete_single_component(&ctx, component_id, &original_uri, &posthog_client).await?;
    }

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
