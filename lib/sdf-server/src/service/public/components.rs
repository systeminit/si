use std::collections::HashMap;

use axum::{
    Json,
    Router,
    extract::Path,
    http::StatusCode,
    response::{
        IntoResponse,
        Response,
    },
    routing::{
        get,
        put,
    },
};
use dal::{
    AttributeValue,
    Component,
    ComponentError,
    ComponentId,
    DalContext,
    Prop,
    PropId,
    SchemaVariantId,
    WsEvent,
    diagram::{
        geometry::Geometry,
        view::View,
    },
    management::prototype::ManagementPrototype,
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
use si_frontend_types::{
    DiagramComponentView,
    GeometryAndView,
};
use si_id::ManagementPrototypeId;
use thiserror::Error;

use crate::{
    AppState,
    extract::{
        PosthogEventTracker,
        change_set::ChangeSetDalContext,
    },
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum ChangeSetsError {
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] dal::attribute::value::AttributeValueError),
    #[error("component error: {0}")]
    Component(#[from] dal::ComponentError),
    #[error("dal change set error: {0}")]
    DalChangeSet(#[from] dal::ChangeSetError),
    #[error("diagram error: {0}")]
    Diagram(#[from] dal::diagram::DiagramError),
    #[error("prop error: {0}")]
    ManagementPrototype(#[from] dal::management::prototype::ManagementPrototypeError),
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
        let status_code = match &self {
            ChangeSetsError::Component(ComponentError::NotFound(_)) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status_code, self.to_string()).into_response()
    }
}

// /api/public/workspaces/:workspace_id/change-sets/:change_set_id/components
pub fn routes() -> Router<AppState> {
    Router::new().nest(
        "/:component_id",
        Router::new()
            .route("/", get(get_component))
            .route("/properties", put(update_component_properties)),
    )
}

async fn update_component_properties(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path(ComponentRequestPath { component_id }): Path<ComponentRequestPath>,
    Json(payload): Json<UpdateComponentPropertiesRequest>,
) -> Result<Json<UpdateComponentPropertiesResponse>> {
    tracker.track(ctx, "update_component_properties", json!(payload));

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
                attribute_path: format!("/domain/{}", prop.name), // Fallback path for legacy route
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

    Ok(Json(UpdateComponentPropertiesResponse {}))
}

#[derive(Deserialize)]
struct ComponentRequestPath {
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

async fn get_component(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    // tracker: PosthogEventTracker,
    Path(ComponentRequestPath { component_id }): Path<ComponentRequestPath>,
) -> Result<Json<GetComponentResponse>> {
    let component = Component::get_by_id(ctx, component_id).await?;
    let domain_av_id = component.domain_prop_attribute_value(ctx).await?;
    let domain = AttributeValue::view(ctx, domain_av_id).await?;

    let mut view_data = vec![];
    for (view_id, geometry) in Geometry::by_view_for_component_id(ctx, component_id).await? {
        view_data.push(GeometryAndViewAndName {
            geometry_and_view: GeometryAndView {
                view_id,
                geometry: geometry.into_raw(),
            },
            name: View::get_by_id(ctx, view_id).await?.name().to_string(),
        });
    }

    let schema_variant_id = Component::schema_variant_id(ctx, component_id).await?;
    let management_functions = ManagementPrototype::list_for_variant_id(ctx, schema_variant_id)
        .await?
        .into_iter()
        .map(|prototype| GetComponentResponseManagementFunction {
            management_prototype_id: prototype.id,
            name: prototype.name,
        })
        .collect();

    Ok(Json(GetComponentResponse {
        component: bare_component_response(ctx, component).await?,
        domain,
        view_data,
        management_functions,
    }))
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GetComponentResponse {
    /// Component data
    component: DiagramComponentView,
    /// Domain props for this component
    domain: Option<serde_json::Value>,
    /// Views this component is in
    view_data: Vec<GeometryAndViewAndName>,
    /// Management functions available to this component
    management_functions: Vec<GetComponentResponseManagementFunction>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GeometryAndViewAndName {
    #[serde(flatten)]
    pub geometry_and_view: GeometryAndView,
    pub name: String,
}

#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
struct GetComponentResponseManagementFunction {
    management_prototype_id: ManagementPrototypeId,
    name: String,
}

async fn bare_component_response(
    ctx: &DalContext,
    component: Component,
) -> Result<DiagramComponentView> {
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
