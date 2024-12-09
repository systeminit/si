use axum::{
    extract::{Host, OriginalUri},
    Json,
};
use dal::{
    action::{Action, ActionState},
    ChangeSet, Component, ComponentId, SchemaVariant, Visibility,
};
use serde::{Deserialize, Serialize};
use si_events::audit_log::AuditLogKind;

use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::{
        component::{ComponentError, ComponentResult},
        force_change_set_response::ForceChangeSetResponse,
    },
    track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeComponentRequest {
    pub component_id: ComponentId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

pub async fn upgrade(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<UpgradeComponentRequest>,
) -> ComponentResult<ForceChangeSetResponse<()>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let current_component = Component::get_by_id(&ctx, request.component_id).await?;
    let current_schema_variant = current_component.schema_variant(&ctx).await?;
    let schema = current_schema_variant.schema(&ctx).await?;

    let upgrade_target_variant =
        match SchemaVariant::get_unlocked_for_schema(&ctx, schema.id()).await? {
            Some(unlocked_variant) => unlocked_variant,
            None => SchemaVariant::get_default_for_schema(&ctx, schema.id()).await?,
        };

    // This is just a check to see if someone has made a request incorrectly!
    if current_schema_variant.id() == upgrade_target_variant.id() {
        return Err(ComponentError::SchemaVariantUpgradeSkipped);
    }

    // block upgrades if there are running or dispatched actions for this component!
    let current_blocking_actions = Action::find_for_states_and_component_id(
        &ctx,
        request.component_id,
        [ActionState::Dispatched, ActionState::Running].to_vec(),
    )
    .await?;
    if !current_blocking_actions.is_empty() {
        return Err(ComponentError::UpgradeSkippedDueToActions);
    }

    current_component
        .upgrade_to_new_variant(&ctx, upgrade_target_variant.id())
        .await?;

    let comp_name = current_component.name(&ctx).await?;
    ctx.write_audit_log(
        AuditLogKind::UpgradeComponent {
            name: comp_name.clone(),
            component_id: current_component.id(),
            schema_id: schema.id(),
            old_schema_variant_id: current_schema_variant.id(),
            old_schema_variant_name: current_schema_variant.display_name().to_owned(),
            new_schema_variant_id: upgrade_target_variant.id(),
            new_schema_variant_name: upgrade_target_variant.display_name().to_owned(),
        },
        comp_name,
    )
    .await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "upgrade_component",
        serde_json::json!({
            "how": "/component/upgrade_component",
            "component_id": request.component_id,
            "component_schema_variant_id": current_schema_variant.id(),
            "new_schema_variant_id": upgrade_target_variant.id(),
            "change_set_id": ctx.change_set_id(),
        }),
    );

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::empty(force_change_set_id))
}
