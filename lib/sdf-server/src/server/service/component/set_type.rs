use axum::extract::OriginalUri;
use axum::Json;

use dal::{
    ChangeSet, ChangeSetPk, Component, ComponentId, ComponentType, StandardModel, Visibility,
    WsEvent,
};
use serde::{Deserialize, Serialize};

use super::ComponentResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::component::ComponentError;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SetTypeRequest {
    pub component_id: ComponentId,
    pub value: Option<serde_json::Value>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ForceChangeSet {
    #[serde(rename = "_forceChangesetPk")] // TODO(victor) find a way to return this as a header
    pub force_changeset_pk: Option<ChangeSetPk>,
}

pub async fn set_type(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<SetTypeRequest>,
) -> ComponentResult<Json<ForceChangeSet>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut force_changeset_pk = None;
    if ctx.visibility().is_head() {
        let change_set = ChangeSet::new(&ctx, ChangeSet::generate_name(&ctx).await?, None).await?;

        let new_visibility = Visibility::new(change_set.pk, request.visibility.deleted_at);

        ctx.update_visibility(new_visibility);

        force_changeset_pk = Some(change_set.pk);

        WsEvent::change_set_created(&ctx, change_set.pk)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    };

    let component = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .ok_or(ComponentError::ComponentNotFound(request.component_id))?;

    let component_schema = component
        .schema(&ctx)
        .await?
        .ok_or(ComponentError::SchemaNotFound)?;

    // If no type was found, default to a standard "component".
    let component_type: ComponentType = match request.value {
        Some(value) => serde_json::from_value(value)?,
        None => ComponentType::Component,
    };
    component.set_type(&ctx, component_type).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "set_component_type",
        serde_json::json!({
                    "component_id": component.id(),
                    "component_schema_name": component_schema.name(),
                    "new_component_type": component_type,
        }),
    );

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(ForceChangeSet { force_changeset_pk }))
}
