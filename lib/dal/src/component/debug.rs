use petgraph::graph::NodeIndex;
use petgraph::Direction;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::value::AttributeValueError;

use crate::attribute::value::debug::AttributeDebugViewError;
use crate::prop::PropError;
use crate::workspace_snapshot::edge_weight::EdgeWeightKind;
use crate::workspace_snapshot::node_weight::NodeWeightError;
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    func::execution::FuncExecutionError, AttributeValue, AttributeValueId, Component, ComponentId,
    DalContext, PropId, SchemaVariantId, SecretError, SecretId,
};
use crate::{ComponentError, FuncError, SchemaVariantError};

type ComponentDebugViewResult<T> = Result<T, ComponentDebugViewError>;

/// A generated view for an [`Component`](crate::Component) that contains metadata about each of
/// the components attributes. Used for constructing a debug view of the component and also for
/// cloning a component
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentDebugView {
    pub name: String,
    pub schema_variant_id: SchemaVariantId,
    pub attribute_tree: HashMap<AttributeValueId, Vec<AttributeValueId>>,
    // pub input_sockets: Vec<AttributeDebugView>,
    // pub output_sockets: Vec<AttributeDebugView>,
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ComponentDebugViewError {
    #[error(transparent)]
    AttributeDebugViewError(#[from] AttributeDebugViewError),
    #[error(transparent)]
    AttributeValue(#[from] AttributeValueError),
    #[error("Attribute Value tree badly constructed with root prop of {0}")]
    AttributeValueTreeBad(AttributeValueId),
    #[error("component error: {0}")]
    Component(String),
    #[error(transparent)]
    ComponentError(#[from] ComponentError),
    #[error(transparent)]
    Func(#[from] FuncError),
    #[error(transparent)]
    FuncExecution(#[from] FuncExecutionError),
    #[error("json pointer not found: {1:?} at {0}")]
    JSONPointerNotFound(serde_json::Value, String),
    #[error(transparent)]
    NodeWeightError(#[from] NodeWeightError),
    #[error("no internal provider for prop {0}")]
    NoInternalProvider(PropId),
    #[error("no root prop found for schema variant {0}")]
    NoRootProp(SchemaVariantId),
    #[error("schema variant not found for component {0}")]
    NoSchemaVariant(ComponentId),
    #[error("component not found {0}")]
    NotFound(ComponentId),
    #[error(transparent)]
    Prop(#[from] PropError),
    #[error(transparent)]
    SchemaVariant(#[from] SchemaVariantError),
    #[error(transparent)]
    Secret(#[from] SecretError),
    #[error("secret not found: {0}")]
    SecretNotFound(SecretId),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    UlidDecode(#[from] ulid::DecodeError),
    #[error(transparent)]
    WorkspaceSnapshotError(#[from] WorkspaceSnapshotError),
}

impl ComponentDebugView {
    #[instrument(level = "info", skip_all)]
    pub async fn new(ctx: &DalContext, component: &Component) -> ComponentDebugViewResult<Self> {
        let schema_variant = Component::schema_variant(component, ctx).await?;

        let child_values =
            Self::get_attribute_value_tree_for_component(ctx, component.id()).await?;

        let name = component
            .name(ctx)
            .await
            .map_err(|e| ComponentDebugViewError::Component(format!("get name error: {}", e)))?;

        let debug_view = ComponentDebugView {
            name,
            schema_variant_id: schema_variant.id(),
            attribute_tree: child_values,
        };

        Ok(debug_view)
    }
    #[instrument(level = "info", skip_all)]
    pub async fn get_attribute_value_tree_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> ComponentDebugViewResult<HashMap<AttributeValueId, Vec<AttributeValueId>>> {
        let mut child_values = HashMap::new();
        // Get the root attribute value and load it into the work queue.
        let root_attribute_value_id = Component::root_attribute_value_id(ctx, component_id).await?;
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut work_queue = VecDeque::from([root_attribute_value_id]);
        while let Some(attribute_value_id) = work_queue.pop_front() {
            // Collect all child attribute values.
            let mut cache: Vec<(AttributeValueId, Option<String>)> = Vec::new();
            {
                let mut child_attribute_values_with_keys_by_id: HashMap<
                    AttributeValueId,
                    (NodeIndex, Option<String>),
                > = HashMap::new();

                for (edge_weight, _, target_idx) in workspace_snapshot
                    .edges_directed(attribute_value_id, Direction::Outgoing)
                    .await?
                {
                    if let EdgeWeightKind::Contain(key) = edge_weight.kind() {
                        let child_id = workspace_snapshot
                            .get_node_weight(target_idx)
                            .await?
                            .id()
                            .into();
                        child_attribute_values_with_keys_by_id
                            .insert(child_id, (target_idx, key.to_owned()));
                    }
                }

                let maybe_ordering =
                    AttributeValue::get_child_av_ids_for_ordered_parent(ctx, attribute_value_id)
                        .await
                        .ok();
                // Ideally every attribute value with children is connected via an ordering node
                // We don't error out on ordering not existing here because we don't have that
                // guarantee. If that becomes a certainty we should fail on maybe_ordering==None.
                for av_id in maybe_ordering.unwrap_or_else(|| {
                    child_attribute_values_with_keys_by_id
                        .keys()
                        .cloned()
                        .collect()
                }) {
                    let (child_attribute_value_node_index, key) =
                        &child_attribute_values_with_keys_by_id[&av_id];
                    let child_attribute_value_node_weight = workspace_snapshot
                        .get_node_weight(*child_attribute_value_node_index)
                        .await?;
                    let content =
                        child_attribute_value_node_weight.get_attribute_value_node_weight()?;
                    cache.push((content.id().into(), key.clone()));
                }
            }

            // Now that we have the child props, prepare debug views and load the work queue.
            let mut child_attribute_value_ids = Vec::new();
            for (child_attribute_value_id, _key) in cache {
                let child_attribute_value =
                    AttributeValue::get_by_id(ctx, child_attribute_value_id).await?;

                // Load the work queue with the child attribute value.
                work_queue.push_back(child_attribute_value_id);

                // Cache the  prop values to eventually insert into the child property editor values map.
                child_attribute_value_ids.push(child_attribute_value.id());
            }
            child_values.insert(attribute_value_id, child_attribute_value_ids);
        }
        return Ok(child_values);
    }
}
