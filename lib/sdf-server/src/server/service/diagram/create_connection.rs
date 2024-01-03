use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use dal::diagram::edge::DiagramEdgeViewId;
use dal::{Component, ComponentId, ExternalProviderId, InternalProviderId, User, Visibility};
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectionRequest {
    pub from_component_id: ComponentId,
    pub from_external_provider_id: ExternalProviderId,
    pub to_component_id: ComponentId,
    pub to_explicit_internal_provider_id: InternalProviderId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateConnectionResponse {
    pub id: DiagramEdgeViewId,
    pub created_by: Option<User>,
    pub deleted_by: Option<User>,
}

pub async fn create_connection(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Json(request): Json<CreateConnectionRequest>,
) -> DiagramResult<impl IntoResponse> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    // TODO(nick): restore this with the new engine.
    // let mut force_changeset_pk = None;
    // if ctx.visibility().is_head() {
    //     let change_set = ChangeSet::new(&ctx, ChangeSet::generate_name(), None).await?;
    //
    //     let new_visibility = Visibility::new(change_set.pk, request.visibility.deleted_at);
    //
    //     ctx.update_visibility(new_visibility);
    //
    //     force_changeset_pk = Some(change_set.pk);
    //
    //     WsEvent::change_set_created(&ctx, change_set.pk)
    //         .await?
    //         .publish_on_commit(&ctx)
    //         .await?;
    // };

    let attribute_prototype_argument_id = Component::connect(
        &ctx,
        request.from_component_id,
        request.from_external_provider_id,
        request.to_component_id,
        request.to_explicit_internal_provider_id,
    )?;

    // TODO(nick): restore dependent values update.
    // let to_attribute_value_context = AttributeReadContext {
    //     internal_provider_id: Some(*to_socket_internal_provider.id()),
    //     component_id: Some(*to_component.id()),
    //     ..Default::default()
    // };
    // let mut to_attribute_value = AttributeValue::find_for_context(&ctx, to_attribute_value_context)
    //     .await?
    //     .ok_or(DiagramError::AttributeValueNotFoundForContext(
    //         to_attribute_value_context,
    //     ))?;
    //
    // to_attribute_value
    //     .update_from_prototype_function(&ctx)
    //     .await?;
    //
    // ctx.enqueue_job(DependentValuesUpdate::new(
    //     ctx.access_builder(),
    //     *ctx.visibility(),
    //     vec![*to_attribute_value.id()],
    // ))
    // .await?;
    //
    // WsEvent::change_set_written(&ctx)
    //     .await?
    //     .publish_on_commit(&ctx)
    //     .await?;
    //
    // track(
    //     &posthog_client,
    //     &ctx,
    //     &original_uri,
    //     "connection_created",
    //     serde_json::json!({
    //                 "from_node_id": request.from_node_id,
    //                 "from_node_schema_name": &from_component_schema.name(),
    //                 "from_socket_id": request.from_socket_id,
    //                 "from_socket_name": &from_socket.name(),
    //                 "to_node_id": request.to_node_id,
    //                 "to_node_schema_name": &to_component_schema.name(),
    //                 "to_socket_id": request.to_socket_id,
    //                 "to_socket_name":  &to_socket.name(),
    //     }),
    // );

    ctx.commit().await?;

    let response = axum::response::Response::builder();
    // TODO(nick): restore this with the new engine.
    // if let Some(force_changeset_pk) = force_changeset_pk {
    //     response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    // }
    Ok(response
        .header("content-type", "application/json")
        .body(serde_json::to_string(&CreateConnectionResponse {
            id: attribute_prototype_argument_id,
            // TODO(nick): figure out what to do with these fields that were left over from the "Connection" struct.
            created_by: None,
            deleted_by: None,
        })?)?)
}
