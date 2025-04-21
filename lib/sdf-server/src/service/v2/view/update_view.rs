use crate::service::force_change_set_response::ForceChangeSetResponse;
use crate::service::v2::view::{ViewError, ViewResult};
use crate::tracking::track;
use crate::{
    extract::{HandlerContext, PosthogClient},
    service::v2::AccessBuilder,
};
use axum::Json;
use axum::extract::{Host, OriginalUri, Path};
use dal::diagram::view::{View, ViewId, ViewView};
use dal::{ChangeSet, ChangeSetId, WorkspacePk, WsEvent};
use serde::{Deserialize, Serialize};
use si_events::audit_log::AuditLogKind;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub name: String,
}

pub async fn update_view(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, view_id)): Path<(WorkspacePk, ChangeSetId, ViewId)>,
    Json(Request { name }): Json<Request>,
) -> ViewResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    // NOTE(victor) We want to still move the user to a new changeset if they ran an update event,
    // just don't change any data if they tried to rename the changeset to the name it already has
    let should_update = if let Some(view) = View::find_by_name(&ctx, name.as_str()).await? {
        if view.id() == view_id {
            false
        } else {
            return Err(ViewError::NameAlreadyInUse(name));
        }
    } else {
        true
    };

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    if should_update {
        let mut view = View::get_by_id(&ctx, view_id).await?;
        let old_view_name = view.name().to_owned();
        view.set_name(&ctx, name).await?;

        track(
            &posthog_client,
            &ctx,
            &original_uri,
            &host_name,
            "update_view",
            serde_json::json!({
                "how": "/diagram/update_view",
                "view_id": view.id(),
                "view_new_name": view.name(),
                "view_old_name": old_view_name,
                "change_set_id": ctx.change_set_id(),
            }),
        );
        ctx.write_audit_log(
            AuditLogKind::UpdateView {
                view_id,
                old_name: old_view_name.clone(),
            },
            view.name().to_owned(),
        )
        .await?;
        let view_view = ViewView::from_view(&ctx, view).await?;

        WsEvent::view_updated(&ctx, view_view.clone())
            .await?
            .publish_on_commit(&ctx)
            .await?;
    }

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, ()))
}
