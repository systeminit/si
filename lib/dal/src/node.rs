use crate::{node_position::NodePositionView, DalContext};
use serde::{Deserialize, Serialize};
use si_data::{NatsError, PgError};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    generate_name, impl_standard_model, pk, schema::variant::SchemaVariantError,
    schematic::SchematicKind, standard_model, standard_model_accessor, standard_model_belongs_to,
    Component, ComponentId, HistoryEventError, ReadTenancyError, Schema, SchemaId, SchemaVariantId,
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
#[serde(rename_all = "camelCase")]
pub struct NodeTemplate {
    name: String,
    pub title: String,
    kind: SchematicKind,
    schema_variant_id: SchemaVariantId,
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

        Ok(NodeTemplate {
            kind: (*schema.kind()).into(),
            title: schema.name().to_owned(),
            name: generate_name(None),
            schema_variant_id,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Copy, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum NodeViewKind {
    #[serde(rename_all = "camelCase")]
    Component { component_id: ComponentId },
    #[serde(rename_all = "camelCase")]
    Deployment { component_id: ComponentId },
}

/// This maps to the typescript SchematicNode, and can go from the database
/// representation of a node, combined with the schema data.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct NodeView {
    id: NodeId,
    name: String,
    title: String,
    kind: NodeViewKind,
    schema_variant_id: SchemaVariantId,
    positions: Vec<NodePositionView>,
}

impl NodeView {
    pub fn new(
        name: impl Into<String>,
        node: &Node,
        kind: NodeViewKind,
        positions: Vec<NodePositionView>,
        node_template: NodeTemplate,
    ) -> Self {
        let name = name.into();
        NodeView {
            id: node.id,
            name,
            kind,
            schema_variant_id: node_template.schema_variant_id,
            title: node_template.title,
            positions,
        }
    }

    pub fn id(&self) -> &NodeId {
        &self.id
    }

    pub fn positions(&self) -> &[NodePositionView] {
        &self.positions
    }
}
