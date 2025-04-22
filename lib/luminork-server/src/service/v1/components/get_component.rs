use std::collections::HashMap;

use axum::{extract::Path, response::Json};
use dal::{
    AttributeValue, Component,
    diagram::{geometry::Geometry, view::View},
    management::prototype::ManagementPrototype,
};
use serde::{Deserialize, Serialize};
use si_frontend_types::{DiagramComponentView, GeometryAndView};
use si_id::ManagementPrototypeId;
use utoipa::{self, ToSchema};

use crate::extract::change_set::ChangeSetDalContext;

use super::{ComponentV1RequestPath, ComponentsError};

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetComponentV1Response {
    #[schema(value_type = Object, example = json!({"id": "01FXNV4P306V3KGZ73YSVN8A60", "name": "Example Component"}))]
    pub component: DiagramComponentView,
    #[schema(example = json!({"propA": "valueA", "propB": "valueB"}))]
    pub domain: Option<serde_json::Value>,
    pub view_data: Vec<GeometryAndViewAndName>,
    pub management_functions: Vec<GetComponentV1ResponseManagementFunction>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GeometryAndViewAndName {
    #[serde(flatten)]
    #[schema(value_type = Object, example = json!({"viewId": "01FXNV4P306V3KGZ73YSVN8A60", "geometry": {"x": 100, "y": 200}}))]
    pub geometry_and_view: GeometryAndView,
    pub name: String,
}

#[derive(Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetComponentV1ResponseManagementFunction {
    #[schema(value_type = String)]
    pub management_prototype_id: ManagementPrototypeId,
    pub name: String,
}

#[utoipa::path(
    get,
    path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/components/{component_id}",
    params(
        ("workspace_id", description = "Workspace identifier"),
        ("change_set_id", description = "Change set identifier"),
        ("component_id", description = "Component identifier")
    ),
    tag = "components",
    responses(
        (status = 200, description = "Component retrieved successfully", body = GetComponentV1Response),
        (status = 401, description = "Unauthorized - Invalid or missing token"),
        (status = 404, description = "Component not found"),
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn get_component(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    Path(ComponentV1RequestPath { component_id }): Path<ComponentV1RequestPath>,
) -> Result<Json<GetComponentV1Response>, ComponentsError> {
    let component = Component::get_by_id(ctx, component_id).await?;
    let domain_av_id = component.domain_prop_attribute_value(ctx).await?;
    let domain = AttributeValue::get_by_id(ctx, domain_av_id)
        .await?
        .view(ctx)
        .await?;

    let mut view_data = vec![];
    for (view_id, geometry) in Geometry::by_view_for_component_id(ctx, component_id).await? {
        view_data.push(GeometryAndViewAndName {
            geometry_and_view: si_frontend_types::GeometryAndView {
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
        .map(|prototype| GetComponentV1ResponseManagementFunction {
            management_prototype_id: prototype.id,
            name: prototype.name,
        })
        .collect();

    Ok(Json(GetComponentV1Response {
        component: bare_component_response(ctx, component).await?,
        domain,
        view_data,
        management_functions,
    }))
}

pub async fn bare_component_response(
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
