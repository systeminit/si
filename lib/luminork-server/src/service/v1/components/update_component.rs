use std::collections::HashMap;

use axum::{
    extract::Path,
    response::Json,
};
use dal::{
    AttributeValue,
    Component,
    PropId,
    SchemaVariantId,
    WsEvent,
    prop::{
        PROP_PATH_SEPARATOR,
        PropPath,
        PropResult,
    },
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
    ComponentV1RequestPath,
    connections::{
        Connection,
        handle_connection,
    },
};
use crate::{
    api_types::component::v1::ComponentViewV1,
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
    service::v1::{
        ComponentsError,
        components::get_component::into_front_end_type,
    },
};

/// Component property key
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash, ToSchema)]
#[serde(untagged)]
pub enum ComponentPropKey {
    #[schema(value_type = String)]
    PropId(PropId),
    PropPath(DomainPropPath),
}

impl ComponentPropKey {
    pub async fn prop_id(
        &self,
        ctx: &dal::DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> PropResult<PropId> {
        match self {
            ComponentPropKey::PropId(prop_id) => Ok(*prop_id),
            ComponentPropKey::PropPath(path) => {
                dal::Prop::find_prop_id_by_path(ctx, schema_variant_id, &path.to_prop_path()).await
            }
        }
    }
}

/// A prop path, starting from root/domain, with / instead of PROP_PATH_SEPARATOR as its separator
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash, ToSchema)]
pub struct DomainPropPath(pub String);

impl DomainPropPath {
    pub fn to_prop_path(&self) -> PropPath {
        PropPath::new(["root", "domain"]).join(&self.0.replace("/", PROP_PATH_SEPARATOR).into())
    }
}

#[utoipa::path(
    put,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}",
    params(
        ("workspace_id", description = "Workspace identifier"),
        ("change_set_id", description = "Change set identifier"),
        ("component_id", description = "Component identifier")
    ),
    tag = "components",
    request_body = UpdateComponentV1Request,
    responses(
        (status = 200, description = "Component updated successfully", body = UpdateComponentV1Response),
        (status = 404, description = "Component not found"),
        (status = 412, description = "Precondition failed - Duplicate component name"),
        (status = 422, description = "Validation error - Invalid request data", body = crate::service::v1::common::ApiError),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
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

    for unset in payload.unset.iter() {
        let prop_id = unset.prop_id(ctx, variant_id).await?;
        let attribute_value_id =
            Component::attribute_value_for_prop_id(ctx, component_id, prop_id).await?;
        AttributeValue::use_default_prototype(ctx, attribute_value_id).await?;
    }

    let after_domain_tree = AttributeValue::get_by_id(ctx, av_id)
        .await?
        .view(ctx)
        .await?;
    let after_value = serde_json::to_value(after_domain_tree)?;

    let component_list = Component::list_ids(ctx).await?;
    let added_connection_summary = super::connections::summarise_connections(
        ctx,
        &payload.connection_changes.add,
        &component_list,
    )
    .await?;
    let removed_connection_summary = super::connections::summarise_connections(
        ctx,
        &payload.connection_changes.remove,
        &component_list,
    )
    .await?;
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
            added_connections: Some(added_connection_summary),
            deleted_connections: Some(removed_connection_summary),
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

    #[schema(value_type = Vec<String>, example = json!(["propId1", "path/to/prop"]))]
    #[serde(default)]
    pub unset: Vec<ComponentPropKey>,

    #[schema(example = json!({
        "add": [
            {"from": {"component": "OtherComponentName", "socketName": "output"}, "to": "ThisComponentInputSocketName"},
            {"from": "ThisComponentOutputSocketName", "to": {"componentId": "01H9ZQD35JPMBGHH69BT0Q79VY", "socketName": "InputSocketName"}}
        ],
        "remove": [
            {"from": {"componentId": "01H9ZQD35JPMBGHH69BT0Q79VY", "socketName": "output"}, "to": "ThisComponentInputSocketName"}
        ]
    }))]
    #[schema(example = json!({
        "add": [
            {"from": {"component": "OtherComponentName", "socketName": "output"}, "to": "ThisComponentInputSocketName"}
        ]
    }))]
    #[schema(example = json!({
        "remove": [
            {"from": {"componentId": "01H9ZQD35JPMBGHH69BT0Q79VY", "socketName": "output"}, "to": "ThisComponentInputSocketName"}
        ]
    }))]
    #[schema(example = json!({}))]
    #[serde(default)]
    pub connection_changes: ConnectionDetails,
}

#[derive(Deserialize, Serialize, Debug, ToSchema, Default)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionDetails {
    #[schema(example = json!([
        {"from": {"component": "OtherComponentName", "socketName": "output"}, "to": "ThisComponentInputSocketName"},
        {"from": "ThisComponentOutputSocketName", "to": {"componentId": "01H9ZQD35JPMBGHH69BT0Q79VY", "socketName": "InputSocketName"}}
    ]))]
    #[serde(default)]
    pub add: Vec<Connection>,

    #[schema(example = json!([
        {"from": {"componentId": "01H9ZQD35JPMBGHH69BT0Q79VY", "socketName": "output"}, "to": "ThisComponentInputSocketName"},
        {"from": "ThisComponentOutputSocketName", "to": {"component": "OtherComponentName", "socketName": "InputSocketName"}}
    ]))]
    #[serde(default)]
    pub remove: Vec<Connection>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComponentV1Response {
    pub component: ComponentViewV1,
}
