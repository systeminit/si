use axum::response::Json;
use dal::{
    Component,
    SchemaVariant,
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
    ComponentReference,
    operations,
};
use crate::{
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::{
        ComponentViewV1,
        ComponentsError,
    },
};

#[utoipa::path(
    post,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
    ),
    tag = "components",
    request_body = CreateComponentV1Request,
    summary = "Create a component",
    responses(
        (status = 200, description = "Component created successfully", body = CreateComponentV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component not found"),
        (status = 412, description = "Precondition Failed - View not found", body = crate::service::v1::common::ApiError),
        (status = 422, description = "Validation error - Invalid request data", body = crate::service::v1::common::ApiError),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
#[allow(deprecated)]
pub async fn create_component(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    payload: Result<Json<CreateComponentV1Request>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<CreateComponentV1Response>, ComponentsError> {
    let Json(payload) = payload?;

    if ctx.change_set_id() == ctx.get_workspace_default_change_set_id().await? {
        return Err(ComponentsError::NotPermittedOnHead);
    }

    // Lazy fetch component list only if managed_by is specified
    let component_list = if !payload.managed_by.is_empty() {
        Component::list_ids(ctx).await?
    } else {
        vec![]
    };

    // Call core logic (includes audit logs, transactional)
    let component_view = operations::create_component_core(
        ctx,
        payload.name,
        payload.schema_name.clone(),
        payload.view_name,
        payload.resource_id,
        payload.attributes.clone(),
        payload.managed_by,
        payload.use_working_copy,
        &component_list,
    )
    .await?;

    // Get variant info for tracking
    let variant = SchemaVariant::get_by_id(ctx, component_view.schema_variant_id).await?;

    // Track creation (non-transactional analytics)
    tracker.track(
        ctx,
        "api_create_component",
        json!({
            "component_id": component_view.id,
            "schema_variant_id": component_view.schema_variant_id,
            "schema_variant_name": variant.display_name().to_string(),
            "category": variant.category(),
        }),
    );

    // Commit (publishes queued audit logs)
    ctx.commit().await?;

    Ok(Json(CreateComponentV1Response {
        component: component_view,
    }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateComponentV1Request {
    #[schema(example = "i-12345678")]
    pub resource_id: Option<String>,

    #[schema(example = "MyComponentName", required = true)]
    pub name: String,

    #[schema(example = "AWS::EC2::Instance", required = true)]
    pub schema_name: String,

    #[schema(example = "MyView")]
    pub view_name: Option<String>,

    #[serde(default)]
    #[schema(example = json!({"component": "ComponentName"}), required = false)]
    pub managed_by: ComponentReference,

    #[serde(default)]
    #[schema(
        value_type = std::collections::BTreeMap<String, serde_json::Value>,
        example = json!({
            "/domain/VpcId": {
                "$source": {
                    "component": "01K0WRC69ZPEMD6SMTKC84FBWC",
                    "path": "/resource_value/VpcId"
                }
            },
            "/domain/SubnetId": {
                "$source": {
                    "component": "01K0WRC69ZPEMD6SMTKC84FBWD",
                    "path": "/resource_value/SubnetId"
                }
            },
            "/domain/Version": "1.2.3",
            "/domain/Version": {
                "$source": null
            }
        })
    )]
    pub attributes: AttributeSources,

    #[schema(example = true)]
    pub use_working_copy: Option<bool>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateComponentV1Response {
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
        "views": [
            {
                "id": "01HAXYZF3GC9CYA6ZVSM3E4YEE",
                "name": "Default View",
                "isDefault": true
            }
        ],
        "sources": [
            ["/domain/RouteTableId", {
                "$source": {
                    "component": "demo-component",
                    "path": "/resource_value/RouteTableId"
                }
            }],
            ["/domain/region", {
                "value": "us-east-1"
            }]
        ]
    }))]
    pub component: ComponentViewV1,
}
