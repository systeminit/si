use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use dal::component::frame::Frame;
use dal::component::{DEFAULT_COMPONENT_HEIGHT, DEFAULT_COMPONENT_WIDTH};
use dal::{generate_name, Component, ComponentId, SchemaId, SchemaVariant, Visibility, WsEvent};

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::service::diagram::DiagramResult;

use super::DiagramError;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateComponentRequest {
    pub schema_id: SchemaId,
    pub parent_id: Option<ComponentId>,
    pub x: String,
    pub y: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateComponentResponse {
    pub component_id: ComponentId,
}

pub async fn create_component(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Json(request): Json<CreateComponentRequest>,
) -> DiagramResult<impl IntoResponse> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    // TODO(nick): restore this with new engine semantics.
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

    let name = generate_name();

    // TODO: restore the notion of a "default" schema variant
    let variant = SchemaVariant::list_for_schema(&ctx, request.schema_id)
        .await?
        .into_iter()
        .next()
        .ok_or(DiagramError::SchemaVariantNotFound)?;

    let component = Component::new(&ctx, &name, variant.id(), None).await?;

    // TODO(nick): restore the action prototype usage here.
    // for prototype in ActionPrototype::find_for_context_and_kind(
    //     &ctx,
    //     ActionKind::Create,
    //     ActionPrototypeContext::new_for_context_field(ActionPrototypeContextField::SchemaVariant(
    //         *schema_variant_id,
    //     )),
    // )
    // .await?
    // {
    //     let action = Action::new(&ctx, *prototype.id(), *component.id()).await?;
    //     let prototype = action.prototype(&ctx).await?;
    //     let component = action.component(&ctx).await?;
    //
    //     track(
    //         &posthog_client,
    //         &ctx,
    //         &original_uri,
    //         "create_action",
    //         serde_json::json!({
    //             "how": "/diagram/create_component",
    //             "prototype_id": prototype.id(),
    //             "prototype_kind": prototype.kind(),
    //             "component_id": component.id(),
    //             "component_name": component.name(&ctx).await?,
    //             "change_set_pk": ctx.visibility().change_set_pk,
    //         }),
    //     );
    // }

    let component = component
        .set_geometry(
            &ctx,
            request.x.clone(),
            request.y.clone(),
            Some(DEFAULT_COMPONENT_WIDTH),
            Some(DEFAULT_COMPONENT_HEIGHT),
        )
        .await?;

    if let Some(frame_id) = request.parent_id {
        Frame::attach_child_to_parent(&ctx, frame_id, component.id()).await?;
    }

    // TODO(nick): restore posthog logic and other potential missing frame logic.
    // if let Some(frame_id) = request.parent_id {
    //     let component_socket = Socket::find_frame_socket_for_node(
    //         &ctx,
    //         *node.id(),
    //         SocketEdgeKind::ConfigurationOutput,
    //     )
    //     .await?;
    //     let frame_socket =
    //         Socket::find_frame_socket_for_node(&ctx, frame_id, SocketEdgeKind::ConfigurationInput)
    //             .await?;
    //
    //     let _connection = Connection::new(
    //         &ctx,
    //         *node.id(),
    //         *component_socket.id(),
    //         frame_id,
    //         *frame_socket.id(),
    //         EdgeKind::Symbolic,
    //     )
    //     .await?;
    //
    //     connect_component_sockets_to_frame(&ctx, frame_id, *node.id()).await?;
    //
    //     let child_comp = Node::get_by_id(&ctx, node.id())
    //         .await?
    //         .ok_or(DiagramError::NodeNotFound(*node.id()))?
    //         .component(&ctx)
    //         .await?
    //         .ok_or(DiagramError::ComponentNotFound)?;
    //
    //     let child_schema = child_comp
    //         .schema(&ctx)
    //         .await?
    //         .ok_or(DiagramError::SchemaNotFound)?;
    //
    //     let parent_comp = Node::get_by_id(&ctx, &frame_id)
    //         .await?
    //         .ok_or(DiagramError::NodeNotFound(frame_id))?
    //         .component(&ctx)
    //         .await?
    //         .ok_or(DiagramError::ComponentNotFound)?;
    //
    //     let parent_schema = parent_comp
    //         .schema(&ctx)
    //         .await?
    //         .ok_or(DiagramError::SchemaNotFound)?;
    //
    //     track(
    //         &posthog_client,
    //         &ctx,
    //         &original_uri,
    //         "component_connected_to_frame",
    //         serde_json::json!({
    //                     "parent_component_id": parent_comp.id(),
    //                     "parent_component_schema_name": parent_schema.name(),
    //                     "parent_socket_id": frame_socket.id(),
    //                     "parent_socket_name": frame_socket.name(),
    //                     "child_component_id": child_comp.id(),
    //                     "child_component_schema_name": child_schema.name(),
    //                     "child_socket_id": component_socket.id(),
    //                     "child_socket_name": component_socket.name(),
    //         }),
    //     );
    // }

    WsEvent::component_created(&ctx, component.id())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    // TODO(nick): restore posthog tracking.
    // track(
    //     &posthog_client,
    //     &ctx,
    //     &original_uri,
    //     "component_created",
    //     serde_json::json!({
    //                 "schema_id": schema.id(),
    //                 "schema_name": schema.name(),
    //                 "schema_variant_id": &schema_variant_id,
    //                 "component_id": component.id(),
    //                 "component_name": &name,
    //     }),
    // );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    // TODO(nick): restore change set creation when on head.
    // if let Some(force_changeset_pk) = force_changeset_pk {
    //     response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    // }
    response = response.header("content-type", "application/json");
    Ok(
        response.body(serde_json::to_string(&CreateComponentResponse {
            component_id: component.id(),
        })?)?,
    )
}
