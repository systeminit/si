use std::collections::HashMap;

use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
    },
};
use dal::{
    ChangeSet,
    Component,
    ComponentId,
    WsEvent,
};
use sdf_core::{
    force_change_set_response::ForceChangeSetResponse,
    tracking::track,
};
use sdf_extract::{
    HandlerContext,
    PosthogClient,
    v1::AccessBuilder,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_db::Visibility;

use super::ComponentResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetNameRequest {
    pub component_id: ComponentId,
    pub name: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn set_name(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(SetNameRequest {
        component_id,
        name,
        visibility,
    }): Json<SetNameRequest>,
) -> ComponentResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder.build(request_ctx.build(visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let component = Component::get_by_id(&ctx, component_id).await?;
    component.set_name(&ctx, &name).await?;

    let component = Component::get_by_id(&ctx, component_id).await?;
    let mut socket_map = HashMap::new();
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

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "set_component_name",
        serde_json::json!({
            "how": "/component/set_name",
            "component_id": component.id(),
        }),
    );

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
