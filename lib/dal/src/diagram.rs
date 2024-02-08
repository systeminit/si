use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use std::collections::{hash_map, HashMap};
use std::num::{ParseFloatError, ParseIntError};
use strum::{AsRefStr, Display, EnumIter, EnumString};
use thiserror::Error;

use crate::actor_view::ActorView;
use crate::attribute::prototype::argument::{
    AttributePrototypeArgumentError, AttributePrototypeArgumentId,
};
use crate::attribute::value::AttributeValueError;
use crate::change_status::ChangeStatus;
use crate::component::{ComponentError, IncomingConnection};
use crate::history_event::HistoryEventMetadata;
use crate::provider::external::ExternalProviderError;
use crate::provider::internal::InternalProviderError;
use crate::schema::variant::SchemaVariantError;
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    history_event, AttributePrototypeId, Component, ComponentId, DalContext, ExternalProviderId,
    HistoryEventError, InternalProviderId, ProviderArity, SchemaId, SchemaVariant, SchemaVariantId,
    StandardModelError,
};

//pub(crate) mod summary_diagram;

// TODO(nick): this module eventually goes the way of the dinosaur.
// pub mod connection;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum DiagramError {
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute prototype argument targets not found for attribute prototype argument ({0}) found via external provider: {1}")]
    AttributePrototypeArgumentTargetsNotFound(AttributePrototypeArgumentId, ExternalProviderId),
    #[error("attribute prototype not found")]
    AttributePrototypeNotFound,
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("attribute value not found")]
    AttributeValueNotFound,
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("component not found")]
    ComponentNotFound,
    #[error("component status not found for component: {0}")]
    ComponentStatusNotFound(ComponentId),
    #[error("deletion timestamp not found")]
    DeletionTimeStamp,
    #[error("destination attribute prototype not found for inter component attribute prototype argument: {0}")]
    DestinationAttributePrototypeNotFound(AttributePrototypeArgumentId),
    #[error("destination explicit internal provider not found for attribute prototype ({0}) and inter component attribute prototype argument ({1})")]
    DestinationExplicitInternalProviderNotFound(AttributePrototypeId, AttributePrototypeArgumentId),
    #[error("edge not found")]
    EdgeNotFound,
    #[error("external provider error: {0}")]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("internal provider error: {0}")]
    InternalProvider(#[from] InternalProviderError),
    #[error("node not found")]
    NodeNotFound,
    #[error(transparent)]
    ParseFloat(#[from] ParseFloatError),
    #[error(transparent)]
    ParseInt(#[from] ParseIntError),
    #[error("pg error: {0}")]
    Pg(#[from] PgError),
    #[error("position not found")]
    PositionNotFound,
    #[error("schema not found")]
    SchemaNotFound,
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("socket not found")]
    SocketNotFound,
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type NodeId = ComponentId;
pub type EdgeId = AttributePrototypeArgumentId;

pub type DiagramResult<T> = Result<T, DiagramError>;

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct GridPoint {
    pub x: isize,
    pub y: isize,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Size2D {
    pub width: isize,
    pub height: isize,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SummaryDiagramComponent {
    pub id: ComponentId,
    pub component_id: ComponentId,
    pub schema_name: String,
    pub schema_id: SchemaId,
    pub schema_variant_id: SchemaVariantId,
    pub schema_variant_name: String,
    pub schema_category: String,
    pub sockets: serde_json::Value,
    pub node_id: NodeId,
    pub display_name: String,
    pub position: GridPoint,
    pub size: Size2D,
    pub color: String,
    pub node_type: String,
    pub change_status: String,
    pub has_resource: bool,
    pub parent_node_id: Option<NodeId>,
    pub child_node_ids: serde_json::Value,
    pub created_info: serde_json::Value,
    pub updated_info: serde_json::Value,
    pub deleted_info: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SummaryDiagramEdge {
    pub id: EdgeId,
    pub edge_id: EdgeId,
    pub from_node_id: NodeId,
    pub from_socket_id: ExternalProviderId,
    pub to_node_id: NodeId,
    pub to_socket_id: InternalProviderId,
    pub change_status: String,
    pub created_info: serde_json::Value,
    pub deleted_info: serde_json::Value,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DiagramSocket {
    pub id: String,
    pub label: String,
    pub connection_annotations: Vec<String>,
    pub direction: DiagramSocketDirection,
    pub max_connections: Option<usize>,
    pub is_required: Option<bool>,
    pub node_side: DiagramSocketNodeSide,
}

#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
enum DiagramSocketDirection {
    Bidirectional,
    Input,
    Output,
}

#[remain::sorted]
#[derive(
    AsRefStr,
    Clone,
    Copy,
    Debug,
    Deserialize,
    Display,
    EnumIter,
    EnumString,
    Eq,
    PartialEq,
    Serialize,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
enum DiagramSocketNodeSide {
    Left,
    Right,
}
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Diagram {
    pub components: Vec<SummaryDiagramComponent>,
    pub edges: Vec<SummaryDiagramEdge>,
}

impl Diagram {
    /// Assemble a [`Diagram`](Self) based on existing [`Nodes`](crate::Node) and
    /// [`Connections`](crate::Connection).
    pub async fn assemble(ctx: &DalContext) -> DiagramResult<Self> {
        let mut diagram_sockets: HashMap<SchemaVariantId, serde_json::Value> = HashMap::new();
        let mut diagram_edges: Vec<SummaryDiagramEdge> = vec![];

        let components = Component::list(ctx).await?;

        let mut component_views = Vec::with_capacity(components.len());
        for component in &components {
            for incoming_connection in component.incoming_connections(ctx).await? {
                diagram_edges.push(SummaryDiagramEdge {
                    id: incoming_connection.attribute_prototype_argument_id,
                    edge_id: incoming_connection.attribute_prototype_argument_id,
                    from_node_id: incoming_connection.from_component_id,
                    from_socket_id: incoming_connection.from_external_provider_id,
                    to_node_id: incoming_connection.to_component_id,
                    to_socket_id: incoming_connection.to_internal_provider_id,
                    change_status: ChangeStatus::Added.to_string(),
                    created_info: serde_json::to_value(incoming_connection.created_info)?,
                    deleted_info: serde_json::to_value(incoming_connection.deleted_info)?,
                });
            }

            let schema_variant = component.schema_variant(ctx).await?;

            let sockets = match diagram_sockets.entry(schema_variant.id()) {
                hash_map::Entry::Vacant(entry) => {
                    let (external_providers, internal_providers) =
                        SchemaVariant::list_external_providers_and_explicit_internal_providers(
                            ctx,
                            schema_variant.id(),
                        )
                        .await?;

                    let mut sockets = vec![];

                    for ip in internal_providers {
                        sockets.push(DiagramSocket {
                            id: ip.id().to_string(),
                            label: ip.name().to_string(),
                            connection_annotations: vec![ip.name().to_string()],
                            direction: DiagramSocketDirection::Input,
                            max_connections: match ip.arity() {
                                ProviderArity::Many => None,
                                ProviderArity::One => Some(1),
                            },
                            is_required: Some(false),
                            node_side: DiagramSocketNodeSide::Left,
                        });
                    }

                    for ep in external_providers {
                        sockets.push(DiagramSocket {
                            id: ep.id().to_string(),
                            label: ep.name().to_string(),
                            connection_annotations: vec![ep.name().to_string()],
                            direction: DiagramSocketDirection::Output,
                            max_connections: match ep.arity() {
                                ProviderArity::Many => None,
                                ProviderArity::One => Some(1),
                            },
                            is_required: Some(false),
                            node_side: DiagramSocketNodeSide::Right,
                        });
                    }

                    let socket_value = serde_json::to_value(sockets)?;

                    entry.insert(socket_value.to_owned());

                    socket_value
                }
                hash_map::Entry::Occupied(entry) => entry.get().to_owned(),
            };

            let schema = SchemaVariant::schema(ctx, schema_variant.id()).await?;

            let position = GridPoint {
                x: component.x().parse::<f64>()?.round() as isize,
                y: component.y().parse::<f64>()?.round() as isize,
            };
            let size = match (component.width(), component.height()) {
                (Some(h), Some(w)) => Size2D {
                    height: h.parse()?,
                    width: w.parse()?,
                },
                _ => Size2D {
                    height: 500,
                    width: 500,
                },
            };

            let updated_info = {
                let history_actor = ctx.history_actor();
                let actor = ActorView::from_history_actor(ctx, *history_actor).await?;
                serde_json::to_value(HistoryEventMetadata {
                    actor,
                    timestamp: component.timestamp().updated_at,
                })?
            };

            let created_info = {
                let history_actor = ctx.history_actor();
                let actor = ActorView::from_history_actor(ctx, *history_actor).await?;
                serde_json::to_value(HistoryEventMetadata {
                    actor,
                    timestamp: component.timestamp().created_at,
                })?
            };

            let component_view = SummaryDiagramComponent {
                id: component.id(),
                component_id: component.id(),
                schema_name: schema.name().to_owned(),
                schema_id: schema.id(),
                schema_variant_id: schema_variant.id(),
                schema_variant_name: schema_variant.name().to_owned(),
                schema_category: schema_variant.category().to_owned(),
                node_id: component.id(),
                display_name: component.name(ctx).await?,
                position,
                size,
                node_type: component.get_type(ctx).await?.to_string(),
                color: component.color(ctx).await?.unwrap_or("#111111".into()),
                change_status: ChangeStatus::Added.to_string(),
                has_resource: false,
                sockets,
                parent_node_id: component.parent(ctx).await?,
                child_node_ids: serde_json::to_value::<Vec<String>>(vec![])?,
                updated_info,
                created_info,
                deleted_info: serde_json::Value::Null,
            };

            component_views.push(component_view);
        }

        // TODO(nick): restore the ability to show edges.
        Ok(Self {
            edges: diagram_edges,
            components: component_views,
        })
    }
}
