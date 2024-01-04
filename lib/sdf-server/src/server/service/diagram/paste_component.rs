use axum::{extract::OriginalUri, http::uri::Uri};
use axum::{response::IntoResponse, Json};
use chrono::Utc;
use dal::NodeId;
use dal::{
    action_prototype::ActionPrototypeContextField, func::backend::js_action::ActionRunResult,
    Action, ActionKind, ActionPrototype, ActionPrototypeContext, ChangeSet, Component,
    ComponentError, ComponentId, Connection, DalContextBuilder, Edge, Node, StandardModel,
    Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use telemetry::prelude::*;
use tokio::task::JoinSet;
use veritech_client::ResourceStatus;

use super::{DiagramError, DiagramResult};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[allow(clippy::too_many_arguments)]
async fn paste_single_component(
    ctx_builder: DalContextBuilder,
    request_ctx: dal::context::AccessBuilder,
    visibility: Visibility,
    component_id: ComponentId,
    offset_x: f64,
    offset_y: f64,
    original_uri: &Uri,
    PosthogClient(posthog_client): &PosthogClient,
) -> DiagramResult<(Component, Node)> {
    let ctx = ctx_builder.build(request_ctx.build(visibility)).await?;

    let original_comp = Component::get_by_id(&ctx, &component_id)
        .await?
        .ok_or(DiagramError::ComponentNotFound)?;
    let original_node = original_comp
        .node(&ctx)
        .await?
        .pop()
        .ok_or(ComponentError::NodeNotFoundForComponent(component_id))?;

    let schema_variant = original_comp
        .schema_variant(&ctx)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;

    let (pasted_comp, mut pasted_node) =
        Component::new(&ctx, original_comp.name(&ctx).await?, *schema_variant.id()).await?;
    let x: f64 = original_node.x().parse()?;
    let y: f64 = original_node.y().parse()?;
    pasted_node
        .set_geometry(
            &ctx,
            (x + offset_x).to_string(),
            (y + offset_y).to_string(),
            original_node.width(),
            original_node.height(),
        )
        .await?;
    ctx.commit().await?;

    pasted_comp
        .clone_attributes_from(&ctx, *original_comp.id())
        .await?;

    pasted_comp
        .set_resource_raw(
            &ctx,
            ActionRunResult {
                status: ResourceStatus::Ok,
                payload: None,
                message: None,
                logs: Vec::new(),
                last_synced: Some(Utc::now().to_rfc3339()),
            },
            false,
        )
        .await?;

    pasted_comp
        .set_name(
            &ctx,
            Some(format!("{} - Copy", original_comp.name(&ctx).await?)),
        )
        .await?;

    ctx.commit().await?;

    for prototype in ActionPrototype::find_for_context_and_kind(
        &ctx,
        ActionKind::Create,
        ActionPrototypeContext::new_for_context_field(ActionPrototypeContextField::SchemaVariant(
            *schema_variant.id(),
        )),
    )
    .await?
    {
        let action = Action::new(&ctx, *prototype.id(), *pasted_comp.id()).await?;
        let prototype = action.prototype(&ctx).await?;

        track(
            posthog_client,
            &ctx,
            original_uri,
            "create_action",
            serde_json::json!({
                "how": "/diagram/paste_components",
                "prototype_id": prototype.id(),
                "prototype_kind": prototype.kind(),
                "component_id": pasted_comp.id(),
                "component_name": pasted_comp.name(&ctx).await?,
                "change_set_pk": ctx.visibility().change_set_pk,
            }),
        );
    }

    let schema = pasted_comp
        .schema(&ctx)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;
    track(
        posthog_client,
        &ctx,
        original_uri,
        "paste_component",
        serde_json::json!({
            "component_id": pasted_comp.id(),
            "component_schema_name": schema.name(),
        }),
    );

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok((pasted_comp, pasted_node))
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PasteComponentsRequest {
    pub component_ids: Vec<ComponentId>,
    pub offset_x: f64,
    pub offset_y: f64,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PasteComponentsResponse {
    pub copied_component_id: ComponentId,
    pub pasted_component_id: ComponentId,
    pub pasted_node_id: NodeId,
}

/// Paste a set of [`Component`](dal::Component)s via their componentId. Creates change-set if on head
pub async fn paste_components(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<PasteComponentsRequest>,
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
    ctx.commit().await?;

    let mut tasks = JoinSet::new();

    let mut pasted_components_by_original = HashMap::new();
    for component_id in &request.component_ids {
        let ctx_builder = ctx.to_builder();
        let (visibility, component_id) = (*ctx.visibility(), *component_id);
        let (offset_x, offset_y) = (request.offset_x, request.offset_y);
        let (original_uri, posthog_client) =
            (original_uri.clone(), PosthogClient(posthog_client.clone()));
        tasks.spawn(async move {
            let (pasted_comp, pasted_node) = paste_single_component(
                ctx_builder,
                request_ctx,
                visibility,
                component_id,
                offset_x,
                offset_y,
                &original_uri,
                &posthog_client,
            )
            .await?;

            Ok::<_, DiagramError>((component_id, pasted_comp, pasted_node))
        });
    }

    while let Some(result) = tasks.join_next().await {
        match result {
            Ok(Ok((component_id, pasted_comp, pasted_node))) => {
                pasted_components_by_original.insert(component_id, (pasted_comp, pasted_node));
            }
            Ok(Err(err)) => return Err(err)?,
            // Task panicked, let's propagate it
            Err(err) => match err.try_into_panic() {
                Ok(panic) => {
                    std::panic::resume_unwind(panic);
                }
                Err(err) => {
                    if err.is_cancelled() {
                        warn!("Paste Component was cancelled: {err}");
                    } else {
                        error!("Unknown failure in component paste: {err}");
                    }
                }
            },
        }
    }

    let mut response_body = Vec::new();

    for component_id in &request.component_ids {
        let (pasted_comp, pasted_node) = match pasted_components_by_original.get(component_id) {
            Some((c, v)) => (c, v),
            None => return Err(DiagramError::PasteError),
        };

        response_body.push(PasteComponentsResponse {
            copied_component_id: *component_id,
            pasted_component_id: *pasted_comp.id(),
            pasted_node_id: *pasted_node.id(),
        });

        let edges = Edge::list_for_component(&ctx, *component_id).await?;
        for edge in edges {
            let tail = pasted_components_by_original.get(&edge.tail_component_id());
            let head = pasted_components_by_original.get(&edge.head_component_id());
            match (tail, head) {
                (Some(tail), Some(head)) => {
                    let (_, tail_node) = tail;
                    let (_, head_node) = head;
                    Connection::new(
                        &ctx,
                        *tail_node.id(),
                        edge.tail_socket_id(),
                        *head_node.id(),
                        edge.head_socket_id(),
                        *edge.kind(),
                    )
                    .await?;
                }
                (None, Some(head)) => {
                    let (_, head_node) = head;
                    Connection::new(
                        &ctx,
                        edge.tail_node_id(),
                        edge.tail_socket_id(),
                        *head_node.id(),
                        edge.head_socket_id(),
                        *edge.kind(),
                    )
                    .await?;
                }
                _ => {}
            }
        }
    }

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }

    Ok(response.body(serde_json::to_string(&response_body)?)?)
}
