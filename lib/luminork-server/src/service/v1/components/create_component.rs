use std::collections::HashMap;

use axum::response::Json;
use dal::{
    AttributeValue,
    Component,
    Prop,
    Schema,
    SchemaVariant,
    Secret,
    attribute::attributes::AttributeSources,
    cached_module::CachedModule,
    diagram::view::View,
    prop::PropPath,
};
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use si_id::ViewId;
use utoipa::{
    self,
    ToSchema,
};

use super::{
    ComponentPropKey,
    ComponentReference,
    SecretPropKey,
    connections::{
        Connection,
        handle_connection,
    },
    resolve_component_reference,
    resolve_secret_id,
    subscriptions::{
        AttributeValueIdent,
        Subscription,
        handle_subscription,
    },
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

    let schema_id =
        match CachedModule::find_latest_for_schema_name(ctx, payload.schema_name.as_str()).await? {
            Some(module) => module.schema_id,
            None => match Schema::get_by_name_opt(ctx, payload.schema_name.as_str()).await? {
                Some(schema) => schema.id(),
                None => return Err(ComponentsError::SchemaNameNotFound(payload.schema_name)),
            },
        };
    let variant_id = Schema::get_or_install_default_variant(ctx, schema_id).await?;
    let variant = SchemaVariant::get_by_id(ctx, variant_id).await?;

    let view_id: ViewId;
    if let Some(view_name) = payload.view_name {
        if let Some(view) = View::find_by_name(ctx, view_name.as_str()).await? {
            view_id = view.id();
        } else {
            let view = View::new(ctx, view_name.as_str()).await?;
            view_id = view.id()
        }
    } else {
        let default_view = View::get_id_for_default(ctx).await?;
        view_id = default_view
    };

    let mut component = Component::new(ctx, payload.name, variant_id, view_id).await?;
    let comp_name = component.name(ctx).await?;
    let initial_geometry = component.geometry(ctx, view_id).await?;
    component
        .set_geometry(
            ctx,
            view_id,
            0,
            0,
            initial_geometry.width(),
            initial_geometry.height(),
        )
        .await?;

    tracker.track(
        ctx,
        "api_create_component",
        json!({
            "component_id": component.id(),
            "schema_variant_id": variant_id,
            "schema_variant_name": variant.display_name().to_string(),
            "category": variant.category(),
        }),
    );
    ctx.write_audit_log(
        AuditLogKind::CreateComponent {
            name: comp_name.clone(),
            component_id: component.id(),
            schema_variant_id: variant_id,
            schema_variant_name: variant.display_name().to_string(),
        },
        comp_name.clone(),
    )
    .await?;

    if !payload.attributes.is_empty() {
        dal::update_attributes(ctx, component.id(), payload.attributes.clone()).await?;
    }

    let component_list = Component::list_ids(ctx).await?;
    if let Some(resource_id) = payload.resource_id {
        let resource_prop_path = ["root", "si", "resourceId"];
        let resource_prop_id =
            Prop::find_prop_id_by_path(ctx, variant_id, &PropPath::new(resource_prop_path)).await?;

        let av_for_resource_id =
            Component::attribute_value_for_prop_id(ctx, component.id(), resource_prop_id).await?;

        AttributeValue::update(
            ctx,
            av_for_resource_id,
            Some(serde_json::to_value(resource_id)?),
        )
        .await?;
    }

    if !payload.managed_by.is_empty() {
        let manager_component_id =
            resolve_component_reference(ctx, &payload.managed_by, &component_list).await?;

        Component::manage_component(ctx, manager_component_id, component.id()).await?;
    }

    // Ideally a user wouldn't set these as well - it would be pain for them
    for (key, value) in payload.secrets.clone().into_iter() {
        let prop_id = key.prop_id(ctx, variant_id).await?;

        let secret_id = resolve_secret_id(ctx, &value).await?;

        let attribute_value_id =
            Component::attribute_value_for_prop_id(ctx, component.id(), prop_id).await?;
        Secret::attach_for_attribute_value(ctx, attribute_value_id, Some(secret_id)).await?;
    }

    for (key, value) in payload.domain.clone().into_iter() {
        let prop_id = key.prop_id(ctx, variant_id).await?;
        let attribute_value_id =
            Component::attribute_value_for_prop_id(ctx, component.id(), prop_id).await?;
        AttributeValue::update(ctx, attribute_value_id, Some(value.clone())).await?;
    }

    for (av_to_set, sub) in payload.subscriptions.clone().into_iter() {
        handle_subscription(ctx, av_to_set, &sub, component.id(), &component_list).await?;
    }

    for connection in payload.connections.iter() {
        handle_connection(
            ctx,
            connection,
            component.id(),
            variant_id,
            &component_list,
            true,
        )
        .await?;
    }

    ctx.write_audit_log(
        AuditLogKind::UpdateComponent {
            component_id: component.id(),
            component_name: comp_name.clone(),
            before_domain_tree: None,
            after_domain_tree: None,
            added_connections: None,
            deleted_connections: None,
            added_secrets: payload.secrets.len(),
        },
        comp_name.clone(),
    )
    .await?;

    tracker.track(
        ctx,
        "api_update_component",
        json!({
            "component_id": component.id(),
            "component_name": comp_name.clone(),
            "added_connections": payload.connections.len(),
            "deleted_connections": "0",
            "updated_props": payload.domain.len(),
            "updated_secrets": payload.secrets.len()
        }),
    );

    ctx.commit().await?;

    Ok(Json(CreateComponentV1Response {
        component: ComponentViewV1::assemble(ctx, component.id()).await?,
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

    #[deprecated(
        note = "Secrets deprecated in favour of using attributes parameter and will be removed in a future version of the API"
    )]
    #[schema(example = json!({}))]
    #[serde(default)]
    pub secrets: HashMap<SecretPropKey, serde_json::Value>,

    #[serde(default)]
    #[deprecated(
        note = "Connections deprecated - socket connections no longer supported and will be removed in a future version of the API"
    )]
    #[schema(example = json!({}))]
    pub connections: Vec<Connection>,

    #[deprecated(
        note = "Domain deprecated in favour of using attributes parameter and will be removed in a future version of the API"
    )]
    #[schema(example = json!({}))]
    #[serde(default)]
    pub domain: HashMap<ComponentPropKey, serde_json::Value>,

    #[deprecated(
        note = "Subscriptions deprecated in favour of using attributes parameter and will be removed in a future version of the API"
    )]
    #[schema(example = json!({}))]
    #[serde(default)]
    pub subscriptions: HashMap<AttributeValueIdent, Subscription>,
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
        "connections": [],
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
