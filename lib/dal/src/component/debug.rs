use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::value::AttributeValueError;

use crate::attribute::value::debug::{AttributeDebugView, AttributeDebugViewError};
use crate::prop::PropError;
use crate::socket::debug::{SocketDebugView, SocketDebugViewError};
use crate::socket::input::InputSocketError;
use crate::socket::output::OutputSocketError;
use crate::workspace_snapshot::node_weight::NodeWeightError;
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    func::execution::FuncExecutionError, AttributeValue, AttributeValueId, Component, ComponentId,
    DalContext, PropId, SchemaVariantId, SecretError, SecretId,
};
use crate::{
    ComponentError, FuncError, InputSocket, InputSocketId, OutputSocket, OutputSocketId,
    SchemaVariantError,
};

type ComponentDebugViewResult<T> = Result<T, ComponentDebugViewError>;

/// A generated view for an [`Component`](crate::Component) that contains metadata about each of
/// the components attributes. Used for constructing a debug view of the component and also for
/// cloning a component
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentDebugView {
    pub name: String,
    pub schema_variant_id: SchemaVariantId,
    pub attributes: Vec<AttributeDebugView>,
    pub input_sockets: Vec<SocketDebugView>,
    pub output_sockets: Vec<SocketDebugView>,
}
/// A generated view for an [`Component`](crate::Component) that contains metadata about each of
/// the components attributes. Used for constructing a debug view of the component and also for
/// cloning a component
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentDebugData {
    pub name: String,
    pub schema_variant_id: SchemaVariantId,
    pub attribute_tree: HashMap<AttributeValueId, Vec<AttributeValueId>>,
    pub input_sockets: HashMap<InputSocketId, Vec<AttributeValueId>>,
    pub output_sockets: HashMap<OutputSocketId, Vec<AttributeValueId>>,
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ComponentDebugViewError {
    #[error("attribute debug view error: {0}")]
    AttributeDebugViewError(#[from] AttributeDebugViewError),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("Attribute Value tree badly constructed with root prop of {0}")]
    AttributeValueTreeBad(AttributeValueId),
    #[error("component error: {0}")]
    Component(String),
    #[error("component error: {0}")]
    ComponentError(#[from] ComponentError),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func execution error: {0}")]
    FuncExecution(#[from] FuncExecutionError),
    #[error("input socket error: {0}")]
    InputSocketError(#[from] InputSocketError),
    #[error("json pointer not found: {1:?} at {0}")]
    JSONPointerNotFound(serde_json::Value, String),
    #[error("node weight error: {0}")]
    NodeWeightError(#[from] NodeWeightError),
    #[error("no internal provider for prop {0}")]
    NoInternalProvider(PropId),
    #[error("no root prop found for schema variant {0}")]
    NoRootProp(SchemaVariantId),
    #[error("schema variant not found for component {0}")]
    NoSchemaVariant(ComponentId),
    #[error("component not found {0}")]
    NotFound(ComponentId),
    #[error("output socket error: {0}")]
    OutputSocketError(#[from] OutputSocketError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("secret error: {0}")]
    Secret(#[from] SecretError),
    #[error("secret not found: {0}")]
    SecretNotFound(SecretId),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("socket debug view error: {0}")]
    SocketDebugViewError(#[from] SocketDebugViewError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshotError(#[from] WorkspaceSnapshotError),
}

impl ComponentDebugView {
    #[instrument(level = "info", skip_all)]
    pub async fn new(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentDebugViewResult<Self> {
        // get ComponentDebugData
        let component = Component::get_by_id(ctx, component_id).await?;

        let component_debug_data = ComponentDebugData::new(ctx, &component).await?;
        let mut attributes = vec![];
        let mut input_sockets = vec![];
        let mut output_sockets = vec![];

        let mut cache: Vec<AttributeValueId> = vec![];
        //construct attribute value debug views from the debug data
        for (av, children) in component_debug_data.attribute_tree {
            if !cache.contains(&av) {
                let avd = AttributeDebugView::new(ctx, av, None, None).await?;
                attributes.push(avd);
                cache.push(av);
            }

            for child_av in children {
                if !cache.contains(&child_av) {
                    let child_avd = AttributeDebugView::new(ctx, child_av, None, Some(av)).await?;
                    attributes.push(child_avd);
                    cache.push(child_av);
                }
            }
        }
        //sort alphabetically by path for the view
        attributes.sort_by_key(|view| view.path.to_lowercase());

        for (input_socket, _) in component_debug_data.input_sockets {
            let avd = SocketDebugView::new_for_input_socket(ctx, input_socket).await?;
            input_sockets.push(avd);
        }
        for (output_socket, _) in component_debug_data.output_sockets {
            let avd = SocketDebugView::new_for_output_socket(ctx, output_socket).await?;
            output_sockets.push(avd);
        }

        let debug_view = ComponentDebugView {
            name: component_debug_data.name,
            schema_variant_id: component_debug_data.schema_variant_id,
            attributes,
            input_sockets,
            output_sockets,
        };

        Ok(debug_view)
    }
}

impl ComponentDebugData {
    #[instrument(level = "info", skip_all)]
    pub async fn new(ctx: &DalContext, component: &Component) -> ComponentDebugViewResult<Self> {
        let schema_variant_id = Component::schema_variant_id(ctx, component.id()).await?;

        let attribute_tree =
            Self::get_attribute_value_tree_for_component(ctx, component.id()).await?;
        let input_sockets = Self::get_input_sockets_for_component(ctx, schema_variant_id).await?;
        let output_sockets = Self::get_output_sockets_for_component(ctx, schema_variant_id).await?;
        let name = component
            .name(ctx)
            .await
            .map_err(|e| ComponentDebugViewError::Component(format!("get name error: {}", e)))?;

        let debug_view = ComponentDebugData {
            name,
            schema_variant_id,
            attribute_tree,
            input_sockets,
            output_sockets,
        };

        Ok(debug_view)
    }

    #[instrument(level = "info", skip_all)]
    pub async fn get_input_sockets_for_component(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> ComponentDebugViewResult<HashMap<InputSocketId, Vec<AttributeValueId>>> {
        let mut input_sockets = HashMap::new();
        let sockets = InputSocket::list_ids_for_schema_variant(ctx, schema_variant_id).await?;
        for socket in sockets {
            let attribute_values_for_socket =
                InputSocket::attribute_values_for_input_socket_id(ctx, socket).await?;
            input_sockets.insert(socket, attribute_values_for_socket);
        }
        Ok(input_sockets)
    }

    #[instrument(level = "info", skip_all)]
    pub async fn get_output_sockets_for_component(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
    ) -> ComponentDebugViewResult<HashMap<OutputSocketId, Vec<AttributeValueId>>> {
        let mut output_sockets = HashMap::new();
        let sockets = OutputSocket::list_ids_for_schema_variant(ctx, schema_variant_id).await?;
        for socket in sockets {
            let attribute_values_for_socket =
                OutputSocket::attribute_values_for_output_socket_id(ctx, socket).await?;
            output_sockets.insert(socket, attribute_values_for_socket);
        }
        Ok(output_sockets)
    }

    #[instrument(level = "info", skip_all)]
    pub async fn get_attribute_value_tree_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentDebugViewResult<HashMap<AttributeValueId, Vec<AttributeValueId>>> {
        Ok(AttributeValue::tree_for_component(ctx, component_id).await?)
    }
}
