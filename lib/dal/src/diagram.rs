mod diagram_object;
pub mod geometry;
pub mod view;

use std::{
    collections::{
        HashMap,
        HashSet,
    },
    num::{
        ParseFloatError,
        ParseIntError,
    },
};

use petgraph::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};
use si_data_pg::PgError;
use si_frontend_types::{
    DiagramComponentView,
    DiagramSocket,
};
use si_id::{
    AttributePrototypeId,
    AttributeValueId,
    ComponentId,
    GeometryId,
    InputSocketId,
    OutputSocketId,
    SchemaId,
    SchemaVariantId,
    ViewId,
};
use si_layer_cache::LayerDbError;
use telemetry::prelude::*;
use thiserror::Error;

use crate::{
    AttributePrototype,
    AttributeValue,
    ChangeSetError,
    Component,
    DalContext,
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    FuncError,
    HelperError,
    NodeWeightDiscriminants,
    SchemaVariant,
    TransactionsError,
    Workspace,
    WorkspaceError,
    WorkspaceSnapshot,
    approval_requirement::ApprovalRequirementError,
    attribute::{
        path::AttributePath,
        prototype::{
            AttributePrototypeError,
            argument::{
                AttributePrototypeArgument,
                AttributePrototypeArgumentError,
                AttributePrototypeArgumentId,
            },
        },
        value::AttributeValueError,
    },
    change_status::ChangeStatus,
    component::{
        ComponentError,
        ComponentResult,
        Connection,
        InferredConnection,
        inferred_connection_graph::InferredConnectionGraphError,
    },
    diagram::{
        geometry::{
            Geometry,
            GeometryRepresents,
        },
        view::{
            View,
            ViewObjectView,
        },
    },
    schema::variant::SchemaVariantError,
    socket::{
        input::InputSocketError,
        output::OutputSocketError,
    },
    workspace_snapshot::{
        WorkspaceSnapshotError,
        WorkspaceSnapshotSelector,
        node_weight::{
            NodeWeight,
            NodeWeightError,
            category_node_weight::CategoryNodeKind,
        },
        selector::WorkspaceSnapshotSelectorDiscriminants,
        split_snapshot::SplitSnapshot,
    },
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum DiagramError {
    #[error("approval requirement error: {0}")]
    ApprovalRequirement(#[from] Box<ApprovalRequirementError>),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] Box<AttributePrototypeError>),
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] Box<AttributePrototypeArgumentError>),
    #[error("attribute prototype not found")]
    AttributePrototypeNotFound,
    #[error("attribute value error: {0}")]
    AttributeValue(#[from] Box<AttributeValueError>),
    #[error("attribute value not found")]
    AttributeValueNotFound,
    #[error("Change Set error: {0}")]
    ChangeSet(#[from] Box<ChangeSetError>),
    #[error("component error: {0}")]
    Component(#[from] Box<ComponentError>),
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
    #[error(
        "destination attribute prototype not found for inter component attribute prototype argument: {0}"
    )]
    DestinationAttributePrototypeNotFound(AttributePrototypeArgumentId),
    #[error(
        "destination input socket not found for attribute prototype ({0}) and inter component attribute prototype argument ({1})"
    )]
    DestinationInputSocketNotFound(AttributePrototypeId, AttributePrototypeArgumentId),
    #[error("more then one diagram object found for view {0}")]
    DiagramObjectMoreThanOneForView(ViewId),
    #[error("diagram object not found for view {0}")]
    DiagramObjectNotFoundForView(ViewId),
    #[error("edge not found")]
    EdgeNotFound,
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("geometry can't represent: {0}")]
    GeometryCannotRepresentNodeWeight(NodeWeightDiscriminants),
    #[error("geometry not found: {0}")]
    GeometryNotFound(GeometryId),
    #[error("geometry not found for component {0} on view {1}")]
    GeometryNotFoundForComponentAndView(ComponentId, ViewId),
    #[error("geometry not found for view object {0} on view {1}")]
    GeometryNotFoundForViewObjectAndView(ViewId, ViewId),
    #[error("Helper error: {0}")]
    Helper(#[from] Box<HelperError>),
    #[error("InferredConnectionGraph error: {0}")]
    InferredConnectionGraph(#[from] Box<InferredConnectionGraphError>),
    #[error("input socket error: {0}")]
    InputSocket(#[from] Box<InputSocketError>),
    #[error("layerdb error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("node not found")]
    NodeNotFound,
    #[error("node weight error: {0}")]
    NodeWeight(#[from] Box<NodeWeightError>),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] Box<OutputSocketError>),
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
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("schema variant not found")]
    SchemaVariantNotFound,
    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error("socket not found")]
    SocketNotFound,
    #[error("Transactions error: {0}")]
    Transactions(#[from] Box<TransactionsError>),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("view category node not found")]
    ViewCategoryNotFound,
    #[error("view not found: {0}")]
    ViewNotFound(ViewId),
    #[error("view not found for geometry id: {0}")]
    ViewNotFoundForGeometry(GeometryId),
    #[error("Workspace error: {0}")]
    Workspace(#[from] Box<WorkspaceError>),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] Box<WorkspaceSnapshotError>),
}

impl From<ApprovalRequirementError> for DiagramError {
    fn from(value: ApprovalRequirementError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributePrototypeError> for DiagramError {
    fn from(value: AttributePrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributePrototypeArgumentError> for DiagramError {
    fn from(value: AttributePrototypeArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributeValueError> for DiagramError {
    fn from(value: AttributeValueError) -> Self {
        Box::new(value).into()
    }
}

impl From<ChangeSetError> for DiagramError {
    fn from(value: ChangeSetError) -> Self {
        Box::new(value).into()
    }
}

impl From<ComponentError> for DiagramError {
    fn from(value: ComponentError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncError> for DiagramError {
    fn from(value: FuncError) -> Self {
        Box::new(value).into()
    }
}

impl From<HelperError> for DiagramError {
    fn from(value: HelperError) -> Self {
        Box::new(value).into()
    }
}

impl From<InferredConnectionGraphError> for DiagramError {
    fn from(value: InferredConnectionGraphError) -> Self {
        Box::new(value).into()
    }
}

impl From<InputSocketError> for DiagramError {
    fn from(value: InputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<NodeWeightError> for DiagramError {
    fn from(value: NodeWeightError) -> Self {
        Box::new(value).into()
    }
}

impl From<OutputSocketError> for DiagramError {
    fn from(value: OutputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<SchemaVariantError> for DiagramError {
    fn from(value: SchemaVariantError) -> Self {
        Box::new(value).into()
    }
}

impl From<TransactionsError> for DiagramError {
    fn from(value: TransactionsError) -> Self {
        Box::new(value).into()
    }
}

impl From<WorkspaceError> for DiagramError {
    fn from(value: WorkspaceError) -> Self {
        Box::new(value).into()
    }
}

impl From<WorkspaceSnapshotError> for DiagramError {
    fn from(value: WorkspaceSnapshotError) -> Self {
        Box::new(value).into()
    }
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
    pub fn assemble_just_added(incoming_connection: Connection) -> ComponentResult<Self> {
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
        incoming_connection: Connection,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all(serialize = "camelCase"))]
pub struct SummaryDiagramAttributeSubscriptionEdge {
    pub from_component_id: ComponentId,
    pub from_attribute_path: String,
    pub to_component_id: ComponentId,
    pub to_attribute_value_id: AttributeValueId,
    pub to_attribute_path: String,
    // this is inferred by if either the to or from component is marked to_delete
    pub to_delete: bool,
    pub change_status: ChangeStatus,
}

struct ComponentInfo {
    component: Component,
    geometry: Option<Geometry>,
    schema_id: SchemaId,
}

type ComponentInfoCache = HashMap<ComponentId, ComponentInfo>;

impl SummaryDiagramInferredEdge {
    pub fn assemble(inferred_incoming_connection: InferredConnection) -> Self {
        SummaryDiagramInferredEdge {
            from_component_id: inferred_incoming_connection.from_component_id,
            from_socket_id: inferred_incoming_connection.from_output_socket_id,
            to_component_id: inferred_incoming_connection.to_component_id,
            to_socket_id: inferred_incoming_connection.to_input_socket_id,
            to_delete: inferred_incoming_connection.to_delete,
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Diagram {
    pub components: Vec<DiagramComponentView>,
    pub edges: Vec<SummaryDiagramEdge>,
    pub inferred_edges: Vec<SummaryDiagramInferredEdge>,
    pub management_edges: Vec<SummaryDiagramManagementEdge>,
    pub attribute_subscription_edges: Vec<SummaryDiagramAttributeSubscriptionEdge>,
    pub views: Vec<ViewObjectView>,
}

pub struct DiagramComponentViews {
    pub component_views: Vec<DiagramComponentView>,
    pub diagram_edges: Vec<SummaryDiagramEdge>,
    pub management_edges: Vec<SummaryDiagramManagementEdge>,
    pub attribute_subscription_edges: Vec<SummaryDiagramAttributeSubscriptionEdge>,
}

impl Diagram {
    #[instrument(level = "info", skip_all)]
    async fn assemble_component_views(
        ctx: &DalContext,
        base_snapshot: &WorkspaceSnapshotSelector,
        components: &ComponentInfoCache,
        diagram_sockets: &mut HashMap<SchemaVariantId, Vec<DiagramSocket>>,
    ) -> DiagramResult<DiagramComponentViews> {
        let mut component_views = Vec::with_capacity(components.len());
        let mut diagram_edges = Vec::with_capacity(components.len());
        let mut management_edges = Vec::with_capacity(components.len() / 2);
        let mut attribute_subscription_edges = Vec::with_capacity(components.len());

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
                    let edge_status = if !base_snapshot
                        .node_exists(incoming_connection.attribute_prototype_argument_id)
                        .await
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

            let from_root_av_id = Component::root_attribute_value_id(ctx, component.id()).await?;
            let base_subscribers = Self::get_subscribers(base_snapshot, from_root_av_id).await;
            for subscriber in
                Self::get_subscribers(&ctx.workspace_snapshot()?, from_root_av_id).await
            {
                let (_, to_apa_id) = subscriber;
                let to_ap_id = AttributePrototypeArgument::prototype_id(ctx, to_apa_id).await?;
                if let Some(to_attribute_value_id) =
                    AttributePrototype::attribute_value_id(ctx, to_ap_id).await?
                {
                    let (root_id, to_attribute_path) =
                        AttributeValue::path_from_root(ctx, to_attribute_value_id).await?;
                    let to_component_id = AttributeValue::component_id(ctx, root_id).await?;
                    if let Some(ComponentInfo {
                        component: to_component,
                        ..
                    }) = components.get(&to_component_id)
                    {
                        let change_status = if base_subscribers.contains(&subscriber) {
                            ChangeStatus::Unmodified
                        } else {
                            ChangeStatus::Added
                        };
                        attribute_subscription_edges.push(
                            SummaryDiagramAttributeSubscriptionEdge {
                                from_component_id: component.id(),
                                from_attribute_path: subscriber.0,
                                to_component_id,
                                to_attribute_value_id,
                                to_attribute_path,
                                change_status,
                                to_delete: component.to_delete() || to_component.to_delete(),
                            },
                        );
                    }
                }
            }
        }

        Ok(DiagramComponentViews {
            component_views,
            diagram_edges,
            management_edges,
            attribute_subscription_edges,
        })
    }

    async fn get_subscribers(
        snapshot: &WorkspaceSnapshotSelector,
        subscribed_to_av_id: AttributeValueId,
    ) -> HashSet<(String, AttributePrototypeArgumentId)> {
        let Ok(edges) = snapshot
            .edges_directed(subscribed_to_av_id, Direction::Incoming)
            .await
        else {
            return HashSet::new();
        };
        edges
            .into_iter()
            .filter_map(|(edge, source_ulid, _)| match edge.kind {
                EdgeWeightKind::ValueSubscription(path) => match path {
                    AttributePath::JsonPointer(path) => Some((path, source_ulid.into())),
                },
                _ => None,
            })
            .collect()
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
    async fn get_base_snapshot(
        ctx: &DalContext,
    ) -> DiagramResult<(WorkspaceSnapshotSelector, bool)> {
        let base_change_set_id = if let Some(change_set_id) = ctx.change_set()?.base_change_set_id {
            change_set_id
        } else {
            return Ok((ctx.workspace_snapshot()?.clone(), false));
        };

        let workspace = Workspace::get_by_pk(
            ctx,
            ctx.tenancy()
                .workspace_pk_opt()
                .ok_or(WorkspaceSnapshotError::WorkspaceMissing)?,
        )
        .await?;

        if workspace.default_change_set_id() == ctx.change_set_id() {
            return Ok((ctx.workspace_snapshot()?.clone(), false));
        }

        match workspace.snapshot_kind() {
            WorkspaceSnapshotSelectorDiscriminants::SplitSnapshot => {
                let split_snapshot =
                    SplitSnapshot::find_for_change_set(ctx, base_change_set_id).await?;
                Ok((
                    WorkspaceSnapshotSelector::SplitSnapshot(split_snapshot.into()),
                    true,
                ))
            }
            WorkspaceSnapshotSelectorDiscriminants::LegacySnapshot => {
                let legacy_snapshot =
                    WorkspaceSnapshot::find_for_change_set(ctx, base_change_set_id).await?;
                Ok((
                    WorkspaceSnapshotSelector::LegacySnapshot(legacy_snapshot.into()),
                    true,
                ))
            }
        }
    }

    #[instrument(level = "info", skip_all)]
    async fn assemble_removed_components(
        ctx: &DalContext,
        base_snapshot: WorkspaceSnapshotSelector,
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

    #[instrument(
        name = "diagram.socket_edges_removed_relative_to_base",
        level = "debug",
        skip_all
    )]
    async fn socket_edges_removed_relative_to_base(
        ctx: &DalContext,
    ) -> DiagramResult<Vec<Connection>> {
        // Even though the default change set for a workspace can have a base change set, we don't
        // want to consider anything as new/modified/removed when looking at the default change
        // set.
        let workspace = Workspace::get_by_pk(ctx, ctx.tenancy().workspace_pk()?).await?;
        if workspace.default_change_set_id() == ctx.change_set_id() {
            return Ok(Vec::new());
        }

        let base_change_set_ctx = ctx.clone_with_base().await?;
        let base_change_set_ctx = &base_change_set_ctx;

        let base_components = Component::list(base_change_set_ctx).await?;

        #[derive(Hash, Clone, PartialEq, Eq)]
        struct UniqueEdge {
            to_component_id: ComponentId,
            from_component_id: ComponentId,
            from_socket_id: OutputSocketId,
            to_socket_id: InputSocketId,
        }
        let mut base_incoming_edges = HashSet::new();
        let mut base_incoming = HashMap::new();
        for base_component in base_components {
            let incoming_edges = base_component
                .incoming_connections(base_change_set_ctx)
                .await?;

            for conn in incoming_edges {
                let hash = UniqueEdge {
                    to_component_id: conn.to_component_id,
                    from_socket_id: conn.from_output_socket_id,
                    from_component_id: conn.from_component_id,
                    to_socket_id: conn.to_input_socket_id,
                };
                base_incoming_edges.insert(hash.clone());
                base_incoming.insert(hash, conn);
            }
        }

        let current_components = Component::list(ctx).await?;
        let mut current_incoming_edges = HashSet::new();
        for current_component in current_components {
            let incoming_edges: Vec<UniqueEdge> = current_component
                .incoming_connections(ctx)
                .await?
                .into_iter()
                .map(|conn| UniqueEdge {
                    to_component_id: conn.to_component_id,
                    from_socket_id: conn.from_output_socket_id,
                    from_component_id: conn.from_component_id,
                    to_socket_id: conn.to_input_socket_id,
                })
                .collect();
            current_incoming_edges.extend(incoming_edges);
        }

        let difference = base_incoming_edges.difference(&current_incoming_edges);
        let mut differences = vec![];
        for diff in difference {
            if let Some(edge) = base_incoming.get(diff) {
                differences.push(edge.clone());
            }
        }

        Ok(differences)
    }

    #[instrument(level = "info", skip_all)]
    async fn assemble_removed_edges(ctx: &DalContext) -> DiagramResult<Vec<SummaryDiagramEdge>> {
        let removed_incoming_connections: Vec<Connection> =
            Self::socket_edges_removed_relative_to_base(ctx).await?;
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
        base_snapshot: &WorkspaceSnapshotSelector,
    ) -> DiagramResult<Vec<SummaryDiagramManagementEdge>> {
        let mut removed_edges = vec![];

        let workspace_snapshot = ctx.workspace_snapshot()?;
        let mut existing_management_edges = HashSet::new();
        for component in Component::list(ctx).await? {
            if component.to_delete() {
                continue;
            }
            for source_id in workspace_snapshot
                .incoming_sources_for_edge_weight_kind(
                    component.id(),
                    EdgeWeightKindDiscriminants::Manages,
                )
                .await?
            {
                let node_weight = workspace_snapshot.get_node_weight(source_id).await?;
                if let NodeWeight::Component(_) = &node_weight {
                    existing_management_edges.insert((node_weight.id().into(), component.id()));
                }
            }
        }

        // list components
        let head_ctx = ctx.clone_with_head().await?;
        for from_id in Component::list_ids(&head_ctx).await? {
            let from_sv = Component::schema_variant_for_component_id(&head_ctx, from_id).await?;
            let from_schema_id = SchemaVariant::schema_id(&head_ctx, from_sv.id()).await?;

            for (_, _, to_id) in base_snapshot
                .edges_directed_for_edge_weight_kind(
                    from_id,
                    Direction::Outgoing,
                    EdgeWeightKindDiscriminants::Manages,
                )
                .await?
            {
                let to_sv =
                    Component::schema_variant_for_component_id(&head_ctx, to_id.into()).await?;
                let to_schema_id = SchemaVariant::schema_id(&head_ctx, to_sv.id()).await?;

                if existing_management_edges.contains(&(from_id, to_id.into())) {
                    continue;
                }

                removed_edges.push(SummaryDiagramManagementEdge::new_removed(
                    from_schema_id,
                    to_schema_id,
                    from_id,
                    to_id.into(),
                    true,
                ));
            }
        }

        Ok(removed_edges)
    }

    /// If a subscription is in the base snapshot, but not in the current
    /// snapshot, that means it has been deleted. If one of the components is
    /// not in this changeset, we can ignore the deleted edge, since we won't
    /// render it. If the components are restored from the base, the edge will
    /// *magically* reappear as deleted.
    #[instrument(level = "info", skip_all)]
    async fn assemble_removed_attribute_subscription_edges(
        _ctx: &DalContext,
        _base_snapshot: &WorkspaceSnapshotSelector,
    ) -> DiagramResult<Vec<SummaryDiagramAttributeSubscriptionEdge>> {
        // TODO implement this
        Ok(vec![])
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
                    let geo_represents = match Geometry::represented_id(ctx, geometry.id()).await? {
                        Some(r) => r,
                        None => continue,
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
                Self::assemble_removed_management_edges(ctx, &base_snapshot).await?;
            diagram_component_views
                .management_edges
                .extend(removed_management_edges);
            let removed_attribute_subscription_edges =
                Self::assemble_removed_attribute_subscription_edges(ctx, &base_snapshot).await?;
            diagram_component_views
                .attribute_subscription_edges
                .extend(removed_attribute_subscription_edges);
        }

        Ok(Self {
            edges: diagram_component_views.diagram_edges,
            components: diagram_component_views.component_views,
            inferred_edges: diagram_inferred_edges,
            management_edges: diagram_component_views.management_edges,
            attribute_subscription_edges: diagram_component_views.attribute_subscription_edges,
            views,
        })
    }

    /// Calls [Self::assemble](Self::assemble) for the default view.
    pub async fn assemble_for_default_view(ctx: &DalContext) -> DiagramResult<Self> {
        let default_view_id = View::get_id_for_default(ctx).await?;

        Self::assemble(ctx, Some(default_view_id)).await
    }
}
