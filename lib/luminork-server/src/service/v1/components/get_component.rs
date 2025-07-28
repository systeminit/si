use std::collections::HashMap;

use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    ActionPrototypeId,
    Component,
};
use serde::Serialize;
use serde_json::json;
use si_id::ManagementPrototypeId;
use utoipa::{
    self,
    ToSchema,
};

use super::{
    ComponentV1RequestPath,
    ComponentViewV1,
    ComponentsError,
};
use crate::extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetComponentV1Response {
    #[schema(example = json!({
        "id": "01H9ZQD35JPMBGHH69BT0Q79AA",
        "schemaId": "01H9ZQD35JPMBGHH69BT0Q79VY",
        "schemaVariantId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
        "sockets": [],
        "domainProps": [],
        "resourceProps": [],
        "name": "My EC2 Instance",
        "resourceId": "i-1234567890abcdef0",
        "toDelete": false,
        "canBeUpgraded": true,
        "connections": [],
        "views": [
            {
                "id": "01HAXYZF3GC9CYA6ZVSM3E4YEE",
                "name": "Default View",
                "isDefault": true
            }
        ],
        "attributes": {
            "/domain/region": "us-east-1",
            "/secrets/credential": {
                "$source": {
                    "component": "demo-credential",
                    "path": "/secrets/AWS Credential"
                }
            }
        }
    }))]
    pub component: ComponentViewV1,
    #[schema(example = json!([
        {"managementPrototypeId": "01HAXYZF3GC9CYA6ZVSM3E4YFF", "funcName": "Start Instance"}
    ]))]
    pub management_functions: Vec<GetComponentV1ResponseManagementFunction>,
    #[schema(example = json!([
        {"prototypeId": "01HAXYZF3GC9CYA6ZVSM3E4YGG", "funcName": "Terminate Instance"}
    ]))]
    pub action_functions: Vec<GetComponentV1ResponseActionFunction>,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetComponentV1ResponseManagementFunction {
    #[schema(value_type = String, example = "01HAXYZF3GC9CYA6ZVSM3E4YFF")]
    pub management_prototype_id: ManagementPrototypeId,
    #[schema(example = "Start Instance")]
    pub func_name: String,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetComponentV1ResponseActionFunction {
    #[schema(value_type = String, example = "01HAXYZF3GC9CYA6ZVSM3E4YGG")]
    pub prototype_id: ActionPrototypeId,
    #[schema(example = "Terminate Instance")]
    pub func_name: String,
}

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("component_id" = String, Path, description = "Component identifier")
    ),
    tag = "components",
    summary = "Get a component by component Id",
    responses(
        (status = 200, description = "Component retrieved successfully", body = GetComponentV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get_component(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ComponentV1RequestPath { component_id }): Path<ComponentV1RequestPath>,
) -> Result<Json<GetComponentV1Response>, ComponentsError> {
    let (management_functions, action_functions) =
        super::get_component_functions(ctx, component_id).await?;

    tracker.track(
        ctx,
        "api_get_component",
        json!({
            "component_id": component_id
        }),
    );

    Ok(Json(GetComponentV1Response {
        component: ComponentViewV1::assemble(ctx, component_id).await?,
        management_functions,
        action_functions,
    }))
}

pub async fn into_front_end_type(
    ctx: &dal::DalContext,
    component: Component,
) -> Result<si_frontend_types::DiagramComponentView, ComponentsError> {
    let mut socket_map = HashMap::new();
    Ok(component
        .into_frontend_type(
            ctx,
            None,
            component.change_status(ctx).await?,
            &mut socket_map,
        )
        .await?)
}
