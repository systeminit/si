use axum::Json;
use dal::{
    ChangeSet,
    Component,
    ComponentId,
    SchemaVariant,
    action::Action,
};
use sdf_core::force_change_set_response::ForceChangeSetResponse;
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ActionState,
    audit_log::AuditLogKind,
};

use super::{
    Error,
    Result,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub component_ids: Vec<ComponentId>,
}

pub async fn upgrade_components(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Json(Request { component_ids }): Json<Request>,
) -> Result<ForceChangeSetResponse<()>> {
    let force_change_set_id = ChangeSet::force_new(ctx).await?;

    for component_id in component_ids {
        let current_component = Component::get_by_id(ctx, component_id).await?;
        let current_schema_variant = current_component.schema_variant(ctx).await?;
        let schema = current_schema_variant.schema(ctx).await?;

        let upgrade_target_variant =
            match SchemaVariant::get_unlocked_for_schema(ctx, schema.id()).await? {
                Some(unlocked_variant) => unlocked_variant,
                None => SchemaVariant::default_for_schema(ctx, schema.id()).await?,
            };

        // This is just a check to see if someone has made a request incorrectly!
        if current_schema_variant.id() == upgrade_target_variant.id() {
            return Err(Error::SchemaVariantUpgradeSkipped);
        }

        // block upgrades if there are running or dispatched actions for this component!
        let current_blocking_actions = Action::find_for_states_and_component_id(
            ctx,
            component_id,
            [ActionState::Dispatched, ActionState::Running].to_vec(),
        )
        .await?;
        if !current_blocking_actions.is_empty() {
            return Err(Error::UpgradeSkippedDueToActions);
        }

        Component::upgrade_to_new_variant(ctx, current_component.id(), upgrade_target_variant.id())
            .await?;

        let comp_name = current_component.name(ctx).await?;
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

        tracker.track(
            ctx,
            "upgrade_components",
            serde_json::json!({
                "component_id": component_id,
                "component_schema_variant_id": current_schema_variant.id(),
                "new_schema_variant_id": upgrade_target_variant.id(),
                "change_set_id": ctx.change_set_id(),
            }),
        );
    }

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, ()))
}
