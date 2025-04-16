use axum::{extract::Path, response::Json};
use dal::{
    prop::{PropPath, PropResult, PROP_PATH_SEPARATOR},
    AttributeValue, Component, Prop, PropId, SchemaVariantId, WsEvent,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use std::collections::HashMap;
use utoipa::{self, ToSchema};

use crate::extract::{change_set::ChangeSetDalContext, PosthogEventTracker};

use crate::service::v1::{components::get_component::bare_component_response, ComponentsError};

use super::ComponentV1RequestPath;

/// Request for updating component properties
#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComponentPropertiesV1Request {
    #[schema(example = json!({"propId1": "value1", "path/to/prop": "value2"}))]
    pub domain: HashMap<ComponentPropKey, serde_json::Value>,
}

/// Response for updating component properties
#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpdateComponentPropertiesV1Response {
    // Empty response, successful updates return an empty object
}

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

/// Update component properties
#[utoipa::path(
    put,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}/properties",
    params(
        ("workspace_id", description = "Workspace identifier"),
        ("change_set_id", description = "Change set identifier"),
        ("component_id", description = "Component identifier")
    ),
    tag = "components",
    request_body = UpdateComponentPropertiesV1Request,
    responses(
        (status = 200, description = "Component properties updated successfully", body = UpdateComponentPropertiesV1Response),
        (status = 404, description = "Component not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn update_component_properties(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ComponentV1RequestPath { component_id }): Path<ComponentV1RequestPath>,
    Json(payload): Json<UpdateComponentPropertiesV1Request>,
) -> Result<Json<UpdateComponentPropertiesV1Response>, ComponentsError> {
    tracker.track(ctx, "api_update_component_properties", json!(payload));

    let component = Component::get_by_id(ctx, component_id).await?;
    let component_name = component.name(ctx).await?;
    let schema_variant = component.schema_variant(ctx).await?;
    let schema_variant_id = schema_variant.id;
    let schema_variant_display_name = schema_variant.display_name().to_string();
    let schema = schema_variant.schema(ctx).await?;

    for (key, value) in payload.domain.into_iter() {
        // Update the property
        let prop_id = key.prop_id(ctx, schema_variant.id).await?;
        let prop = Prop::get_by_id(ctx, prop_id).await?;
        let attribute_value_id =
            Component::attribute_value_for_prop_id(ctx, component_id, prop_id).await?;
        let av = AttributeValue::get_by_id(ctx, attribute_value_id).await?;
        let before_value = av.value(ctx).await?;
        AttributeValue::update(ctx, attribute_value_id, Some(value.clone())).await?;

        // Log the property update
        ctx.write_audit_log(
            AuditLogKind::UpdatePropertyEditorValue {
                component_id,
                component_name: component_name.clone(),
                schema_variant_id,
                schema_variant_display_name: schema_variant_display_name.clone(),
                prop_id,
                prop_name: prop.name.clone(),
                attribute_value_id,
                before_value,
                after_value: Some(value),
            },
            prop.name.clone(),
        )
        .await?;
        let parent_prop = match Prop::parent_prop_id_by_id(ctx, prop_id).await? {
            Some(parent_prop_id) => Some(Prop::get_by_id(ctx, parent_prop_id).await?),
            None => None,
        };

        // Send the property update event to posthog
        tracker.track(
            ctx,
            "property_value_updated",
            serde_json::json!({
                "how": "/public/component/property_value_updated",
                "component_id": component_id,
                "component_schema_name": schema.name(),
                "prop_id": prop_id,
                "prop_name": prop.name,
                "parent_prop_id": parent_prop.as_ref().map(|prop| prop.id),
                "parent_prop_name": parent_prop.as_ref().map(|prop| prop.name.clone()),
                "change_set_id": ctx.change_set_id(),
            }),
        );
    }

    // Send the WsEvent indicating the component was updated
    let component = Component::get_by_id(ctx, component_id).await?;
    WsEvent::component_updated(ctx, bare_component_response(ctx, component).await?)
        .await?
        .publish_on_commit(ctx)
        .await?;

    // Commit the changes
    ctx.commit().await?;

    Ok(Json(UpdateComponentPropertiesV1Response {}))
}
