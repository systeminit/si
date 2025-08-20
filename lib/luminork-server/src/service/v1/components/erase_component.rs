use std::collections::{
    HashMap,
    HashSet,
};

use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    Component,
    ComponentId,
    component::delete::{
        self,
    },
};
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
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/erase",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("component_id" = String, Path, description = "Component identifier")
    ),
    tag = "components",
    summary = "Erase a component without queuing a delete action",
    responses(
        (status = 200, description = "Component erased successfully", body = EraseComponentV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn erase_component(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ComponentV1RequestPath { component_id }): Path<ComponentV1RequestPath>,
) -> Result<Json<EraseComponentV1Response>, ComponentsError> {
    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(ComponentsError::NotPermittedOnHead);
    }

    let head_components: HashSet<ComponentId> =
        Component::exists_on_head_by_ids(ctx, &[component_id]).await?;

    let comp = Component::get_by_id(ctx, component_id).await?;
    let variant = comp.schema_variant(ctx).await?;
    let name = comp.name(ctx).await?;

    let mut socket_map = HashMap::new();
    let mut socket_map_head = HashMap::new();
    let base_change_set_ctx = ctx.clone_with_base().await?;

    delete::delete_and_process(
        ctx,
        true,
        &head_components,
        &mut socket_map,
        &mut socket_map_head,
        &base_change_set_ctx,
        component_id,
    )
    .await?;

    tracker.track(
        ctx,
        "api_erase_component",
        json!({
            "component_id": component_id,
            "component_name": name,
        }),
    );

    ctx.write_audit_log(
        AuditLogKind::EraseComponent {
            name: name.to_owned(),
            component_id,
            schema_variant_id: variant.id(),
            schema_variant_name: variant.display_name().to_string(),
        },
        name,
    )
    .await?;

    ctx.commit().await?;

    Ok(Json(EraseComponentV1Response { status: true }))
}

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EraseComponentV1Response {
    #[schema(example = "true")]
    pub status: bool,
}
