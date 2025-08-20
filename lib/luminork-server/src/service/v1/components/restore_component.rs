use axum::{
    extract::Path,
    response::Json,
};
use dal::Component;
use serde::Serialize;
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use utoipa::{
    self,
    ToSchema,
};

use super::ComponentV1RequestPath;
use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::ComponentsError,
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/restore",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("component_id" = String, Path, description = "Component identifier")
    ),
    tag = "components",
    summary = "Restore a component that is marked for deletion",
    responses(
        (status = 200, description = "Component restored successfully", body = RestoreComponentV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component not found"),
        (status = 412, description = "Component not marked for deletion"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn restore_component(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ComponentV1RequestPath { component_id }): Path<ComponentV1RequestPath>,
) -> Result<Json<RestoreComponentV1Response>, ComponentsError> {
    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(ComponentsError::NotPermittedOnHead);
    }

    let component = Component::get_by_id(ctx, component_id).await?;
    let name = component.name(ctx).await?;
    let schema = component.schema(ctx).await?;
    let variant = component.schema_variant(ctx).await?;
    if !component.to_delete() {
        return Err(ComponentsError::ComponentNotRestorable(component_id));
    }

    component.set_to_delete(ctx, false).await?;

    ctx.write_audit_log(
        AuditLogKind::RestoreComponent {
            name: name.clone(),
            component_id,
            before_to_delete: true,
            schema_id: schema.id(),
            schema_name: schema.name().to_owned(),
            schema_variant_id: variant.id,
            schema_variant_name: variant.display_name().to_string(),
        },
        name,
    )
    .await?;

    tracker.track(
        ctx,
        "api_restore_component",
        json!({
            "component_id": component_id,
            "component_schema_name": variant.display_name().to_string(),
        }),
    );

    ctx.commit().await?;

    Ok(Json(RestoreComponentV1Response { status: true }))
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct RestoreComponentV1Response {
    #[schema(example = "true")]
    pub status: bool,
}
