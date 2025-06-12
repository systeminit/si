use axum::Json;
use dal::{
    ChangeSet,
    Component,
    ComponentId,
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
use si_events::audit_log::AuditLogKind;

use super::Result;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct RestoreComponentsRequest {
    pub component_ids: Vec<ComponentId>,
}

pub async fn restore_components(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Json(request): Json<RestoreComponentsRequest>,
) -> Result<ForceChangeSetResponse<()>> {
    let force_change_set_id = ChangeSet::force_new(ctx).await?;

    let mut track_payloads = Vec::new();
    let mut components = Vec::new();
    for component_id in request.component_ids {
        if let Some(component) = Component::try_get_by_id(ctx, component_id).await? {
            let schema_variant = component.schema_variant(ctx).await?;
            let schema = schema_variant.schema(ctx).await?;

            // TODO(nick): I kept "diagram" in the "how" because all existing locations use diagram for
            // these kinds of operations. We are due to overhaul our posthog usages after the new UI is
            // shipped.
            track_payloads.push(serde_json::json!({
                "how": "/diagram/restore_component",
                "component_id": component_id,
                "component_schema_name": schema.name,
                "change_set_id": ctx.change_set_id(),
            }));

            components.push((
                component,
                schema_variant.id(),
                schema_variant.display_name().to_owned(),
                schema.id(),
                schema.name,
            ));
        }
    }

    // Restore the components!
    for (component, schema_variant_id, schema_variant_name, schema_id, schema_name) in components {
        let component_name = component.name(ctx).await?;
        let component_id = component.id();
        let before_to_delete = component.to_delete();

        component.set_to_delete(ctx, false).await?;

        ctx.write_audit_log(
            AuditLogKind::RestoreComponent {
                name: component_name.to_owned(),
                component_id,
                before_to_delete,
                schema_id,
                schema_name,
                schema_variant_id,
                schema_variant_name,
            },
            component_name,
        )
        .await?;
    }

    for payload in track_payloads {
        tracker.track(ctx, "restore_component", payload);
    }

    ctx.commit().await?;

    Ok(ForceChangeSetResponse::new(force_change_set_id, ()))
}
