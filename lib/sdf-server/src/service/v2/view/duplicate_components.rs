use axum::{
    Json,
    extract::Path,
};
use dal::{
    ChangeSet,
    Component,
    ComponentId,
};
use serde::{
    Deserialize,
    Serialize,
};

use super::{
    ViewParam,
    ViewResult,
};
use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::force_change_set_response::ForceChangeSetResponse,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct PasteComponentsRequest {
    pub components: Vec<ComponentId>,
}

/// Duplicate a set of [`Component`](Component)s via their componentIds. Creates change-set if on head
pub async fn duplicate_components(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ViewParam { view_id }): Path<ViewParam>,
    Json(request): Json<PasteComponentsRequest>,
) -> ViewResult<ForceChangeSetResponse<()>> {
    let force_change_set_id = ChangeSet::force_new(ctx).await?;

    let pasted_component_ids = Component::duplicate(ctx, view_id, request.components).await?;

    // Emit  posthog events
    for pasted_component_id in pasted_component_ids {
        let schema = Component::schema_for_component_id(ctx, pasted_component_id).await?;
        tracker.track(
            ctx,
            "paste_component",
            serde_json::json!({
                "how": "/v2/view/paste_component",
                "component_id": pasted_component_id,
                "component_schema_name": schema.name(),
            }),
        );
    }

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
