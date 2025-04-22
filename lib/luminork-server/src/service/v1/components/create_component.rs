use axum::response::Json;
use dal::diagram::view::View;
use dal::socket::input::InputSocket;
use dal::socket::output::OutputSocket;
use dal::{AttributeValue, Component, ComponentId, Schema};
use serde::{Deserialize, Serialize};
use si_id::ViewId;
use std::collections::HashMap;
use utoipa::{self, ToSchema};

use crate::extract::{change_set::ChangeSetDalContext, PosthogEventTracker};

use crate::service::v1::ComponentsError;

use super::update_component::ComponentPropKey;

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
        (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
    )
)]
pub async fn create_component(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    _tracker: PosthogEventTracker,
    Json(payload): Json<CreateComponentV1Request>,
) -> Result<Json<CreateComponentV1Response>, ComponentsError> {
    let schema = Schema::get_by_name(ctx, payload.schema_name).await?;
    let variant_id = Schema::get_or_install_default_variant(ctx, schema.id()).await?;

    let view_id: ViewId;
    if let Some(view_name) = payload.view_name {
        if let Some(view) = View::find_by_name(ctx, view_name.as_str()).await? {
            view_id = view.id();
        } else {
            let default_view = View::get_id_for_default(ctx).await?;
            view_id = default_view
        }
    } else {
        let default_view = View::get_id_for_default(ctx).await?;
        view_id = default_view
    };

    let mut component = Component::new(ctx, payload.name, variant_id, view_id).await?;
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

    for (key, value) in payload.domain.into_iter() {
        let prop_id = key.prop_id(ctx, variant_id).await?;
        let attribute_value_id =
            Component::attribute_value_for_prop_id(ctx, component.id(), prop_id).await?;
        AttributeValue::update(ctx, attribute_value_id, Some(value.clone())).await?;
    }

    if !payload.connections.is_empty() {
        let component_list = Component::list(ctx).await?;

        for connection in payload.connections.iter() {
            match connection {
                Connection::Incoming {
                    from,
                    to: to_socket_name,
                } => {
                    let source_component_id =
                        resolve_component_reference(ctx, &from.component_ref, &component_list)
                            .await?;
                    let target_socket_id =
                        find_input_socket_id(ctx, to_socket_name, variant_id).await?;
                    let source_socket_id = find_component_output_socket_id(
                        ctx,
                        source_component_id,
                        &from.socket_name,
                    )
                    .await?;

                    Component::connect(
                        ctx,
                        source_component_id,
                        source_socket_id,
                        component.id(),
                        target_socket_id,
                    )
                    .await?;
                }
                Connection::Outgoing {
                    from: from_socket_name,
                    to,
                } => {
                    let target_component_id =
                        resolve_component_reference(ctx, &to.component_ref, &component_list)
                            .await?;
                    let source_socket_id =
                        find_output_socket_id(ctx, from_socket_name, variant_id).await?;
                    let target_socket_id =
                        find_component_input_socket_id(ctx, target_component_id, &to.socket_name)
                            .await?;
                    Component::connect(
                        ctx,
                        component.id(),
                        source_socket_id,
                        target_component_id,
                        target_socket_id,
                    )
                    .await?;
                }
            }
        }
    }

    ctx.commit().await?;

    Ok(Json(CreateComponentV1Response {
        component_id: component.id(),
    }))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[schema(example = json!({"component": "ComponentName", "socket_name": "InputSocketName"}))]
#[schema(example = json!({"component_id": "01H9ZQD35JPMBGHH69BT0Q79VY", "socket_name": "OutputSocketName"}))]
pub struct ConnectionPoint {
    #[serde(flatten)]
    pub component_ref: ComponentReference,
    pub socket_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
#[serde(untagged)]
#[schema(example = json!({"component": "ComponentName"}))]
#[schema(example = json!({"component_id": "01H9ZQD35JPMBGHH69BT0Q79VY"}))]
pub enum ComponentReference {
    ByName {
        component: String,
    },
    #[serde(rename_all = "camelCase")]
    ById {
        #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VY")]
        component_id: ComponentId,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ToSchema)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum Connection {
    #[schema(example = json!({"from": {"component": "OtherComponentName", "socket_name": "output"}, "to": "ThisComponentInputSocketName"}))]
    #[schema(example = json!({"from": {"component_id": "01H9ZQD35JPMBGHH69BT0Q79VY", "socket_name": "output"}, "to": "ThisComponentInputSocketName"}))]
    Incoming { from: ConnectionPoint, to: String },
    #[schema(example = json!({"from": "ThisComponentOutputSocketName", "to": {"component": "OtherComponentName", "socket_name": "InputSocketName"}}))]
    #[schema(example = json!({"from": "ThisComponentOutputSocketName", "to": {"component_id": "01H9ZQD35JPMBGHH69BT0Q79VY", "socket_name": "InputSocketName"}}))]
    Outgoing { from: String, to: ConnectionPoint },
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateComponentV1Request {
    #[schema(example = json!({"propId1": "value1", "path/to/prop": "value2"}))]
    pub domain: HashMap<ComponentPropKey, serde_json::Value>,
    #[schema(example = "MyComponentName")]
    pub name: String,
    #[schema(example = "AWS::EC2::Instance")]
    pub schema_name: String,
    #[schema(example = "MyView")]
    pub view_name: Option<String>,
    #[schema(example = json!([
        {"from": {"component": "OtherComponentName", "socket_name": "SocketName"}, "to": "ThisComponentInputSocketName"},
        {"from": {"component_id": "01H9ZQD35JPMBGHH69BT0Q79VY", "socket_name": "SocketName"}, "to": "ThisComponentInputSocketName"},
        {"from": "ThisComponentOutputSocketName", "to": {"component": "OtherComponentName", "socket_name": "InputSocketName"}},
        {"from": "ThisComponentOutputSocketName", "to": {"component_id": "01H9ZQD35JPMBGHH69BT0Q79VY", "socket_name": "InputSocketName"}}
    ]))]
    pub connections: Vec<Connection>,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CreateComponentV1Response {
    #[schema(value_type = String, example = "01H9ZQD35JPMBGHH69BT0Q79VY")]
    pub component_id: ComponentId,
}

/// Returns the component ID if found, or appropriate error if not found or if duplicate names exist
async fn find_component_id_by_name(
    ctx: &dal::DalContext,
    component_list: &[Component],
    component_name: &str,
) -> Result<ComponentId, ComponentsError> {
    let mut matching_components = Vec::new();

    for c in component_list {
        let name = c.name(ctx).await?;
        if name == component_name {
            matching_components.push(c.id());
        }
    }

    match matching_components.len() {
        0 => Err(ComponentsError::ComponentNotFound(
            component_name.to_string(),
        )),
        1 => Ok(matching_components[0]),
        _ => Err(ComponentsError::DuplicateComponentName(
            component_name.to_string(),
        )),
    }
}

/// Helper function to resolve a component reference to a component ID
async fn resolve_component_reference(
    ctx: &dal::DalContext,
    component_ref: &ComponentReference,
    component_list: &[Component],
) -> Result<ComponentId, ComponentsError> {
    match component_ref {
        ComponentReference::ById { component_id } => Ok(*component_id),
        ComponentReference::ByName { component } => {
            find_component_id_by_name(ctx, component_list, component).await
        }
    }
}

/// Helper function to find an input socket ID by name for a given schema variant
async fn find_input_socket_id(
    ctx: &dal::DalContext,
    socket_name: &str,
    schema_variant_id: dal::SchemaVariantId,
) -> Result<si_id::InputSocketId, ComponentsError> {
    let input_socket =
        InputSocket::find_with_name_or_error(ctx, socket_name, schema_variant_id).await?;
    Ok(input_socket.id())
}

/// Helper function to find an output socket ID by name for a given schema variant
async fn find_output_socket_id(
    ctx: &dal::DalContext,
    socket_name: &str,
    schema_variant_id: dal::SchemaVariantId,
) -> Result<si_id::OutputSocketId, ComponentsError> {
    let output_socket =
        OutputSocket::find_with_name_or_error(ctx, socket_name, schema_variant_id).await?;
    Ok(output_socket.id())
}

/// Helper function to find an input socket ID by name for a specific component
async fn find_component_input_socket_id(
    ctx: &dal::DalContext,
    component_id: ComponentId,
    socket_name: &str,
) -> Result<si_id::InputSocketId, ComponentsError> {
    let component = Component::get_by_id(ctx, component_id).await?;
    let schema_variant = component.schema_variant(ctx).await?;
    let variant_id = schema_variant.id();

    find_input_socket_id(ctx, socket_name, variant_id).await
}

/// Helper function to find an output socket ID by name for a specific component
async fn find_component_output_socket_id(
    ctx: &dal::DalContext,
    component_id: ComponentId,
    socket_name: &str,
) -> Result<si_id::OutputSocketId, ComponentsError> {
    let component = Component::get_by_id(ctx, component_id).await?;
    let schema_variant = component.schema_variant(ctx).await?;
    let variant_id = schema_variant.id();

    find_output_socket_id(ctx, socket_name, variant_id).await
}
