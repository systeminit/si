use std::collections::HashMap;

use axum::{
    extract::{Host, OriginalUri},
    http::uri::Uri,
    Json,
};
use dal::{
    change_status::ChangeStatus, diagram::SummaryDiagramEdge, ChangeSet, Component, ComponentId,
    DalContext, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};
use si_events::audit_log::AuditLogKind;

use super::DiagramResult;
use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::force_change_set_response::ForceChangeSetResponse,
    track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteComponentsRequest {
    pub component_ids: Vec<ComponentId>,
    pub force_erase: bool,
    #[serde(flatten)]
    pub visibility: Visibility,
}

/// Delete a set of [`Component`](dal::Component)s via their componentId. Creates change-set if on head
pub async fn delete_components(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    posthog_client: PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<DeleteComponentsRequest>,
) -> DiagramResult<ForceChangeSetResponse<HashMap<ComponentId, bool>>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;
    let components_existing_on_head =
        Component::exists_on_head(&ctx, request.component_ids.clone()).await?;
    let base_change_set_ctx = ctx.clone_with_base().await?;

    let mut components = HashMap::new();
    let mut socket_map = HashMap::new();
    for component_id in request.component_ids {
        let component: Component = Component::get_by_id(&ctx, component_id).await?;
        let incoming_connections = component.incoming_connections(&ctx).await?.clone();
        let outgoing_connections = component.outgoing_connections(&ctx).await?.clone();

        let component_still_exists = delete_single_component(
            &ctx,
            component_id,
            request.force_erase,
            &original_uri,
            &host_name,
            &posthog_client,
        )
        .await?;
        ctx.workspace_snapshot()?.cleanup().await?;

        components.insert(component_id, component_still_exists);

        let exists_on_head = components_existing_on_head.contains(&component_id);

        if component_still_exists {
            // to_delete=True
            let component: Component = Component::get_by_id(&ctx, component_id).await?;
            let payload = component
                .into_frontend_type_for_default_view(&ctx, ChangeStatus::Deleted, &mut socket_map)
                .await?;
            WsEvent::component_updated(&ctx, payload)
                .await?
                .publish_on_commit(&ctx)
                .await?;
        } else if exists_on_head {
            let component: Component =
                Component::get_by_id(&base_change_set_ctx, component_id).await?;
            let payload = component
                .into_frontend_type_for_default_view(
                    &base_change_set_ctx,
                    ChangeStatus::Deleted,
                    &mut socket_map,
                )
                .await?;
            WsEvent::component_updated(&ctx, payload)
                .await?
                .publish_on_commit(&ctx)
                .await?;
        } else {
            WsEvent::component_deleted(&ctx, component_id)
                .await?
                .publish_on_commit(&ctx)
                .await?;
        }

        for incoming_connection in incoming_connections {
            let payload = SummaryDiagramEdge {
                from_component_id: incoming_connection.from_component_id,
                from_socket_id: incoming_connection.from_output_socket_id,
                to_component_id: incoming_connection.to_component_id,
                to_socket_id: incoming_connection.to_input_socket_id,
                change_status: ChangeStatus::Deleted,
                created_info: serde_json::to_value(incoming_connection.created_info)?,
                deleted_info: serde_json::to_value(incoming_connection.deleted_info)?,
                to_delete: true,
                from_base_change_set: false,
            };
            WsEvent::connection_upserted(&ctx, payload)
                .await?
                .publish_on_commit(&ctx)
                .await?;
        }

        for outgoing_connection in outgoing_connections {
            let payload = SummaryDiagramEdge {
                from_component_id: outgoing_connection.from_component_id,
                from_socket_id: outgoing_connection.from_output_socket_id,
                to_component_id: outgoing_connection.to_component_id,
                to_socket_id: outgoing_connection.to_input_socket_id,
                change_status: ChangeStatus::Deleted,
                created_info: serde_json::to_value(outgoing_connection.created_info)?,
                deleted_info: serde_json::to_value(outgoing_connection.deleted_info)?,
                to_delete: true,
                from_base_change_set: false,
            };
            WsEvent::connection_upserted(&ctx, payload)
                .await?
                .publish_on_commit(&ctx)
                .await?;
        }
    }

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, components))
}

async fn delete_single_component(
    ctx: &DalContext,
    component_id: ComponentId,
    force_erase: bool,
    original_uri: &Uri,
    host_name: &String,
    PosthogClient(posthog_client): &PosthogClient,
) -> DiagramResult<bool> {
    let component = Component::get_by_id(ctx, component_id).await?;
    let component_name = component.name(ctx).await?;
    let component_schema_variant = component.schema_variant(ctx).await?;

    let id = component.id();
    let component_schema = component.schema(ctx).await?;

    let component_still_exists = if force_erase {
        Component::remove(ctx, id).await?;
        false
    } else {
        component.delete(ctx).await?.is_some()
    };

    ctx.write_audit_log(
        AuditLogKind::DeleteComponent {
            component_id: id.into(),
            name: component_name.to_owned(),
            schema_variant_id: component_schema_variant.id().into(),
            schema_variant_name: component_schema_variant.display_name().to_string(),
        },
        component_name,
    )
    .await?;

    track(
        posthog_client,
        ctx,
        original_uri,
        host_name,
        "delete_component",
        serde_json::json!({
            "how": "/diagram/delete_component",
            "component_id": id,
            "component_schema_name": component_schema.name(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    Ok(component_still_exists)
}
