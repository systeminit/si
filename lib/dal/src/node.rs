use crate::DalContext;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError};
use telemetry::prelude::*;
use thiserror::Error;

use crate::schema::variant::SchemaVariantError;
use crate::socket::{Socket, SocketEdgeKind};
use crate::{
    generate_name, impl_standard_model, pk, schematic::SchematicKind, standard_model,
    standard_model_accessor, standard_model_belongs_to, Component, ComponentId, HistoryEventError,
    NodePosition, ReadTenancyError, Schema, SchemaId, SchemaVariant, SchemaVariantId,
    StandardModel, StandardModelError, System, SystemId, Timestamp, Visibility, WriteTenancy,
};

pub type ApplicationId = NodeId;

#[derive(Error, Debug)]
pub enum NodeError {
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("standard model error: {0}")]
    StandardModelError(#[from] StandardModelError),
    #[error("cannot find schema id to generate node template")]
    SchemaIdNotFound,
    #[error("cannot generate node template with missing default schema variant")]
    SchemaMissingDefaultVariant,
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("read tenancy error: {0}")]
    ReadTenancy(#[from] ReadTenancyError),
    #[error("component is None")]
    ComponentIsNone,
    #[error("could not find node with ID: {0}")]
    NotFound(NodeId),
}

pub type NodeResult<T> = Result<T, NodeError>;

pk!(NodePk);
pk!(NodeId);

#[derive(
    Deserialize,
    Serialize,
    Debug,
    Copy,
    Clone,
    PartialEq,
    Eq,
    strum_macros::Display,
    strum_macros::EnumString,
    strum_macros::AsRefStr,
    strum_macros::EnumIter,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum NodeKind {
    Component,
    Deployment,
    System,
}

impl From<NodeKind> for SchematicKind {
    fn from(kind: NodeKind) -> Self {
        match kind {
            NodeKind::Component => Self::Component,
            NodeKind::Deployment => Self::Deployment,
            NodeKind::System => Self::System,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pk: NodePk,
    id: NodeId,
    kind: NodeKind,
    #[serde(flatten)]
    tenancy: WriteTenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,
}

impl_standard_model! {
    model: Node,
    pk: NodePk,
    id: NodeId,
    table_name: "nodes",
    history_event_label_base: "node",
    history_event_message_name: "Node"
}

impl Node {
    #[instrument(skip_all)]
    pub async fn new(ctx: &DalContext<'_, '_>, kind: &NodeKind) -> NodeResult<Self> {
        let row = ctx
            .txns()
            .pg()
            .query_one(
                "SELECT object FROM node_create_v1($1, $2, $3)",
                &[ctx.write_tenancy(), ctx.visibility(), &kind.as_ref()],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(kind, Enum(NodeKind), NodeResult);

    standard_model_belongs_to!(
        lookup_fn: component,
        set_fn: set_component,
        unset_fn: unset_component,
        table: "node_belongs_to_component",
        model_table: "components",
        belongs_to_id: ComponentId,
        returns: Component,
        result: NodeResult,
    );

    standard_model_belongs_to!(
        lookup_fn: system,
        set_fn: set_system,
        unset_fn: unset_system,
        table: "node_belongs_to_system",
        model_table: "systems",
        belongs_to_id: SystemId,
        returns: System,
        result: NodeResult,
    );
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct NodeLabel {
    pub title: String,
    name: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub enum NodeComponentType {
    Application,
    Computing,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NodeClassification {
    pub component: NodeComponentType,
    pub kind: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NodeDisplay {
    pub color: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NodeTemplate {
    pub kind: NodeKind,
    pub label: NodeLabel,
    pub classification: NodeClassification,
    pub input: Vec<Socket>,
    pub output: Vec<Socket>,
    pub display: NodeDisplay,
    pub schema_variant_id: SchemaVariantId,
}

impl NodeTemplate {
    pub async fn new_from_schema_id(
        ctx: &DalContext<'_, '_>,
        schema_id: SchemaId,
    ) -> NodeResult<Self> {
        let schema = Schema::get_by_id(ctx, &schema_id)
            .await?
            .ok_or(NodeError::SchemaIdNotFound)?;
        let schema_variant_id = *schema
            .default_schema_variant_id()
            .ok_or(NodeError::SchemaMissingDefaultVariant)?;
        let schema_variant = SchemaVariant::get_by_id(ctx, &schema_variant_id)
            .await?
            .ok_or(NodeError::SchemaMissingDefaultVariant)?;

        let sockets = schema_variant.sockets(ctx).await?;
        let mut outputs = vec![];
        let mut inputs = vec![];
        for socket in sockets.into_iter() {
            match socket.edge_kind() {
                SocketEdgeKind::Output => outputs.push(socket),
                _ => inputs.push(socket),
            }
        }

        let node_name = schema.name().to_string();
        Ok(NodeTemplate {
            kind: (*schema.kind()).into(),
            label: NodeLabel {
                title: node_name.clone(),
                // name: node_name.clone(),
                name: generate_name(None),
            },
            // eventually, this needs to come from the schema itself
            classification: NodeClassification {
                component: NodeComponentType::Application,
                kind: node_name,
            },
            input: inputs,
            output: outputs,
            display: NodeDisplay {
                color: "0x32b832".to_string(),
            },
            schema_variant_id,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum NodeKindWithBaggage {
    #[serde(rename_all = "camelCase")]
    Component {
        component_id: ComponentId,
    },
    #[serde(rename_all = "camelCase")]
    Deployment {
        component_id: ComponentId,
    },
    System,
}

/// This maps to the typescript node, and can go from the database
/// representation of a node, combined with the schema data.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NodeView {
    id: NodeId,
    kind: NodeKindWithBaggage,
    label: NodeLabel,
    classification: NodeClassification,
    position: Vec<NodePosition>,
    input: Vec<Socket>,
    output: Vec<Socket>,
    display: NodeDisplay,
    last_updated: DateTime<Utc>,
    checksum: serde_json::Value,
}

impl NodeView {
    pub fn new(
        name: impl Into<String>,
        node: &Node,
        kind: NodeKindWithBaggage,
        position: Vec<NodePosition>,
        node_template: NodeTemplate,
    ) -> Self {
        let name = name.into();
        NodeView {
            id: node.id,
            kind,
            label: NodeLabel {
                name,
                title: node_template.label.title,
            },
            classification: node_template.classification,
            position,
            input: node_template.input,
            output: node_template.output,
            display: node_template.display,
            last_updated: node.timestamp.updated_at,
            // What is this for?
            checksum: serde_json::json!["j4j4j4j4j4j4j4j4j4j4j4"],
        }
    }

    pub fn id(&self) -> &NodeId {
        &self.id
    }

    pub fn positions(&self) -> &[NodePosition] {
        &self.position
    }
}
