use axum::response::Json;
use dal::{
    Component,
    ComponentId,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use utoipa::{
    self,
    ToSchema,
};

use super::{
    ComponentViewV1,
    operations,
};
use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::{
        ComponentsError,
        components::create_component::CreateComponentV1Request,
    },
};

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateManyComponentsV1Request {
    pub components: Vec<CreateComponentV1Request>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateManyComponentsV1Response {
    pub components: Vec<ComponentViewV1>,
}

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/create_many",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
    ),
    tag = "components",
    request_body = CreateManyComponentsV1Request,
    summary = "Create multiple components",
    responses(
        (status = 200, description = "Components created successfully", body = CreateManyComponentsV1Response),
        (status = 400, description = "Bad Request - Not permitted on HEAD"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn create_many_components(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    posthog: PosthogEventTracker,
    payload: Result<Json<CreateManyComponentsV1Request>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<CreateManyComponentsV1Response>, ComponentsError> {
    let Json(payload) = payload?;

    // Validate not on HEAD change set
    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(ComponentsError::NotPermittedOnHead);
    }

    // Lazy cache for component list (only fetch if needed by any request)
    let mut component_list_cache: Option<Vec<ComponentId>> = None;
    let mut results = Vec::with_capacity(payload.components.len());

    // Process each component creation in order, stop on first error
    for (index, request) in payload.components.iter().enumerate() {
        // Lazy fetch component list only if this item needs it
        if !request.managed_by.is_empty() && component_list_cache.is_none() {
            component_list_cache = Some(Component::list_ids(ctx).await?);
        }

        let list = component_list_cache.as_deref().unwrap_or(&[]);

        // Call core logic (includes audit logs, transactional)
        let component = operations::create_component_core(
            ctx,
            request.name.clone(),
            request.schema_name.clone(),
            request.view_name.clone(),
            request.resource_id.clone(),
            request.attributes.clone(),
            request.managed_by.clone(),
            request.use_working_copy,
            list,
        )
        .await
        .map_err(|e| ComponentsError::BulkOperationFailed {
            index,
            source: Box::new(e),
        })?;

        results.push(component);
    }

    // Track bulk creation (non-transactional analytics)
    posthog.track(
        ctx,
        "api_create_many_components",
        json!({
            "count": results.len(),
        }),
    );

    // Commit (publishes queued audit logs transactionally)
    ctx.commit().await?;

    Ok(Json(CreateManyComponentsV1Response {
        components: results,
    }))
}
