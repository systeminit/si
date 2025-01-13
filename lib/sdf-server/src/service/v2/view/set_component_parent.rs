use super::ViewResult;
use crate::{
    extract::HandlerContext, service::force_change_set_response::ForceChangeSetResponse,
    service::v2::AccessBuilder,
};
use axum::extract::Path;
use axum::Json;
use dal::diagram::view::ViewId;
use dal::{
    component::frame::Frame, ChangeSet, ChangeSetId, Component, ComponentId, WorkspacePk, WsEvent,
};
use serde::{Deserialize, Serialize};
use si_events::audit_log::AuditLogKind;
use std::collections::HashMap;
use ulid::Ulid;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetComponentParentRequest {
    pub parent_id_by_component_id: HashMap<ComponentId, Option<ComponentId>>,
    pub client_ulid: Ulid,
    pub request_ulid: Ulid,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetComponentParentResponse {
    pub request_ulid: Ulid,
}

// TODO move this to outside of the view controller
pub async fn set_component_parent(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id, _view_id)): Path<(WorkspacePk, ChangeSetId, ViewId)>,
    Json(request): Json<SetComponentParentRequest>,
) -> ViewResult<ForceChangeSetResponse<SetComponentParentResponse>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let mut socket_map = HashMap::new();
    for (id, maybe_new_parent) in request.parent_id_by_component_id {
        let component = Component::get_by_id(&ctx, id).await?;

        if let Some(new_parent) = maybe_new_parent {
            let old_parent_id = component.parent(&ctx).await?;
            let old_parent_name = if let Some(old_parent_id) = old_parent_id {
                let parent = Component::get_by_id(&ctx, old_parent_id).await?;
                Some(parent.name(&ctx).await?)
            } else {
                None
            };
            Frame::upsert_parent(&ctx, component.id(), new_parent).await?;
            let new_parent_name = Component::get_by_id(&ctx, new_parent)
                .await?
                .name(&ctx)
                .await?;
            ctx.write_audit_log(
                AuditLogKind::UpdateComponentParent {
                    component_id: component.id(),
                    old_parent_id,
                    old_parent_name,
                    new_parent_id: new_parent,
                    new_parent_name,
                },
                component.name(&ctx).await?,
            )
            .await?;
        } else {
            let old_parent_id = component.parent(&ctx).await?;
            if let Some(old_parent_id) = old_parent_id {
                let parent = Component::get_by_id(&ctx, old_parent_id).await?;
                ctx.write_audit_log(
                    AuditLogKind::OrphanComponent {
                        component_id: component.id(),
                        previous_parent_id: old_parent_id,
                        previous_parent_name: parent.name(&ctx).await?,
                    },
                    component.name(&ctx).await?.to_owned(),
                )
                .await?;
            }
            Frame::orphan_child(&ctx, component.id()).await?;
        }

        let component = Component::get_by_id(&ctx, id).await?;
        let payload = component
            .into_frontend_type(
                &ctx,
                None,
                component.change_status(&ctx).await?,
                &mut socket_map,
            )
            .await?;
        WsEvent::component_updated(&ctx, payload)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    }

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(
        force_change_set_id,
        SetComponentParentResponse {
            request_ulid: request.request_ulid,
        },
    ))
}
