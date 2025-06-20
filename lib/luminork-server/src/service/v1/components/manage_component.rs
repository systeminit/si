use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    Component,
    ComponentError,
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
    ComponentV1RequestPath,
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
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/manage",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("component_id" = String, Path, description = "Component identifier")
    ),
    tag = "components",
    request_body = ManageComponentV1Request,
    summary = "Putting a component under the management of another component",
    responses(
        (status = 200, description = "Component successfully under management", body = ManageComponentV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn manage_component(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ComponentV1RequestPath { component_id }): Path<ComponentV1RequestPath>,
    payload: Result<Json<ManageComponentV1Request>, axum::extract::rejection::JsonRejection>,
) -> ComponentsResult<Json<ManageComponentV1Response>> {
    let Json(payload) = payload?;

    let _manager_component = Component::get_by_id(ctx, component_id)
        .await
        .map_err(|_e| ComponentError::NotFound(component_id))?;
    let _component_to_manage = Component::get_by_id(ctx, payload.component_id)
        .await
        .map_err(|_e| ComponentError::NotFound(payload.component_id))?;

    Component::manage_component(ctx, component_id, payload.component_id).await?;

    ctx.commit().await?;

    tracker.track(
        ctx,
        "set_management_edge",
        json!({
            "manager": component_id,
            "managee": payload.component_id,
        }),
    );

    Ok(Json(ManageComponentV1Response {
        component: ComponentViewV1::assemble(ctx, component_id).await?,
    }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ManageComponentV1Request {
    #[serde(rename = "componentId")]
    #[schema(value_type = String, required = true)]
    pub component_id: ComponentId,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ManageComponentV1Response {
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
