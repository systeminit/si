use std::collections::HashMap;

use axum::{
    extract::Path,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::put,
    Json, Router,
};
use dal::{
    prop::{PropPath, PropResult, PROP_PATH_SEPARATOR},
    AttributeValue, Component, ComponentId, Prop, PropId, SchemaVariantId, WsEvent,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use si_events::audit_log::AuditLogKind;
use thiserror::Error;

use crate::extract::{change_set::ChangeSetDalContext, PosthogEventTracker};
use crate::AppState;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangeSetsError {
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] dal::attribute::value::AttributeValueError),
    #[error("component error: {0}")]
    Component(#[from] dal::ComponentError),
    #[error("dal change set error: {0}")]
    DalChangeSet(#[from] dal::ChangeSetError),
    #[error("prop error: {0}")]
    Prop(#[from] dal::prop::PropError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] dal::WsEventError),
}

type Result<T> = std::result::Result<T, ChangeSetsError>;

impl IntoResponse for ChangeSetsError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}

// /api/public/workspaces/:workspace_id/change-sets/:change_set_id/components
pub fn routes() -> Router<AppState> {
    Router::new().nest(
        "/:component_id",
        Router::new().route("/properties", put(update_component_properties)),
    )
}

async fn update_component_properties(
    ChangeSetDalContext(ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(UpdateComponentPropertiesPath { component_id }): Path<UpdateComponentPropertiesPath>,
    Json(payload): Json<UpdateComponentPropertiesRequest>,
) -> Result<Json<UpdateComponentPropertiesResponse>> {
    tracker.track(&ctx, "update_component_properties", json!(payload));

    let component = Component::get_by_id(&ctx, component_id).await?;
    let component_name = component.name(&ctx).await?;
    let schema_variant = component.schema_variant(&ctx).await?;
    let schema_variant_id = schema_variant.id;
    let schema_variant_display_name = schema_variant.display_name().to_string();
    let schema = schema_variant.schema(&ctx).await?;

    for (key, value) in payload.domain.into_iter() {
        // Update the property
        let prop_id = key.prop_id(&ctx, schema_variant.id).await?;
        let prop = Prop::get_by_id(&ctx, prop_id).await?;
        let attribute_value_id =
            Component::attribute_value_for_prop_id(&ctx, component_id, prop_id).await?;
        let av = AttributeValue::get_by_id(&ctx, attribute_value_id).await?;
        let before_value = av.value(&ctx).await?;
        AttributeValue::update(&ctx, attribute_value_id, Some(value.clone())).await?;

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
        let parent_prop = match Prop::parent_prop_id_by_id(&ctx, prop_id).await? {
            Some(parent_prop_id) => Some(Prop::get_by_id(&ctx, parent_prop_id).await?),
            None => None,
        };

        // Send the property update event to posthog
        tracker.track(
            &ctx,
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
    let mut socket_map = HashMap::new();
    let payload = component
        .into_frontend_type(
            &ctx,
            None,
            component.change_status(&ctx).await?,
            &mut socket_map,
        )
        .await?;
    WsEvent::component_updated(&ctx, payload)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    // Commit the changes
    ctx.commit().await?;

    Ok(Json(UpdateComponentPropertiesResponse {}))
}

#[derive(Deserialize)]
struct UpdateComponentPropertiesPath {
    component_id: ComponentId,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct UpdateComponentPropertiesRequest {
    domain: HashMap<ComponentPropKey, serde_json::Value>,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct UpdateComponentPropertiesResponse {}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
enum ComponentPropKey {
    PropId(PropId),
    PropPath(DomainPropPath),
}

impl ComponentPropKey {
    async fn prop_id(
        &self,
        ctx: &dal::DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> PropResult<PropId> {
        match self {
            ComponentPropKey::PropId(prop_id) => Ok(*prop_id),
            ComponentPropKey::PropPath(path) => {
                Prop::find_prop_id_by_path(ctx, schema_variant_id, &path.to_prop_path()).await
            }
        }
    }
}

// A prop path, starting from root/domain, with / instead of PROP_PATH_SEPARATOR as its separator
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Hash)]
struct DomainPropPath(String);

impl DomainPropPath {
    fn to_prop_path(&self) -> PropPath {
        PropPath::new(["root", "domain"]).join(&self.0.replace("/", PROP_PATH_SEPARATOR).into())
    }
}
