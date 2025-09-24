use std::collections::HashMap;

use serde::{
    Deserialize,
    Serialize,
};
use telemetry::prelude::*;
use thiserror::Error;

use super::socket::{
    ComponentInputSocket,
    ComponentOutputSocket,
};
use crate::{
    AttributeValue,
    AttributeValueId,
    Component,
    ComponentError,
    ComponentId,
    DalContext,
    FuncError,
    InputSocket,
    OutputSocket,
    PropId,
    SchemaVariantError,
    SchemaVariantId,
    SecretError,
    SecretId,
    attribute::value::{
        AttributeValueError,
        debug::{
            AttributeDebugView,
            AttributeDebugViewError,
        },
    },
    diagram::{
        DiagramError,
        geometry::Geometry,
        view::{
            View,
            ViewId,
        },
    },
    prop::PropError,
    socket::{
        debug::{
            SocketDebugView,
            SocketDebugViewError,
        },
        input::InputSocketError,
        output::OutputSocketError,
    },
    workspace_snapshot::{
        WorkspaceSnapshotError,
        node_weight::NodeWeightError,
    },
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
    pub geometry: HashMap<ViewId, Geometry>,
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
    pub input_sockets: Vec<ComponentInputSocket>,
    pub output_sockets: Vec<ComponentOutputSocket>,
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ComponentDebugViewError {
    #[error("attribute debug view error: {0}")]
    AttributeDebugViewError(#[from] Box<AttributeDebugViewError>),
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] Box<AttributeValueError>),
    #[error("Attribute Value tree badly constructed with root prop of {0}")]
    AttributeValueTreeBad(AttributeValueId),
    #[error("component error: {0}")]
    Component(String),
    #[error("component error: {0}")]
    ComponentError(#[from] Box<ComponentError>),
    #[error("diagram error: {0}")]
    Diagram(#[from] Box<DiagramError>),
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("input socket error: {0}")]
    InputSocketError(#[from] Box<InputSocketError>),
    #[error("json pointer not found: {1:?} at {0}")]
    JSONPointerNotFound(serde_json::Value, String),
    #[error("node weight error: {0}")]
    NodeWeightError(#[from] Box<NodeWeightError>),
    #[error("no internal provider for prop {0}")]
    NoInternalProvider(PropId),
    #[error("no root prop found for schema variant {0}")]
    NoRootProp(SchemaVariantId),
    #[error("schema variant not found for component {0}")]
    NoSchemaVariant(ComponentId),
    #[error("component not found {0}")]
    NotFound(ComponentId),
    #[error("output socket error: {0}")]
    OutputSocketError(#[from] Box<OutputSocketError>),
    #[error("prop error: {0}")]
    Prop(#[from] Box<PropError>),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("secret error: {0}")]
    Secret(#[from] Box<SecretError>),
    #[error("secret not found: {0}")]
    SecretNotFound(SecretId),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("socket debug view error: {0}")]
    SocketDebugViewError(#[from] Box<SocketDebugViewError>),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshotError(#[from] Box<WorkspaceSnapshotError>),
}

impl ComponentDebugView {
    #[instrument(level = "trace", skip_all)]
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

        for component_input_socket in component_debug_data.input_sockets {
            let avd = SocketDebugView::new_for_input_socket(ctx, component_input_socket).await?;
            input_sockets.push(avd);
        }
        for component_output_socket in component_debug_data.output_sockets {
            let avd = SocketDebugView::new_for_output_socket(ctx, component_output_socket).await?;
            output_sockets.push(avd);
        }

        let mut geometry = HashMap::new();
        for view in View::list(ctx).await? {
            let Some(geo) =
                Geometry::try_get_by_component_and_view(ctx, component.id, view.id()).await?
            else {
                continue;
            };

            geometry.insert(view.id(), geo);
        }

        let debug_view = ComponentDebugView {
            name: component_debug_data.name,
            schema_variant_id: component_debug_data.schema_variant_id,
            attributes,
            input_sockets,
            output_sockets,
            geometry,
        };

        Ok(debug_view)
    }
}

impl ComponentDebugData {
    #[instrument(level = "trace", skip_all)]
    pub async fn new(ctx: &DalContext, component: &Component) -> ComponentDebugViewResult<Self> {
        let schema_variant_id = Component::schema_variant_id(ctx, component.id()).await?;
        let attribute_tree =
            Self::get_attribute_value_tree_for_component(ctx, component.id()).await?;
        let input_sockets =
            Self::get_input_sockets_for_component(ctx, schema_variant_id, component.id()).await?;
        let output_sockets =
            Self::get_output_sockets_for_component(ctx, schema_variant_id, component.id()).await?;
        let name = component
            .name(ctx)
            .await
            .map_err(|e| ComponentDebugViewError::Component(format!("get name error: {e}")))?;

        let debug_view = ComponentDebugData {
            name,
            schema_variant_id,
            attribute_tree,
            input_sockets,
            output_sockets,
        };

        Ok(debug_view)
    }

    #[instrument(level = "trace", skip_all)]
    pub async fn get_input_sockets_for_component(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        component_id: ComponentId,
    ) -> ComponentDebugViewResult<Vec<ComponentInputSocket>> {
        let mut input_sockets = Vec::new();
        let sockets = InputSocket::list_ids_for_schema_variant(ctx, schema_variant_id).await?;
        for socket in sockets {
            if let Some(input_socket_match) =
                ComponentInputSocket::get_by_ids(ctx, component_id, socket).await?
            {
                input_sockets.push(input_socket_match);
            }
        }
        Ok(input_sockets)
    }

    #[instrument(level = "trace", skip_all)]
    pub async fn get_output_sockets_for_component(
        ctx: &DalContext,
        schema_variant_id: SchemaVariantId,
        component_id: ComponentId,
    ) -> ComponentDebugViewResult<Vec<ComponentOutputSocket>> {
        let mut output_sockets = Vec::new();
        let sockets = OutputSocket::list_ids_for_schema_variant(ctx, schema_variant_id).await?;
        for socket in sockets {
            if let Some(output_socket_match) =
                ComponentOutputSocket::get_by_ids(ctx, component_id, socket).await?
            {
                output_sockets.push(output_socket_match);
            }
        }

        Ok(output_sockets)
    }

    #[instrument(level = "trace", skip_all)]
    pub async fn get_attribute_value_tree_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentDebugViewResult<HashMap<AttributeValueId, Vec<AttributeValueId>>> {
        Ok(AttributeValue::tree_for_component(ctx, component_id).await?)
    }
}

impl From<AttributeDebugViewError> for ComponentDebugViewError {
    fn from(value: AttributeDebugViewError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributeValueError> for ComponentDebugViewError {
    fn from(value: AttributeValueError) -> Self {
        Box::new(value).into()
    }
}

impl From<ComponentError> for ComponentDebugViewError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}

impl From<DiagramError> for ComponentDebugViewError {
    fn from(value: DiagramError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncError> for ComponentDebugViewError {
    fn from(value: FuncError) -> Self {
        Box::new(value).into()
    }
}

impl From<InputSocketError> for ComponentDebugViewError {
    fn from(value: InputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<NodeWeightError> for ComponentDebugViewError {
    fn from(value: NodeWeightError) -> Self {
        Box::new(value).into()
    }
}

impl From<OutputSocketError> for ComponentDebugViewError {
    fn from(value: OutputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<PropError> for ComponentDebugViewError {
    fn from(value: PropError) -> Self {
        Box::new(value).into()
    }
}

impl From<SchemaVariantError> for ComponentDebugViewError {
    fn from(value: SchemaVariantError) -> Self {
        Box::new(value).into()
    }
}

impl From<SecretError> for ComponentDebugViewError {
    fn from(value: SecretError) -> Self {
        Box::new(value).into()
    }
}

impl From<SocketDebugViewError> for ComponentDebugViewError {
    fn from(value: SocketDebugViewError) -> Self {
        Box::new(value).into()
    }
}

impl From<WorkspaceSnapshotError> for ComponentDebugViewError {
    fn from(value: WorkspaceSnapshotError) -> Self {
        Box::new(value).into()
    }
}
