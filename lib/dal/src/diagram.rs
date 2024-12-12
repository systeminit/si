mod diagram_object;
pub mod geometry;
pub mod view;

use anyhow::Result;
use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use si_frontend_types::{DiagramComponentView, DiagramSocket};
use si_layer_cache::LayerDbError;
use std::{
    collections::{HashMap, HashSet},
    num::{ParseFloatError, ParseIntError},
    sync::Arc,
};
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    approval_requirement::ApprovalRequirementError,
    attribute::{
        prototype::argument::{AttributePrototypeArgumentError, AttributePrototypeArgumentId},
        value::AttributeValueError,
    },
    change_status::ChangeStatus,
    component::{
        inferred_connection_graph::InferredConnectionGraphError, ComponentError,
        IncomingConnection, InferredConnection, OutgoingConnection,
    },
    diagram::{
        geometry::{Geometry, GeometryId, GeometryRepresents},
        view::{View, ViewId, ViewObjectView},
    },
    schema::variant::SchemaVariantError,
    socket::{input::InputSocketError, output::OutputSocketError},
    workspace_snapshot::{
        node_weight::{category_node_weight::CategoryNodeKind, NodeWeight, NodeWeightError},
        WorkspaceSnapshotError,
    },
    AttributePrototypeId, ChangeSetError, Component, ComponentId, DalContext,
    EdgeWeightKindDiscriminants, FuncError, HelperError, HistoryEventError, InputSocketId,
    NodeWeightDiscriminants, OutputSocketId, SchemaId, SchemaVariant, SchemaVariantId,
    StandardModelError, TransactionsError, Workspace, WorkspaceError, WorkspaceSnapshot,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum DiagramError {
    #[error("approval requirement error: {0}")]
    ApprovalRequirement(#[from] ApprovalRequirementError),
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
    #[error("more then one diagram object found for view {0}")]
    DiagramObjectMoreThanOneForView(ViewId),
    #[error("diagram object not found for view {0}")]
    DiagramObjectNotFoundForView(ViewId),
    #[error("edge not found")]
    EdgeNotFound,
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("geometry can't represent: {0}")]
    GeometryCannotRepresentNodeWeight(NodeWeightDiscriminants),
    #[error("geometry not found: {0}")]
    GeometryNotFound(GeometryId),
    #[error("geometry not found for component {0} on view {1}")]
    GeometryNotFoundForComponentAndView(ComponentId, ViewId),
    #[error("geometry not found for view object {0} on view {1}")]
    GeometryNotFoundForViewObjectAndView(ViewId, ViewId),
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
    #[error("represented node not found for geometry: {0}")]
    RepresentedNotFoundForGeometry(GeometryId),
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

pub type DiagramResult<T> = Result<T>;

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
    pub fn assemble_just_added(incoming_connection: IncomingConnection) -> Result<Self> {
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
    ) -> Result<Self> {
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
    ) -> Result<Self> {
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SummaryDiagramManagementEdge {
    pub from_socket_id: String,
    pub to_socket_id: String,
    pub from_component_id: ComponentId,
    pub to_component_id: ComponentId,
    pub to_delete: bool,
    pub change_status: ChangeStatus,
    pub from_base_change_set: bool,
}

impl SummaryDiagramManagementEdge {
    pub fn new(
        from_schema_id: SchemaId,
        to_schema_id: SchemaId,
        from_component_id: ComponentId,
        to_component_id: ComponentId,
    ) -> Self {
        SummaryDiagramManagementEdge {
            from_socket_id: Self::output_socket_id(from_schema_id),
            to_socket_id: Self::input_socket_id(to_schema_id),
            from_component_id,
            to_component_id,
            to_delete: false,
            change_status: ChangeStatus::Added,
            from_base_change_set: false,
        }
    }

    pub fn new_removed(
        from_schema_id: SchemaId,
        to_schema_id: SchemaId,
        from_component_id: ComponentId,
        to_component_id: ComponentId,
        from_base_change_set: bool,
    ) -> Self {
        SummaryDiagramManagementEdge {
            from_socket_id: Self::output_socket_id(from_schema_id),
            to_socket_id: Self::input_socket_id(to_schema_id),
            from_component_id,
            to_component_id,
            to_delete: true,
            change_status: ChangeStatus::Deleted,
            from_base_change_set,
        }
    }

    pub fn output_socket_id(schema_id: SchemaId) -> String {
        format!("mgmt_output_{schema_id}")
    }

    pub fn input_socket_id(schema_id: SchemaId) -> String {
        format!("mgmt_input_{schema_id}")
    }
}

struct ComponentInfo {
    component: Component,
    geometry: Option<Geometry>,
    schema_id: SchemaId,
}

type ComponentInfoCache = HashMap<ComponentId, ComponentInfo>;

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
    pub management_edges: Vec<SummaryDiagramManagementEdge>,
    pub views: Vec<ViewObjectView>,
}

pub struct DiagramComponentViews {
    component_views: Vec<DiagramComponentView>,
    diagram_edges: Vec<SummaryDiagramEdge>,
    management_edges: Vec<SummaryDiagramManagementEdge>,
}

impl Diagram {
    #[instrument(level = "info", skip_all)]
    async fn assemble_component_views(
        ctx: &DalContext,
        base_snapshot: &Arc<WorkspaceSnapshot>,
        components: &ComponentInfoCache,
        diagram_sockets: &mut HashMap<SchemaVariantId, Vec<DiagramSocket>>,
    ) -> DiagramResult<DiagramComponentViews> {
        let mut component_views = Vec::with_capacity(components.len());
        let mut diagram_edges = Vec::with_capacity(components.len());
        let mut management_edges = Vec::with_capacity(components.len() / 2);

        for ComponentInfo {
            component,
            geometry,
            schema_id,
            ..
        } in components.values()
        {
            let change_status = component.change_status(ctx).await?;
            component_views.push(
                component
                    .into_frontend_type(ctx, geometry.as_ref(), change_status, diagram_sockets)
                    .await?,
            );

            let managers = component.managers(ctx).await?;
            for manager_id in managers {
                let Some(ComponentInfo {
                    component: from_component,
                    schema_id: from_schema_id,
                    ..
                }) = components.get(&manager_id)
                else {
                    continue;
                };

                let change_status = if base_snapshot
                    .find_edge(
                        manager_id,
                        component.id(),
                        EdgeWeightKindDiscriminants::Manages,
                    )
                    .await
                    .is_none()
                {
                    ChangeStatus::Added
                } else {
                    ChangeStatus::Unmodified
                };

                let mut management_edge = SummaryDiagramManagementEdge::new(
                    *from_schema_id,
                    *schema_id,
                    manager_id,
                    component.id(),
                );
                management_edge.to_delete = from_component.to_delete() || component.to_delete();
                management_edge.change_status = change_status;

                management_edges.push(management_edge);
            }

            for incoming_connection in component.incoming_connections(ctx).await? {
                if let Some(ComponentInfo {
                    component: from_component,
                    ..
                }) = components.get(&incoming_connection.from_component_id)
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

        Ok(DiagramComponentViews {
            component_views,
            diagram_edges,
            management_edges,
        })
    }

    #[instrument(level = "info", skip_all)]
    async fn assemble_inferred_connection_views(
        ctx: &DalContext,
        components: &ComponentInfoCache,
    ) -> DiagramResult<Vec<SummaryDiagramInferredEdge>> {
        let mut diagram_inferred_edges = vec![];

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut component_tree = workspace_snapshot.inferred_connection_graph(ctx).await?;

        for incoming_connection in component_tree
            .inferred_connections_for_all_components(ctx)
            .await?
        {
            let to_delete = components
                .get(&incoming_connection.source_component_id)
                .zip(components.get(&incoming_connection.destination_component_id))
                .is_some_and(
                    |(
                        ComponentInfo {
                            component: source_component,
                            ..
                        },
                        ComponentInfo {
                            component: destination_component,
                            ..
                        },
                    )| {
                        source_component.to_delete() || destination_component.to_delete()
                    },
                );

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

    #[instrument(level = "info", skip_all)]
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

    #[instrument(level = "info", skip_all)]
    async fn assemble_removed_components(
        ctx: &DalContext,
        base_snapshot: Arc<WorkspaceSnapshot>,
        maybe_view_id: Option<ViewId>,
        diagram_sockets: &mut HashMap<SchemaVariantId, Vec<DiagramSocket>>,
    ) -> DiagramResult<Vec<DiagramComponentView>> {
        let mut removed_component_summaries = vec![];
        let components = Component::list_ids(ctx).await?;

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
                if !components.contains(&component_id) {
                    let deleted_component =
                        Component::get_by_id(base_change_set_ctx, component_id).await?;

                    // If we get a view, try to get geometry, skip whole component if we don't find it
                    // If we don't get a view, don't skip, geometry is None
                    let maybe_geometry = if let Some(view_id) = maybe_view_id {
                        let Some(geometry) = Geometry::try_get_by_component_and_view(
                            base_change_set_ctx,
                            component_id,
                            view_id,
                        )
                        .await?
                        else {
                            continue;
                        };

                        Some(geometry)
                    } else {
                        None
                    };

                    let mut summary_diagram_component = deleted_component
                        .into_frontend_type(
                            base_change_set_ctx,
                            maybe_geometry.as_ref(),
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

    #[instrument(level = "info", skip_all)]
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

    /// If a manages edge is in the base snapshot, but not in the current
    /// snapshot, that means it has been deleted. If one of the components is
    /// not in this changeset, we can ignore the deleted edge, since we won't
    /// render it. If the components are restored from the base, the edge will
    /// *magically* reappear as deleted.
    #[instrument(level = "info", skip_all)]
    async fn assemble_removed_management_edges(
        ctx: &DalContext,
        base_snapshot: Arc<WorkspaceSnapshot>,
    ) -> DiagramResult<Vec<SummaryDiagramManagementEdge>> {
        let mut removed_edges = vec![];

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut existing_management_edges = HashSet::new();
        for component in Component::list(ctx).await? {
            if component.to_delete() {
                continue;
            }
            for source_idx in workspace_snapshot
                .incoming_sources_for_edge_weight_kind(
                    component.id(),
                    EdgeWeightKindDiscriminants::Manages,
                )
                .await?
            {
                let node_weight = workspace_snapshot.get_node_weight(source_idx).await?;
                if let NodeWeight::Component(_) = &node_weight {
                    existing_management_edges.insert((node_weight.id().into(), component.id()));
                }
            }
        }

        // list components
        let head_ctx = ctx.clone_with_head().await?;
        for from_id in Component::list_ids(&head_ctx).await? {
            let from_sv = Component::schema_variant_for_component_id(&head_ctx, from_id).await?;
            let from_schema_id =
                SchemaVariant::schema_id_for_schema_variant_id(&head_ctx, from_sv.id()).await?;
            let Some(from_idx) = base_snapshot.get_node_index_by_id_opt(from_id).await else {
                continue;
            };

            for to_idx in base_snapshot
                .edges_directed_by_index(from_idx, Direction::Outgoing)
                .await?
                .iter()
                .filter(|(edge_weight, _, _)| {
                    EdgeWeightKindDiscriminants::Manages == edge_weight.kind().into()
                })
                .map(|(_, _, to_idx)| *to_idx)
            {
                let Some(to_id) = base_snapshot
                    .get_node_weight_opt(to_idx)
                    .await
                    .map(|weight| weight.id().into())
                else {
                    continue;
                };
                let to_sv = Component::schema_variant_for_component_id(&head_ctx, to_id).await?;
                let to_schema_id =
                    SchemaVariant::schema_id_for_schema_variant_id(&head_ctx, to_sv.id()).await?;

                if existing_management_edges.contains(&(from_id, to_id)) {
                    continue;
                }

                removed_edges.push(SummaryDiagramManagementEdge::new_removed(
                    from_schema_id,
                    to_schema_id,
                    from_id,
                    to_id,
                    true,
                ));
            }
        }

        Ok(removed_edges)
    }

    /// Assemble a [`Diagram`](Self) based on existing [`Nodes`](crate::Node) and
    /// [`Connections`](crate::Connection).
    /// If passed a [ViewId], assemble it for that view only, otherwise, do it for the whole
    /// graph.
    #[instrument(level = "info", skip(ctx))]
    pub async fn assemble(ctx: &DalContext, maybe_view_id: Option<ViewId>) -> DiagramResult<Self> {
        let mut views = vec![];
        let component_info_cache = {
            let mut map = HashMap::new();

            if let Some(view_id) = maybe_view_id {
                for geometry in Geometry::list_by_view_id(ctx, view_id).await? {
                    let geo_represents = match Geometry::represented_id(ctx, geometry.id()).await {
                        Ok(r) => r,
                        Err(error) => match error.downcast_ref() {
                            Some(DiagramError::RepresentedNotFoundForGeometry(geo_id)) => {
                                let changeset_id = ctx.change_set_id();
                                // NOTE(victor): The first version of views didn't delete geometries with components,
                                // so we have dangling geometries in some workspaces. We should clean this up at some point,
                                // but we just skip orphan geometries here to make assemble work.

                                debug!(
                                    si.change_set.id = %changeset_id,
                                    si.geometry.id = %geo_id,
                                    "Could not find represented node for geometry - skipping"
                                );

                                continue;
                            }
                            _ => return Err(error),
                        },
                    };
                    match geo_represents {
                        GeometryRepresents::Component(component_id) => {
                            let component = Component::get_by_id(ctx, component_id).await?;
                            let schema_id = component.schema(ctx).await?.id();

                            map.insert(
                                component_id,
                                ComponentInfo {
                                    component,
                                    geometry: Some(geometry),
                                    schema_id,
                                },
                            );
                        }
                        GeometryRepresents::View(view_id) => {
                            let view = View::get_by_id(ctx, view_id).await?;
                            let view_object_view = ViewObjectView::from_view_and_geometry(
                                ctx,
                                view,
                                geometry.into_raw(),
                            )
                            .await?;
                            views.push(view_object_view);
                        }
                    }
                }
            } else {
                for component in Component::list(ctx).await? {
                    let schema_id = component.schema(ctx).await?.id();
                    map.insert(
                        component.id(),
                        ComponentInfo {
                            component,
                            geometry: None,
                            schema_id,
                        },
                    );
                }
            }
            map
        };

        let (base_snapshot, not_on_head) = Self::get_base_snapshot(ctx).await?;
        let mut diagram_sockets = HashMap::new();
        let mut diagram_component_views = Self::assemble_component_views(
            ctx,
            &base_snapshot,
            &component_info_cache,
            &mut diagram_sockets,
        )
        .await?;

        let diagram_inferred_edges =
            Self::assemble_inferred_connection_views(ctx, &component_info_cache).await?;

        if not_on_head {
            let removed_component_summaries = Self::assemble_removed_components(
                ctx,
                base_snapshot.clone(),
                maybe_view_id,
                &mut diagram_sockets,
            )
            .await?;
            diagram_component_views
                .component_views
                .extend(removed_component_summaries);

            let removed_edges = Self::assemble_removed_edges(ctx).await?;
            diagram_component_views.diagram_edges.extend(removed_edges);
            let removed_management_edges =
                Self::assemble_removed_management_edges(ctx, base_snapshot).await?;
            diagram_component_views
                .management_edges
                .extend(removed_management_edges);
        }

        Ok(Self {
            edges: diagram_component_views.diagram_edges,
            components: diagram_component_views.component_views,
            inferred_edges: diagram_inferred_edges,
            management_edges: diagram_component_views.management_edges,
            views,
        })
    }

    /// Calls [Self::assemble](Self::assemble) for the default view.
    pub async fn assemble_for_default_view(ctx: &DalContext) -> DiagramResult<Self> {
        let default_view_id = View::get_id_for_default(ctx).await?;

        Self::assemble(ctx, Some(default_view_id)).await
    }
}
