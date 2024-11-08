pub mod geometry;
pub mod view;

use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use std::collections::HashMap;
use std::num::{ParseFloatError, ParseIntError};
use std::sync::Arc;
use telemetry::prelude::*;
use thiserror::Error;

use crate::attribute::prototype::argument::{
    AttributePrototypeArgumentError, AttributePrototypeArgumentId,
};
use crate::attribute::value::AttributeValueError;
use crate::change_status::ChangeStatus;
use crate::component::inferred_connection_graph::InferredConnectionGraphError;
use crate::component::{
    ComponentError, ComponentResult, IncomingConnection, InferredConnection, OutgoingConnection,
};
use crate::diagram::geometry::{Geometry, GeometryId};
use crate::diagram::view::{View, ViewId};
use crate::schema::variant::SchemaVariantError;
use crate::socket::input::InputSocketError;
use crate::socket::output::OutputSocketError;
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::NodeWeightError;
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    AttributePrototypeId, ChangeSetError, Component, ComponentId, DalContext, HelperError,
    HistoryEventError, InputSocketId, OutputSocketId, SchemaVariantId, StandardModelError,
    TransactionsError, Workspace, WorkspaceError, WorkspaceSnapshot,
};
use si_frontend_types::{DiagramComponentView, DiagramSocket};
use si_layer_cache::LayerDbError;

#[remain::sorted]
#[derive(Error, Debug)]
pub enum DiagramError {
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("attribute prototype not found")]
    AttributePrototypeNotFound,
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] AttributeValueError),
    #[error("attribute value not found")]
    AttributeValueNotFound,
    #[error("Change Set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("component error: {0}")]
    Component(#[from] ComponentError),
    #[error("component not found")]
    ComponentNotFound,
    #[error("component not found for geometry: {0}")]
    ComponentNotFoundForGeometry(GeometryId),
    #[error("component status not found for component: {0}")]
    ComponentStatusNotFound(ComponentId),
    #[error("default view not found")]
    DefaultViewNotFound,
    #[error("trying to delete only geometry (the one on view {0}) for component {1}")]
    DeletingLastGeometryForComponent(ViewId, ComponentId),
    #[error("deletion timestamp not found")]
    DeletionTimeStamp,
    #[error("destination attribute prototype not found for inter component attribute prototype argument: {0}")]
    DestinationAttributePrototypeNotFound(AttributePrototypeArgumentId),
    #[error("destination input socket not found for attribute prototype ({0}) and inter component attribute prototype argument ({1})")]
    DestinationInputSocketNotFound(AttributePrototypeId, AttributePrototypeArgumentId),
    #[error("edge not found")]
    EdgeNotFound,
    #[error("geometry not found: {0}")]
    GeometryNotFound(GeometryId),
    #[error("geometry not found for component {0} on view {1}")]
    GeometryNotFoundForComponentAndView(ComponentId, ViewId),
    #[error("Helper error: {0}")]
    Helper(#[from] HelperError),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("InferredConnectionGraph error: {0}")]
    InferredConnectionGraph(#[from] InferredConnectionGraphError),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("layerdb error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("node not found")]
    NodeNotFound,
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
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
    #[error("Transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("view category node not found")]
    ViewCategoryNotFound,
    #[error("view not found: {0}")]
    ViewNotFound(ViewId),
    #[error("view not found for geometry id: {0}")]
    ViewNotFoundForGeometry(GeometryId),
    #[error("Workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
}

pub type DiagramResult<T> = Result<T, DiagramError>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SummaryDiagramEdge {
    pub from_component_id: ComponentId,
    pub from_socket_id: OutputSocketId,
    pub to_component_id: ComponentId,
    pub to_socket_id: InputSocketId,
    pub change_status: ChangeStatus,
    pub created_info: serde_json::Value,
    pub deleted_info: serde_json::Value,
    pub to_delete: bool,
    pub from_base_change_set: bool,
}

impl SummaryDiagramEdge {
    pub fn assemble_just_added(incoming_connection: IncomingConnection) -> ComponentResult<Self> {
        Ok(SummaryDiagramEdge {
            from_component_id: incoming_connection.from_component_id,
            from_socket_id: incoming_connection.from_output_socket_id,
            to_component_id: incoming_connection.to_component_id,
            to_socket_id: incoming_connection.to_input_socket_id,
            change_status: ChangeStatus::Added,
            created_info: serde_json::to_value(incoming_connection.created_info)?,
            deleted_info: serde_json::to_value(incoming_connection.deleted_info)?,
            to_delete: false,
            from_base_change_set: false,
        })
    }

    pub fn assemble(
        incoming_connection: IncomingConnection,
        from_component: &Component,
        to_component: &Component,
        change_status: ChangeStatus,
    ) -> ComponentResult<Self> {
        Ok(SummaryDiagramEdge {
            from_component_id: incoming_connection.from_component_id,
            from_socket_id: incoming_connection.from_output_socket_id,
            to_component_id: incoming_connection.to_component_id,
            to_socket_id: incoming_connection.to_input_socket_id,
            change_status,
            created_info: serde_json::to_value(incoming_connection.created_info)?,
            deleted_info: serde_json::to_value(incoming_connection.deleted_info)?,
            to_delete: from_component.to_delete() || to_component.to_delete(),
            from_base_change_set: false,
        })
    }

    pub fn assemble_outgoing(
        outgoing_connection: OutgoingConnection,
        from_component: &Component,
        to_component: &Component,
        change_status: ChangeStatus,
    ) -> ComponentResult<Self> {
        Ok(SummaryDiagramEdge {
            from_component_id: outgoing_connection.from_component_id,
            from_socket_id: outgoing_connection.from_output_socket_id,
            to_component_id: outgoing_connection.to_component_id,
            to_socket_id: outgoing_connection.to_input_socket_id,
            change_status,
            created_info: serde_json::to_value(outgoing_connection.created_info)?,
            deleted_info: serde_json::to_value(outgoing_connection.deleted_info)?,
            to_delete: from_component.to_delete() || to_component.to_delete(),
            from_base_change_set: false,
        })
    }
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Hash)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SummaryDiagramInferredEdge {
    pub from_component_id: ComponentId,
    pub from_socket_id: OutputSocketId,
    pub to_component_id: ComponentId,
    pub to_socket_id: InputSocketId,
    // this is inferred by if either the to or from component is marked to_delete
    pub to_delete: bool,
}

impl SummaryDiagramInferredEdge {
    pub fn assemble(inferred_incoming_connection: InferredConnection) -> DiagramResult<Self> {
        Ok(SummaryDiagramInferredEdge {
            from_component_id: inferred_incoming_connection.from_component_id,
            from_socket_id: inferred_incoming_connection.from_output_socket_id,
            to_component_id: inferred_incoming_connection.to_component_id,
            to_socket_id: inferred_incoming_connection.to_input_socket_id,
            to_delete: inferred_incoming_connection.to_delete,
        })
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Diagram {
    pub components: Vec<DiagramComponentView>,
    pub edges: Vec<SummaryDiagramEdge>,
    pub inferred_edges: Vec<SummaryDiagramInferredEdge>,
}

impl Diagram {
    async fn assemble_component_views(
        ctx: &DalContext,
        base_snapshot: &Arc<WorkspaceSnapshot>,
        components: &HashMap<ComponentId, (Component, Geometry)>,
        diagram_sockets: &mut HashMap<SchemaVariantId, Vec<DiagramSocket>>,
    ) -> DiagramResult<(Vec<DiagramComponentView>, Vec<SummaryDiagramEdge>)> {
        let mut component_views = Vec::with_capacity(components.len());
        let mut diagram_edges = Vec::with_capacity(components.len());

        for (component, geometry) in components.values() {
            let change_status = component.change_status(ctx).await?;
            component_views.push(
                component
                    .into_frontend_type(ctx, Some(geometry), change_status, diagram_sockets)
                    .await?,
            );

            for incoming_connection in component.incoming_connections(ctx).await? {
                if let Some((from_component, _)) =
                    components.get(&incoming_connection.from_component_id)
                {
                    let edge_status = if base_snapshot
                        .get_node_index_by_id_opt(
                            incoming_connection.attribute_prototype_argument_id,
                        )
                        .await
                        .is_none()
                    {
                        ChangeStatus::Added
                    } else {
                        ChangeStatus::Unmodified
                    };

                    diagram_edges.push(SummaryDiagramEdge::assemble(
                        incoming_connection,
                        from_component,
                        component,
                        edge_status,
                    )?);
                }
            }
        }

        Ok((component_views, diagram_edges))
    }

    async fn assemble_inferred_connection_views(
        ctx: &DalContext,
        components: &HashMap<ComponentId, (Component, Geometry)>,
    ) -> DiagramResult<Vec<SummaryDiagramInferredEdge>> {
        let mut diagram_inferred_edges = vec![];

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut component_tree = workspace_snapshot.inferred_connection_graph(ctx).await?;

        for incoming_connection in component_tree
            .inferred_connections_for_all_components(ctx)
            .await?
        {
            let to_delete = if let (Some((source_component, _)), Some((destination_component, _))) = (
                components.get(&incoming_connection.source_component_id),
                components.get(&incoming_connection.destination_component_id),
            ) {
                source_component.to_delete() || destination_component.to_delete()
            } else {
                false
            };

            diagram_inferred_edges.push(SummaryDiagramInferredEdge {
                from_component_id: incoming_connection.source_component_id,
                from_socket_id: incoming_connection.output_socket_id,
                to_component_id: incoming_connection.destination_component_id,
                to_socket_id: incoming_connection.input_socket_id,
                to_delete,
            });
        }

        Ok(diagram_inferred_edges)
    }

    async fn get_base_snapshot(ctx: &DalContext) -> DiagramResult<(Arc<WorkspaceSnapshot>, bool)> {
        let base_change_set_id = if let Some(change_set_id) = ctx.change_set()?.base_change_set_id {
            change_set_id
        } else {
            return Ok((ctx.workspace_snapshot()?.clone(), false));
        };

        let workspace = Workspace::get_by_pk_or_error(
            ctx,
            ctx.tenancy()
                .workspace_pk_opt()
                .ok_or(WorkspaceSnapshotError::WorkspaceMissing)?,
        )
        .await?;

        if workspace.default_change_set_id() == ctx.change_set_id() {
            return Ok((ctx.workspace_snapshot()?.clone(), false));
        }

        Ok((
            Arc::new(WorkspaceSnapshot::find_for_change_set(ctx, base_change_set_id).await?),
            true,
        ))
    }

    async fn assemble_removed_components(
        ctx: &DalContext,
        base_snapshot: Arc<WorkspaceSnapshot>,
        components: &HashMap<ComponentId, (Component, Geometry)>,
        view_id: ViewId,
        diagram_sockets: &mut HashMap<SchemaVariantId, Vec<DiagramSocket>>,
    ) -> DiagramResult<Vec<DiagramComponentView>> {
        let mut removed_component_summaries = vec![];

        let base_change_set_ctx = ctx.clone_with_base().await?;
        let base_change_set_ctx = &base_change_set_ctx;

        if let Some(components_cat_id) = base_snapshot
            .get_category_node(None, CategoryNodeKind::Component)
            .await?
        {
            for component_id in base_snapshot
                .all_outgoing_targets(components_cat_id)
                .await?
                .iter()
                .map(|weight| weight.id())
            {
                let component_id: ComponentId = component_id.into();
                if !components.contains_key(&component_id) {
                    let deleted_component =
                        Component::get_by_id(base_change_set_ctx, component_id).await?;

                    let Some(geometry) = Geometry::try_get_by_component_and_view(
                        base_change_set_ctx,
                        component_id,
                        view_id,
                    )
                    .await?
                    else {
                        continue;
                    };

                    let mut summary_diagram_component = deleted_component
                        .into_frontend_type(
                            base_change_set_ctx,
                            Some(&geometry),
                            ChangeStatus::Deleted,
                            diagram_sockets,
                        )
                        .await?;
                    summary_diagram_component.from_base_change_set = true;

                    removed_component_summaries.push(summary_diagram_component);
                }
            }
        }

        Ok(removed_component_summaries)
    }

    async fn assemble_removed_edges(ctx: &DalContext) -> DiagramResult<Vec<SummaryDiagramEdge>> {
        let removed_incoming_connections: Vec<IncomingConnection> = ctx
            .workspace_snapshot()?
            .socket_edges_removed_relative_to_base(ctx)
            .await?;
        let mut diagram_edges = Vec::with_capacity(removed_incoming_connections.len());
        for removed_incoming_connection in &removed_incoming_connections {
            diagram_edges.push(SummaryDiagramEdge {
                from_component_id: removed_incoming_connection.from_component_id,
                from_socket_id: removed_incoming_connection.from_output_socket_id,
                to_component_id: removed_incoming_connection.to_component_id,
                to_socket_id: removed_incoming_connection.to_input_socket_id,
                change_status: ChangeStatus::Deleted,
                created_info: serde_json::to_value(&removed_incoming_connection.created_info)?,
                deleted_info: serde_json::to_value(&removed_incoming_connection.deleted_info)?,
                to_delete: true,
                from_base_change_set: true,
            });
        }

        Ok(diagram_edges)
    }

    /// Assemble a [`Diagram`](Self) based on existing [`Nodes`](crate::Node) and
    /// [`Connections`](crate::Connection) for a [View](view::View).
    #[instrument(level = "info", skip(ctx))]
    pub async fn assemble(ctx: &DalContext, view_id: ViewId) -> DiagramResult<Self> {
        let component_and_geometry_by_component_id = {
            let mut map = HashMap::new();

            for geometry in Geometry::list_by_view_id(ctx, view_id).await? {
                let component_id = match Geometry::component_id(ctx, geometry.id()).await {
                    Ok(id) => id,
                    Err(DiagramError::ComponentNotFoundForGeometry(geo_id)) => {
                        let changeset_id = ctx.change_set_id();
                        // NOTE(victor): The first version of views didn't delete geometries with components,
                        // so we have dangling geometries in some workspaces. We should clean this up at some point,
                        // but we just skip orphan geometries here to make assemble work.
                        warn!(
                            si.change_set.id = %changeset_id,
                            si.geometry.id = %geo_id,
                            "Could not find component for geometry - skipping"
                        );

                        continue;
                    }
                    Err(e) => return Err(e),
                };

                let component = Component::get_by_id(ctx, component_id).await?;

                map.insert(component_id, (component, geometry));
            }

            map
        };

        let (base_snapshot, not_on_head) = Self::get_base_snapshot(ctx).await?;
        let mut diagram_sockets = HashMap::new();
        let (mut component_views, mut diagram_edges) = Self::assemble_component_views(
            ctx,
            &base_snapshot,
            &component_and_geometry_by_component_id,
            &mut diagram_sockets,
        )
        .await?;

        let diagram_inferred_edges =
            Self::assemble_inferred_connection_views(ctx, &component_and_geometry_by_component_id)
                .await?;

        if not_on_head {
            let removed_component_summaries = Self::assemble_removed_components(
                ctx,
                base_snapshot,
                &component_and_geometry_by_component_id,
                view_id,
                &mut diagram_sockets,
            )
            .await?;
            component_views.extend(removed_component_summaries);

            let removed_edges = Self::assemble_removed_edges(ctx).await?;
            diagram_edges.extend(removed_edges);
        }

        Ok(Self {
            edges: diagram_edges,
            components: component_views,
            inferred_edges: diagram_inferred_edges,
        })
    }

    /// Calls [Self::assemble](Self::assemble) for the default view.
    pub async fn assemble_for_default_view(ctx: &DalContext) -> DiagramResult<Self> {
        let default_view_id = View::get_id_for_default(ctx).await?;

        Self::assemble(ctx, default_view_id).await
    }
}
