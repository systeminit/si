use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use dal::node::NodeId;
use dal::{
    action_prototype::ActionPrototypeContextField, generate_name_from_schema_name, Action,
    ActionKind, ActionPrototype, ActionPrototypeContext, ChangeSet, Component, ComponentId, Schema,
    SchemaId, StandardModel, Visibility, WsEvent,
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

    let schema = Schema::get_by_id(&ctx, &request.schema_id)
        .await?
        .ok_or(DiagramError::SchemaNotFound)?;
    let name = generate_name_from_schema_name(schema.name());

    let schema_variant_id = schema
        .default_schema_variant_id()
        .ok_or(DiagramError::SchemaVariantNotFound)?;

    let (component, mut node) = Component::new(&ctx, &name, *schema_variant_id).await?;

    for prototype in ActionPrototype::find_for_context_and_kind(
        &ctx,
        ActionKind::Create,
        ActionPrototypeContext::new_for_context_field(ActionPrototypeContextField::SchemaVariant(
            *schema_variant_id,
        )),
    )
    .await?
    {
        let action = Action::new(&ctx, *prototype.id(), *component.id()).await?;
        let prototype = action.prototype(&ctx).await?;
        let component = action.component(&ctx).await?;

        track(
            &posthog_client,
            &ctx,
            &original_uri,
            "create_action",
            serde_json::json!({
                "how": "/diagram/create_node",
                "prototype_id": prototype.id(),
                "prototype_kind": prototype.kind(),
                "component_id": component.id(),
                "component_name": component.name(&ctx).await?,
                "change_set_pk": ctx.visibility().change_set_pk,
            }),
        );
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
        connect_component_sockets_to_frame(
            &ctx,
            frame_id,
            *node.id(),
            &original_uri,
            &posthog_client,
        )
        .await?;
    }

    WsEvent::component_created(&ctx, *component.id())
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
    response = response.header("content-type", "application/json");
    Ok(response.body(serde_json::to_string(&CreateNodeResponse {
        component_id: *component.id(),
        node_id: *node.id(),
    })?)?)
}
