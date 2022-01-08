use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_data::{NatsError, NatsTxn, PgError, PgTxn};
use telemetry::prelude::*;
use thiserror::Error;

use crate::schema::variant::SchemaVariantError;
use crate::socket::Socket;
use crate::{
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_belongs_to,
    Component, ComponentId, HistoryActor, HistoryEventError, NodePosition, Schema, SchemaId,
    SchemaVariant, StandardModel, StandardModelError, Tenancy, Timestamp, Visibility,
};

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
}

pub type NodeResult<T> = Result<T, NodeError>;

pk!(NodePk);
pk!(NodeId);

#[derive(
    Deserialize,
    Serialize,
    Debug,
    Clone,
    PartialEq,
    Eq,
    strum_macros::Display,
    strum_macros::EnumString,
    strum_macros::AsRefStr,
    strum_macros::EnumIter,
)]
pub enum NodeKind {
    Component,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct Node {
    pk: NodePk,
    id: NodeId,
    kind: NodeKind,
    #[serde(flatten)]
    tenancy: Tenancy,
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
    #[tracing::instrument(skip(txn, nats))]
    pub async fn new(
        txn: &PgTxn<'_>,
        nats: &NatsTxn,
        tenancy: &Tenancy,
        visibility: &Visibility,
        history_actor: &HistoryActor,
        kind: &NodeKind,
    ) -> NodeResult<Self> {
        let row = txn
            .query_one(
                "SELECT object FROM node_create_v1($1, $2, $3)",
                &[&tenancy, &visibility, &kind.to_string()],
            )
            .await?;
        let object = standard_model::finish_create_from_row(
            txn,
            nats,
            tenancy,
            visibility,
            history_actor,
            row,
        )
        .await?;
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
pub struct NodeTemplate {
    pub kind: NodeKind,
    pub label: NodeLabel,
    pub classification: NodeClassification,
    pub input: Vec<Socket>,
    pub output: Vec<Socket>,
    pub display: NodeDisplay,
}

impl NodeTemplate {
    pub async fn new_from_schema_id(
        txn: &PgTxn<'_>,
        tenancy: &Tenancy,
        visibility: &Visibility,
        schema_id: SchemaId,
    ) -> NodeResult<Self> {
        let schema = Schema::get_by_id(txn, tenancy, visibility, &schema_id)
            .await?
            .ok_or(NodeError::SchemaIdNotFound)?;
        let schema_variant_id = schema
            .default_schema_variant_id()
            .ok_or(NodeError::SchemaMissingDefaultVariant)?;
        let schema_variant = SchemaVariant::get_by_id(txn, tenancy, visibility, schema_variant_id)
            .await?
            .ok_or(NodeError::SchemaMissingDefaultVariant)?;
        let sockets = schema_variant.sockets(txn, visibility).await?;

        let node_name = schema.name().to_string();
        Ok(NodeTemplate {
            kind: NodeKind::Component,
            label: NodeLabel {
                title: node_name.clone(),
                name: node_name.clone(),
            },
            // eventually, this needs to come from the schema itself
            classification: NodeClassification {
                component: NodeComponentType::Application,
                kind: node_name,
            },
            input: sockets.clone(),
            output: sockets,
            display: NodeDisplay {
                color: "0x32b832".to_string(),
            },
        })
    }
}

// This maps to the typescript node, and can go from the database
// representation of a node, combined with the schema data.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct NodeView {
    id: NodeId,
    kind: NodeKind,
    label: NodeLabel,
    classification: NodeClassification,
    position: Vec<NodePosition>,
    input: Vec<Socket>,
    output: Vec<Socket>,
    display: NodeDisplay,
    last_updated: DateTime<Utc>,
    checksum: serde_json::Value,
    schematic: serde_json::Value,
}

impl NodeView {
    pub fn new(
        name: impl Into<String>,
        node: Node,
        position: Vec<NodePosition>,
        node_template: NodeTemplate,
    ) -> Self {
        let name = name.into();
        NodeView {
            id: node.id,
            kind: node_template.kind,
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
            // This feels redundant
            schematic: serde_json::json![{
                "deployment": true,
                "component": true,
            }],
        }
    }
}
