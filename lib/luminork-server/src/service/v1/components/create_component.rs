use std::collections::HashMap;

use axum::response::Json;
use dal::{
    AttributeValue,
    Component,
    Prop,
    Schema,
    SchemaVariant,
    Secret,
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
    SecretPropKey,
    connections::{
        Connection,
        handle_connection,
    },
    resolve_secret_id,
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
pub async fn create_component(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    payload: Result<Json<CreateComponentV1Request>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<CreateComponentV1Response>, ComponentsError> {
    let Json(payload) = payload?;

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

    for (key, value) in payload.domain.clone().into_iter() {
        let prop_id = key.prop_id(ctx, variant_id).await?;
        let attribute_value_id =
            Component::attribute_value_for_prop_id(ctx, component.id(), prop_id).await?;
        AttributeValue::update(ctx, attribute_value_id, Some(value.clone())).await?;
    }

    for (key, value) in payload.secrets.clone().into_iter() {
        let prop_id = key.prop_id(ctx, variant_id).await?;

        let secret_id = resolve_secret_id(ctx, &value).await?;

        let attribute_value_id =
            Component::attribute_value_for_prop_id(ctx, component.id(), prop_id).await?;
        Secret::attach_for_attribute_value(ctx, attribute_value_id, Some(secret_id)).await?;
    }

    let av_id = component.domain_prop_attribute_value(ctx).await?;
    let after_domain_tree = AttributeValue::get_by_id(ctx, av_id)
        .await?
        .view(ctx)
        .await?;
    let after_value = serde_json::to_value(after_domain_tree)?;

    let component_list = Component::list_ids(ctx).await?;

    let added_connection_summary =
        super::connections::summarise_connections(ctx, &payload.connections, &component_list)
            .await?;

    if !payload.connections.is_empty() {
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
    }

    ctx.write_audit_log(
        AuditLogKind::UpdateComponent {
            component_id: component.id(),
            component_name: comp_name.clone(),
            before_domain_tree: None,
            after_domain_tree: Some(after_value),
            added_connections: Some(added_connection_summary),
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
    #[schema(example = json!({"propId1": "value1", "path/to/prop": "value2"}))]
    #[serde(default)]
    pub domain: HashMap<ComponentPropKey, serde_json::Value>,

    #[schema(example = "i-12345678")]
    pub resource_id: Option<String>,

    #[schema(example = json!({"secretDefinitionName": "secretId", "secretDefinitionName": "secretName"}))]
    #[serde(default)]
    pub secrets: HashMap<SecretPropKey, serde_json::Value>,

    #[schema(example = "MyComponentName", required = true)]
    pub name: String,

    #[schema(example = "AWS::EC2::Instance", required = true)]
    pub schema_name: String,

    #[schema(example = "MyView")]
    pub view_name: Option<String>,

    #[schema(example = json!([
        {"from": {"component": "OtherComponentName", "socketName": "SocketName"}, "to": "ThisComponentInputSocketName"},
        {"from": {"componentId": "01H9ZQD35JPMBGHH69BT0Q79VY", "socketName": "SocketName"}, "to": "ThisComponentInputSocketName"},
        {"from": "ThisComponentOutputSocketName", "to": {"component": "OtherComponentName", "socketName": "InputSocketName"}},
        {"from": "ThisComponentOutputSocketName", "to": {"componentId": "01H9ZQD35JPMBGHH69BT0Q79VY", "socketName": "InputSocketName"}}
    ]))]
    #[serde(default)]
    pub connections: Vec<Connection>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateComponentV1Response {
    #[schema(example = json!({
        "id": "01H9ZQD35JPMBGHH69BT0Q79AA",
        "schemaId": "01H9ZQD35JPMBGHH69BT0Q79VY",
        "schemaVariantId": "01H9ZQD35JPMBGHH69BT0Q79VZ",
        "sockets": [{"id": "socket1", "name": "input", "direction": "input", "arity": "one", "value": null}],
        "domainProps": [{"id": "01HAXYZF3GC9CYA6ZVSM3E4YAA", "propId": "01HAXYZF3GC9CYA6ZVSM3E4YBB", "value": "my-value", "path": "domain/path"}],
        "resourceProps": [{"id": "01HAXYZF3GC9CYA6ZVSM3E4YCC", "propId": "01HAXYZF3GC9CYA6ZVSM3E4YDD", "value": "resource-value", "path": "resource/path"}],
        "name": "My EC2 Instance",
        "resourceId": "i-1234567890abcdef0",
        "toDelete": false,
        "canBeUpgraded": true,
        "connections": [],
        "views": [{"id": "01HAXYZF3GC9CYA6ZVSM3E4YEE", "name": "Default View", "isDefault": true}]
    }))]
    pub component: ComponentViewV1,
}
