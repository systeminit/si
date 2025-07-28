use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        Path,
    },
};
use dal::{
    ChangeSet,
    ChangeSetId,
    Component,
    ComponentError,
    ComponentId,
    WorkspacePk,
    diagram::{
        geometry::RawGeometry,
        view::{
            View,
            ViewId,
        },
    },
};
use serde::{
    Deserialize,
    Serialize,
};

use super::ViewResult;
use crate::{
    extract::{
        HandlerContext,
        PosthogClient,
    },
    service::{
        force_change_set_response::ForceChangeSetResponse,
        v2::AccessBuilder,
    },
    track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ViewAddComponentsRequest {
    pub component_ids: Vec<ComponentId>,
}

pub async fn add_components(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Path((_workspace_pk, change_set_id, view_id)): Path<(WorkspacePk, ChangeSetId, ViewId)>,
    Json(ViewAddComponentsRequest { component_ids }): Json<ViewAddComponentsRequest>,
) -> ViewResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let view = View::get_by_id(&ctx, view_id).await?;

    // If at least one of the components can be added, don't blow up if errors happen.
    let mut at_least_one_succeeded = false;
    let mut latest_error = None;

    for component_id in component_ids {
        match Component::add_to_view(&ctx, component_id, view_id, RawGeometry::default()).await {
            Ok(_) => {}
            Err(err @ ComponentError::ComponentAlreadyInView(_, _)) => {
                latest_error = Some(err);
                continue;
            }
            Err(err) => return Err(err)?,
        };

        at_least_one_succeeded = true;
    }

    if let Some(err) = latest_error {
        if !at_least_one_succeeded {
            return Err(err)?;
        }
    }

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "component_added_to_view",
        serde_json::json!({
            "how": "/view/add_components",
            "view_id": view_id,
            "view_name": view.name(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, ()))
}
