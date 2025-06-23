use std::collections::HashMap;

use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    AttributeValue,
    Component,
    Prop,
    WsEvent,
    prop::PropPath,
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

use super::{
    ComponentPropKey,
    ComponentV1RequestPath,
    ComponentViewV1,
    SecretPropKey,
    connections::{
        Connection,
        handle_connection,
    },
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
        ComponentsError,
        components::get_component::into_front_end_type,
    },
};

#[utoipa::path(
    put,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}",
    params(
        ("workspace_id" = String, Path, description = "Workspace identifier"),
        ("change_set_id" = String, Path, description = "Change Set identifier"),
        ("component_id" = String, Path, description = "Component identifier")
    ),
    tag = "components",
    summary = "Update a component",
    request_body = UpdateComponentV1Request,
    responses(
        (status = 200, description = "Component updated successfully", body = UpdateComponentV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component not found"),
        (status = 412, description = "Precondition failed - Duplicate component name"),
        (status = 422, description = "Validation error - Invalid request data", body = crate::service::v1::common::ApiError),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
#[allow(deprecated)]
pub async fn update_component(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ComponentV1RequestPath { component_id }): Path<ComponentV1RequestPath>,
    payload: Result<Json<UpdateComponentV1Request>, axum::extract::rejection::JsonRejection>,
) -> Result<Json<UpdateComponentV1Response>, ComponentsError> {
    let Json(payload) = payload?;
    let component = Component::get_by_id(ctx, component_id).await?;

    let old_name = component.name(ctx).await?;

    if let Some(name) = payload.name {
        component.set_name(ctx, name.as_str()).await?;

        tracker.track(
            ctx,
            "api_component_renamed",
            json!({
                "component_id": component_id,
                "old_name": old_name,
                "new_name": name.clone(),
            }),
        );

        ctx.write_audit_log(
            AuditLogKind::RenameComponent {
                component_id,
                old_name,
                new_name: name.clone(),
            },
            name.clone(),
        )
        .await?;
    }

    let schema_variant = component.schema_variant(ctx).await?;
    let variant_id = schema_variant.id;

    let av_id = component.domain_prop_attribute_value(ctx).await?;

    let before_domain_tree = AttributeValue::get_by_id(ctx, av_id)
        .await?
        .view(ctx)
        .await?;
    let before_value = serde_json::to_value(before_domain_tree)?;

    for (key, value) in payload.domain.clone().into_iter() {
        let prop_id = key.prop_id(ctx, variant_id).await?;
        let attribute_value_id =
            Component::attribute_value_for_prop_id(ctx, component_id, prop_id).await?;
        AttributeValue::update(ctx, attribute_value_id, Some(value.clone())).await?;
    }

    for (key, value) in payload.secrets.clone().into_iter() {
        let prop_id = key.prop_id(ctx, variant_id).await?;

        let secret_id = resolve_secret_id(ctx, &value).await?;

        let attribute_value_id =
            Component::attribute_value_for_prop_id(ctx, component_id, prop_id).await?;
        dal::Secret::attach_for_attribute_value(ctx, attribute_value_id, Some(secret_id)).await?;
    }

    for unset in payload.unset.iter() {
        let prop_id = unset.prop_id(ctx, variant_id).await?;
        let attribute_value_id =
            Component::attribute_value_for_prop_id(ctx, component_id, prop_id).await?;
        AttributeValue::use_default_prototype(ctx, attribute_value_id).await?;
    }

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

    let after_domain_tree = AttributeValue::get_by_id(ctx, av_id)
        .await?
        .view(ctx)
        .await?;
    let after_value = serde_json::to_value(after_domain_tree)?;

    let component_list = Component::list_ids(ctx).await?;
    if !payload.connection_changes.add.is_empty() || !payload.connection_changes.remove.is_empty() {
        for connection in payload.connection_changes.add.iter() {
            handle_connection(
                ctx,
                connection,
                component_id,
                variant_id,
                &component_list,
                true,
            )
            .await?;
        }

        for connection in payload.connection_changes.remove.iter() {
            handle_connection(
                ctx,
                connection,
                component_id,
                variant_id,
                &component_list,
                false,
            )
            .await?;
        }
    };

    for (av_to_set, sub) in payload.subscriptions.clone().into_iter() {
        handle_subscription(ctx, av_to_set, &sub, component_id, &component_list).await?;
    }

    // Send a websocket event about the component update
    let updated_component = Component::get_by_id(ctx, component_id).await?;
    let new_name = updated_component.name(ctx).await?;
    WsEvent::component_updated(
        ctx,
        into_front_end_type(ctx, updated_component.clone()).await?,
    )
    .await?
    .publish_on_commit(ctx)
    .await?;

    ctx.write_audit_log(
        AuditLogKind::UpdateComponent {
            component_id: updated_component.id(),
            component_name: new_name.clone(),
            before_domain_tree: Some(before_value),
            after_domain_tree: Some(after_value),
            added_connections: None,
            deleted_connections: None,
            added_secrets: payload.secrets.len(),
        },
        new_name.clone(),
    )
    .await?;

    tracker.track(
        ctx,
        "api_update_component",
        json!({
            "component_id": component.id(),
            "component_name": new_name.clone(),
            "added_connections": payload.connection_changes.add.len(),
            "deleted_connections": payload.connection_changes.remove.len(),
            "updated_props": payload.domain.len() + payload.unset.len(),
            "updated_secrets": payload.secrets.len(),
        }),
    );

    ctx.commit().await?;

    Ok(Json(UpdateComponentV1Response {
        component: ComponentViewV1::assemble(ctx, component_id).await?,
    }))
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComponentV1Request {
    #[schema(example = "MyUpdatedComponentName")]
    pub name: Option<String>,

    #[schema(example = json!({"propId1": "value1", "path/to/prop": "value2"}))]
    #[serde(default)]
    pub domain: HashMap<ComponentPropKey, serde_json::Value>,

    #[schema(example = "i-12345678")]
    pub resource_id: Option<String>,

    #[schema(example = json!({"secretDefinitionName": "secretId", "secretDefinitionName": "secretName"}))]
    #[serde(default)]
    pub secrets: HashMap<SecretPropKey, serde_json::Value>,

    #[schema(value_type = Vec<String>, example = json!(["propId1", "path/to/prop"]))]
    #[serde(default)]
    pub unset: Vec<ComponentPropKey>,

    #[schema(example = json!({}))]
    #[serde(default)]
    #[deprecated]
    pub connection_changes: ConnectionDetails,

    #[serde(default)]
    #[schema(example = json!({"/prop/path/on/this/component": {"component": "OtherComponentName", "propPath": "/prop/path/on/other/component", "keepOtherSubscriptions": true}}))]
    pub subscriptions: HashMap<AttributeValueIdent, Subscription>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionDetails {
    #[serde(default)]
    #[deprecated]
    #[schema(example = json!({}))]
    pub add: Vec<Connection>,

    #[serde(default)]
    #[deprecated]
    #[schema(example = json!({}))]
    pub remove: Vec<Connection>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComponentV1Response {
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
        "views": [{"id": "01HAXYZF3GC9CYA6ZVSM3E4YEE", "name": "Default View", "isDefault": true}],
        "sources": {"/domain/RouteTableId": {"component": "demo-component","propPath": "/resource_value/RouteTableId"}}
    }))]
    pub component: ComponentViewV1,
}
