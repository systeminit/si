use std::collections::HashMap;

use axum::response::Json;
use dal::{
    AttributeValue,
    Component,
    Schema,
    cached_module::CachedModule,
    diagram::view::View,
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
    connections::{
        Connection,
        handle_connection,
    },
    update_component::ComponentPropKey,
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
        ("workspace_id", description = "Workspace identifier"),
        ("change_set_id", description = "Change set identifier"),
    ),
    tag = "components",
    request_body = CreateComponentV1Request,
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
    let module = CachedModule::find_latest_for_schema_name(ctx, payload.schema_name.as_str())
        .await?
        .ok_or(ComponentsError::SchemaNameNotFound(payload.schema_name))?;
    let variant_id = Schema::get_or_install_default_variant(ctx, module.schema_id).await?;

    let view_id: ViewId;
    if let Some(view_name) = payload.view_name {
        if let Some(view) = View::find_by_name(ctx, view_name.as_str()).await? {
            view_id = view.id();
        } else {
            return Err(ComponentsError::ViewNotFound(format!(
                "View '{}' not found",
                view_name
            )));
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
            "schema_variant_name": module.display_name.clone().unwrap_or("unknown".to_string()),
            "category": module.category.unwrap_or("unknown".to_string()),
        }),
    );
    ctx.write_audit_log(
        AuditLogKind::CreateComponent {
            name: comp_name.clone(),
            component_id: component.id(),
            schema_variant_id: variant_id,
            schema_variant_name: module.display_name.clone().unwrap_or("unknown".to_string()),
        },
        comp_name.clone(),
    )
    .await?;

    for (key, value) in payload.domain.clone().into_iter() {
        let prop_id = key.prop_id(ctx, variant_id).await?;
        let attribute_value_id =
            Component::attribute_value_for_prop_id(ctx, component.id(), prop_id).await?;
        AttributeValue::update(ctx, attribute_value_id, Some(value.clone())).await?;
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
            "updated_props": payload.domain.len(),
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
    pub component: ComponentViewV1,
}
