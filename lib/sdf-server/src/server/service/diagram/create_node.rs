use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use dal::edge::EdgeKind;
use dal::node::NodeId;
use dal::socket::SocketEdgeKind;
use dal::{
    action_prototype::ActionPrototypeContextField, generate_name, Action, ActionKind,
    ActionPrototype, ActionPrototypeContext, ChangeSet, Component, ComponentId, Connection, Node,
    Schema, SchemaId, Socket, StandardModel, Visibility, WsEvent,
};

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::diagram::connect_component_to_frame::connect_component_sockets_to_frame;
use crate::service::diagram::{DiagramError, DiagramResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateNodeRequest {
    pub schema_id: SchemaId,
    pub parent_id: Option<NodeId>,
    pub x: String,
    pub y: String,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateNodeResponse {
    pub component_id: ComponentId,
    pub node_id: NodeId,
}

pub async fn create_node(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<CreateNodeRequest>,
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

    let name = generate_name();
    let schema = Schema::get_by_id(&ctx, &request.schema_id)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;

    let schema_variant_id = schema
        .default_schema_variant_id()
        .ok_or(DiagramError::SchemaVariantNotFound)?;

    let (component, mut node) = Component::new(&ctx, &name, *schema_variant_id).await?;

    if let Some(prototype) = ActionPrototype::find_for_context_and_kind(
        &ctx,
        ActionKind::Create,
        ActionPrototypeContext::new_for_context_field(ActionPrototypeContextField::SchemaVariant(
            *schema_variant_id,
        )),
    )
    .await?
    .first()
    {
        Action::new(&ctx, *prototype.id(), *component.id()).await?;
    }

    node.set_geometry(
        &ctx,
        request.x.clone(),
        request.y.clone(),
        Some("500"),
        Some("500"),
    )
    .await?;

    if let Some(frame_id) = request.parent_id {
        let component_socket = Socket::find_frame_socket_for_node(
            &ctx,
            *node.id(),
            SocketEdgeKind::ConfigurationOutput,
        )
        .await?;
        let frame_socket =
            Socket::find_frame_socket_for_node(&ctx, frame_id, SocketEdgeKind::ConfigurationInput)
                .await?;

        let _connection = Connection::new(
            &ctx,
            *node.id(),
            *component_socket.id(),
            frame_id,
            *frame_socket.id(),
            EdgeKind::Symbolic,
        )
        .await?;

        connect_component_sockets_to_frame(&ctx, frame_id, *node.id()).await?;

        let child_comp = Node::get_by_id(&ctx, node.id())
            .await?
            .ok_or(DiagramError::NodeNotFound(*node.id()))?
            .component(&ctx)
            .await?
            .ok_or(DiagramError::ComponentNotFound)?;

        let child_schema = child_comp
            .schema(&ctx)
            .await?
            .ok_or(DiagramError::SchemaNotFound)?;

        let parent_comp = Node::get_by_id(&ctx, &frame_id)
            .await?
            .ok_or(DiagramError::NodeNotFound(frame_id))?
            .component(&ctx)
            .await?
            .ok_or(DiagramError::ComponentNotFound)?;

        let parent_schema = parent_comp
            .schema(&ctx)
            .await?
            .ok_or(DiagramError::SchemaNotFound)?;

        track(
            &posthog_client,
            &ctx,
            &original_uri,
            "component_connected_to_frame",
            serde_json::json!({
                        "parent_component_id": parent_comp.id(),
                        "parent_component_schema_name": parent_schema.name(),
                        "parent_socket_id": frame_socket.id(),
                        "parent_socket_name": frame_socket.name(),
                        "child_component_id": child_comp.id(),
                        "child_component_schema_name": child_schema.name(),
                        "child_socket_id": component_socket.id(),
                        "child_socket_name": component_socket.name(),
            }),
        );
    }

    WsEvent::component_created(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "component_created",
        serde_json::json!({
                    "schema_id": schema.id(),
                    "schema_name": schema.name(),
                    "schema_variant_id": &schema_variant_id,
                    "component_id": component.id(),
                    "component_name": &name,
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_changeset_pk) = force_changeset_pk {
        response = response.header("force_changeset_pk", force_changeset_pk.to_string());
    }
    Ok(response.body(serde_json::to_string(&CreateNodeResponse {
        component_id: *component.id(),
        node_id: *node.id(),
    })?)?)
}
