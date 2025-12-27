use std::collections::{
    HashMap,
    HashSet,
};

use axum::response::Json;
use dal::{
    Component,
    ComponentId,
    component::delete,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use utoipa::{
    self,
    ToSchema,
};

use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::ComponentsError,
};

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EraseManyComponentsV1Request {
    #[schema(value_type = Vec<String>)]
    pub component_ids: Vec<ComponentId>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct EraseManyComponentsV1Response {
    #[schema(value_type = Vec<String>)]
    pub erased: Vec<ComponentId>,
}

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/erase_many",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
    ),
    tag = "components",
    request_body = EraseManyComponentsV1Request,
    summary = "Erase multiple components without queuing delete actions",
    responses(
        (status = 200, description = "Components erased successfully", body = EraseManyComponentsV1Response),
        (status = 400, description = "Bad Request - Not permitted on HEAD"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn erase_many_components(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    posthog: PosthogEventTracker,
    payload: Result<Json<EraseManyComponentsV1Request>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<EraseManyComponentsV1Response>, ComponentsError> {
    let Json(payload) = payload?;

    // Validate not on HEAD change set
    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(ComponentsError::NotPermittedOnHead);
    }

    // Fetch shared data once upfront for all erasures
    let head_components: HashSet<ComponentId> =
        Component::exists_on_head_by_ids(ctx, &payload.component_ids).await?;
    let base_change_set_ctx = ctx.clone_with_base().await?;
    let mut socket_map = HashMap::new();
    let mut socket_map_head = HashMap::new();

    let mut erased = Vec::with_capacity(payload.component_ids.len());

    // Process each erase in order, stop on first error
    for (index, component_id) in payload.component_ids.iter().enumerate() {
        // Get component info before erasing (for audit log)
        let comp = Component::get_by_id(ctx, *component_id)
            .await
            .map_err(|e| ComponentsError::BulkOperationFailed {
                index,
                source: Box::new(ComponentsError::Component(e)),
            })?;

        let variant =
            comp.schema_variant(ctx)
                .await
                .map_err(|e| ComponentsError::BulkOperationFailed {
                    index,
                    source: Box::new(e.into()),
                })?;

        let name = comp
            .name(ctx)
            .await
            .map_err(|e| ComponentsError::BulkOperationFailed {
                index,
                source: Box::new(e.into()),
            })?;

        // Perform the hard delete
        delete::delete_and_process(
            ctx,
            true, // force_erase = true for hard delete
            &head_components,
            &mut socket_map,
            &mut socket_map_head,
            &base_change_set_ctx,
            *component_id,
        )
        .await
        .map_err(|e| ComponentsError::BulkOperationFailed {
            index,
            source: Box::new(e.into()),
        })?;

        // Write audit log for this erasure (transactional, queued)
        ctx.write_audit_log(
            AuditLogKind::EraseComponent {
                name: name.to_owned(),
                component_id: *component_id,
                schema_variant_id: variant.id(),
                schema_variant_name: variant.display_name().to_string(),
            },
            name,
        )
        .await
        .map_err(|e| ComponentsError::BulkOperationFailed {
            index,
            source: Box::new(ComponentsError::Transactions(e)),
        })?;

        erased.push(*component_id);
    }

    // Track bulk erase (non-transactional analytics)
    posthog.track(
        ctx,
        "api_erase_many_components",
        json!({
            "count": erased.len(),
        }),
    );

    // Commit (publishes queued audit logs transactionally)
    ctx.commit().await?;

    Ok(Json(EraseManyComponentsV1Response { erased }))
}
