use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
    },
};
use dal::{
    AttributeValue,
    AttributeValueId,
    ChangeSet,
    ComponentId,
    InputSocketId,
    OutputSocketId,
    Visibility,
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
use sdf_v1_routes_diagram::create_connection::create_connection_inner;
use serde::{
    Deserialize,
    Serialize,
};

use super::{
    ComponentError,
    ComponentResult,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct OverrideWithConnectionRequest {
    pub from_component_id: ComponentId,
    pub from_socket_id: OutputSocketId,
    pub to_component_id: ComponentId,
    pub to_socket_id: InputSocketId,
    pub attribute_value_id_to_override: AttributeValueId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn override_with_connection(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    PosthogClient(posthog_client): PosthogClient,
    Json(OverrideWithConnectionRequest {
        from_component_id,
        from_socket_id,
        to_component_id,
        to_socket_id,
        attribute_value_id_to_override,
        visibility,
    }): Json<OverrideWithConnectionRequest>,
) -> ComponentResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder.build(request_ctx.build(visibility)).await?;
    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    // first reset the attribute value prototype being overridden
    if AttributeValue::component_prototype_id(&ctx, attribute_value_id_to_override)
        .await?
        .is_some()
    {
        AttributeValue::use_default_prototype(&ctx, attribute_value_id_to_override).await?;
    }
    // then make the connection
    create_connection_inner(
        &ctx,
        from_component_id,
        from_socket_id,
        to_component_id,
        to_socket_id,
    )
    .await
    .map_err(ComponentError::Diagram)?;
    // just look at all this paperwork!
    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "override_with_connection",
        serde_json::json!({
            "from_component_id": from_component_id,
            "from_socket_id": from_socket_id,
            "to_component_id": to_component_id,
            "to_socket_id": to_socket_id,
            "attribute_value_id_to_override": attribute_value_id_to_override,
            "change_set_id": ctx.change_set_id(),
        }),
    );

    ctx.commit().await?;

    // let the front end know if nothing was created so we can tell the user
    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
