use crate::{node_position::NodePositionView, DalContext, SchemaError};
use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    diagram::DiagramKind, generate_name, impl_standard_model, pk,
    schema::variant::SchemaVariantError, standard_model, standard_model_accessor,
    standard_model_belongs_to, Component, ComponentId, HistoryEventError, ReadTenancyError, Schema,
    SchemaId, SchemaVariantId, StandardModel, StandardModelError, Timestamp, Visibility,
    WriteTenancy,
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
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
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

/// The kind of a given [`Node`](Node) that corresponds to the [`DiagramKind`](crate::DiagramKind).
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
    /// The [`Node`](Node) used within [`configuration`](crate::DiagramKind::Configuration)
    /// diagrams.
    Configuration,
}

/// A mathematical node that can be used to create [`Edges`](crate::Edge).
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
    pub async fn new(ctx: &DalContext, kind: &NodeKind) -> NodeResult<Self> {
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
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NodeTemplate {
    name: String,
    pub title: String,
    kind: DiagramKind,
    schema_variant_id: SchemaVariantId,
}

impl NodeTemplate {
    /// Creates [`self`](Self) for a given [`SchemaId`](crate::Schema). The resulting template will
    /// be of [`DiagramKind::Configuration`](crate::DiagramKind::Configuration).
    pub async fn new_for_schema(ctx: &DalContext, schema_id: SchemaId) -> NodeResult<Self> {
        let schema = Schema::get_by_id(ctx, &schema_id)
            .await?
            .ok_or(NodeError::SchemaIdNotFound)?;
        let schema_variant_id = *schema
            .default_schema_variant_id()
            .ok_or(NodeError::SchemaMissingDefaultVariant)?;

        Ok(NodeTemplate {
            kind: DiagramKind::Configuration,
            title: schema.name().to_owned(),
            name: generate_name(),
            schema_variant_id,
        })
    }
}

/// This maps to the typescript DiagramNode, and can go from the database
/// representation of a node, combined with the schema data.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NodeView {
    id: NodeId,
    name: String,
    title: String,
    component_id: ComponentId,
    schema_variant_id: SchemaVariantId,
    positions: Vec<NodePositionView>,
}

impl NodeView {
    pub fn new(
        name: impl Into<String>,
        node: &Node,
        component_id: ComponentId,
        positions: Vec<NodePositionView>,
        node_template: NodeTemplate,
    ) -> Self {
        let name = name.into();
        NodeView {
            id: node.id,
            name,
            component_id,
            schema_variant_id: node_template.schema_variant_id,
            title: node_template.title,
            positions,
        }
    }

    pub fn id(&self) -> &NodeId {
        &self.id
    }

    pub fn component_id(&self) -> ComponentId {
        self.component_id
    }

    pub fn positions(&self) -> &[NodePositionView] {
        &self.positions
    }
}
