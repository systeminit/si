use std::collections::HashMap;

use axum::response::Json;
use dal::{
    ComponentId,
    attribute::attributes::AttributeSources,
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
        components::update_component::SecretPropKey,
    },
};

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateManyComponentsV1Request {
    pub components: Vec<UpdateComponentItemV1>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComponentItemV1 {
    #[schema(value_type = String)]
    pub component_id: ComponentId,

    #[schema(example = "MyUpdatedComponentName")]
    pub name: Option<String>,

    #[schema(example = "i-12345678")]
    pub resource_id: Option<String>,

    #[serde(default)]
    #[schema(example = json!({"secretDefinitionName": "secretId"}))]
    pub secrets: HashMap<SecretPropKey, serde_json::Value>,

    #[serde(default)]
    #[schema(
        value_type = std::collections::BTreeMap<String, serde_json::Value>,
        example = json!({
            "/domain/VpcId": {
                "$source": {
                    "component": "01K0WRC69ZPEMD6SMTKC84FBWC",
                    "path": "/resource_value/VpcId"
                }
            }
        })
    )]
    pub attributes: AttributeSources,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateManyComponentsV1Response {
    pub components: Vec<ComponentViewV1>,
}

#[utoipa::path(
    put,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/update_many",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
    ),
    tag = "components",
    request_body = UpdateManyComponentsV1Request,
    summary = "Update multiple components",
    responses(
        (status = 200, description = "Components updated successfully", body = UpdateManyComponentsV1Response),
        (status = 400, description = "Bad Request - Not permitted on HEAD"),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn update_many_components(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    posthog: PosthogEventTracker,
    payload: Result<Json<UpdateManyComponentsV1Request>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<UpdateManyComponentsV1Response>, ComponentsError> {
    let Json(payload) = payload?;

    // Validate not on HEAD change set
    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(ComponentsError::NotPermittedOnHead);
    }

    let mut results = Vec::with_capacity(payload.components.len());

    // Process each component update in order, stop on first error
    for (index, request) in payload.components.iter().enumerate() {
        // Call core logic (includes audit logs, WsEvents, transactional)
        let component = operations::update_component_core(
            ctx,
            request.component_id,
            request.name.clone(),
            request.resource_id.clone(),
            request.secrets.clone(),
            request.attributes.clone(),
        )
        .await
        .map_err(|e| ComponentsError::BulkOperationFailed {
            index,
            source: Box::new(e),
        })?;

        results.push(component);
    }

    // Track bulk update (non-transactional analytics)
    posthog.track(
        ctx,
        "api_update_many_components",
        json!({
            "count": results.len(),
        }),
    );

    // Commit (publishes queued audit logs and WsEvents transactionally)
    ctx.commit().await?;

    Ok(Json(UpdateManyComponentsV1Response {
        components: results,
    }))
}
