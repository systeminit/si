use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    Component,
    SchemaVariant,
    action::Action,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_events::ActionState;
use utoipa::{
    self,
    ToSchema,
};

use super::{
    ComponentV1RequestPath,
    ComponentsError,
    ComponentsResult,
};
use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::ComponentViewV1,
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/upgrade",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("component_id" = String, Path, description = "Component identifier")
    ),
    tag = "components",
    summary = "Upgrade a component to the latest schema variant",
    responses(
        (status = 200, description = "Component successfully upgraded", body = UpgradeComponentV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn upgrade_component(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ComponentV1RequestPath { component_id }): Path<ComponentV1RequestPath>,
) -> ComponentsResult<Json<UpgradeComponentV1Response>> {
    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(ComponentsError::NotPermittedOnHead);
    }

    let current_component = Component::get_by_id(ctx, component_id).await?;
    let current_schema_variant = current_component.schema_variant(ctx).await?;
    let schema = current_schema_variant.schema(ctx).await?;

    let upgrade_target_variant =
        match SchemaVariant::get_unlocked_for_schema(ctx, schema.id()).await? {
            Some(unlocked_variant) => unlocked_variant,
            None => SchemaVariant::default_for_schema(ctx, schema.id()).await?,
        };

    if current_schema_variant.id() == upgrade_target_variant.id() {
        return Err(ComponentsError::SchemaVariantUpgradeSkipped);
    }

    let current_blocking_actions = Action::find_for_states_and_component_id(
        ctx,
        component_id,
        [ActionState::Dispatched, ActionState::Running].to_vec(),
    )
    .await?;
    if !current_blocking_actions.is_empty() {
        return Err(ComponentsError::UpgradeSkippedDueToActions);
    }

    Component::upgrade_to_new_variant(ctx, component_id, upgrade_target_variant.id()).await?;

    ctx.commit().await?;

    tracker.track(
        ctx,
        "api_upgrade_component",
        json!({
            "component_id": component_id,
            "old_schema_variant_id": current_schema_variant.id(),
            "new_schema_variant_id": upgrade_target_variant.id(),
        }),
    );

    Ok(Json(UpgradeComponentV1Response {
        component: ComponentViewV1::assemble(ctx, component_id).await?,
    }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeComponentV1Response {
    #[schema(example = json!({
        "id": "01H9ZQD35JPMBGHH69BT0Q79AA",
        "schemaId": "01H9ZQD35JPMBGHH69BT0Q79VY",
        "schemaVariantId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
        "sockets": [{"id": "socket1", "name": "input", "direction": "input", "arity": "one", "value": null}],
        "domainProps": [{"id": "01HAXYZF3GC9CYA6ZVSM3E4YAA", "propId": "01HAXYZF3GC9CYA6ZVSM3E4YBB", "value": "updated-value", "path": "domain/path"}],
        "resourceProps": [{"id": "01HAXYZF3GC9CYA6ZVSM3E4YCC", "propId": "01HAXYZF3GC9CYA6ZVSM3E4YDD", "value": "updated-resource-value", "path": "resource/path"}],
        "name": "My Updated EC2 Instance",
        "resourceId": "i-1234567890abcdef0",
        "toDelete": false,
        "canBeUpgraded": true,
        "connections": [{"incoming": {"fromComponentId": "01H9ZQD35JPMBGHH69BT0Q79BB", "fromComponentName": "Other Component", "from": "output1", "to": "input1"}}],
        "views": [{"id": "01HAXYZF3GC9CYA6ZVSM3E4YEE", "name": "Default View", "isDefault": true}]
    }))]
    pub component: ComponentViewV1,
}
