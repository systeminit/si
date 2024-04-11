use axum::{extract::OriginalUri, http::uri::Uri};
use axum::{response::IntoResponse, Json};
use dal::{ChangeSet, Component, ComponentId, DalContext, Visibility, WsEvent};
use serde::{Deserialize, Serialize};

use super::DiagramResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

async fn delete_single_component(
    ctx: &DalContext,
    component_id: ComponentId,
    original_uri: &Uri,
    PosthogClient(posthog_client): &PosthogClient,
) -> DiagramResult<()> {
    let comp = Component::get_by_id(ctx, component_id).await?;

    let comp_schema = comp.schema(ctx).await?;

    // XXX: Most of this should probably go away by being moved into Component::delete
    //
    // let comp_schema_variant = comp.schema_variant(ctx).await?;
    //
    // let resource = comp.resource(ctx).await?;
    //
    // // TODO: this is tricky, we don't want to delete all actions, but we don't need to be perfect
    // // right now, let's see how the usage plays with users
    // let actions = Action::find_for_change_set(ctx).await?;
    // for mut action in actions {
    //     if action.component_id() == comp.id() {
    //         action.delete_by_id(ctx).await?;
    //     }
    // }
    //
    // if resource.payload.is_some() {
    //     for prototype in ActionPrototype::find_for_context_and_kind(
    //         ctx,
    //         ActionKind::Delete,
    //         ActionPrototypeContext::new_for_context_field(
    //             ActionPrototypeContextField::SchemaVariant(*comp_schema_variant.id()),
    //         ),
    //     )
    //     .await?
    //     {
    //         let action = Action::upsert(ctx, *prototype.id(), *comp.id()).await?;
    //         let prototype = action.prototype(ctx).await?;
    //
    //         track(
    //             posthog_client,
    //             ctx,
    //             original_uri,
    //             "create_action",
    //             serde_json::json!({
    //                 "how": "/diagram/delete_component",
    //                 "prototype_id": prototype.id(),
    //                 "prototype_kind": prototype.kind(),
    //                 "component_id": comp.id(),
    //                 "component_name": comp.name(ctx).await?,
    //                 "change_set_pk": ctx.visibility().change_set_pk,
    //             }),
    //         );
    //     }
    // }

    let comp = comp.delete(ctx).await?;

    track(
        posthog_client,
        ctx,
        original_uri,
        "delete_component",
        serde_json::json!({
            "component_id": comp.id(),
            "component_schema_name": comp_schema.name(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    ctx.commit().await?;

    Ok(())
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct DeleteComponentsRequest {
    pub component_ids: Vec<ComponentId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

/// Delete a set of [`Component`](dal::Component)s via their componentId. Creates change-set if on head
pub async fn delete_components(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    posthog_client: PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<DeleteComponentsRequest>,
) -> DiagramResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    for component_id in request.component_ids {
        delete_single_component(&ctx, component_id, &original_uri, &posthog_client).await?;

        WsEvent::component_updated(&ctx, component_id)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    }

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(axum::body::Empty::new())?)
}
