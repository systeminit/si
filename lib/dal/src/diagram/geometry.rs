use std::{
    collections::HashMap,
    sync::Arc,
};

use jwt_simple::prelude::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    Timestamp,
    ulid::Ulid,
};
pub use si_frontend_types::RawGeometry;

use crate::{
    ComponentId,
    DalContext,
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    WorkspaceSnapshotError,
    diagram::{
        DiagramError,
        DiagramResult,
        diagram_object::DiagramObject,
        view::{
            View,
            ViewId,
        },
    },
    implement_add_edge_to,
    layer_db_types::{
        GeometryContent,
        GeometryContentV1,
    },
    workspace_snapshot::node_weight::{
        NodeWeight,
        diagram_object_node_weight::DiagramObjectKind,
        geometry_node_weight::GeometryNodeWeight,
        traits::SiVersionedNodeWeight,
    },
};

const DEFAULT_COMPONENT_X_POSITION: &str = "0";
const DEFAULT_COMPONENT_Y_POSITION: &str = "0";
const DEFAULT_COMPONENT_WIDTH: &str = "500";
const DEFAULT_COMPONENT_HEIGHT: &str = "500";

impl From<Geometry> for RawGeometry {
    fn from(value: Geometry) -> Self {
        Self {
            x: value.x,
            y: value.y,
            width: value.width,
            height: value.height,
        }
    }
}

pub use si_id::GeometryId;

pub enum GeometryRepresents {
    Component(ComponentId),
    View(ViewId),
}

/// Represents spatial data for something to be shown on a view
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq, Copy)]
pub struct Geometry {
    id: GeometryId,
    #[serde(flatten)]
    timestamp: Timestamp,
    x: isize,
    y: isize,
    width: Option<isize>,
    height: Option<isize>,
}

impl Geometry {
    implement_add_edge_to!(
        source_id: GeometryId,
        destination_id: ComponentId,
        add_fn: add_edge_to_component,
        discriminant: EdgeWeightKindDiscriminants::Represents,
        result: DiagramResult,
    );

    implement_add_edge_to!(
        source_id: GeometryId,
        destination_id: Ulid,
        add_fn: add_edge_to_diagram_object,
        discriminant: EdgeWeightKindDiscriminants::Represents,
        result: DiagramResult,
    );

    pub fn into_raw(self) -> RawGeometry {
        self.into()
    }

    pub fn id(&self) -> GeometryId {
        self.id
    }

    pub fn x(&self) -> isize {
        self.x
    }

    pub fn y(&self) -> isize {
        self.y
    }

    pub fn width(&self) -> Option<isize> {
        self.width
    }

    pub fn height(&self) -> Option<isize> {
        self.height
    }

    fn assemble(node_weight: GeometryNodeWeight, content: GeometryContent) -> Self {
        let content = content.extract();

        Self {
            id: node_weight.id().into(),
            timestamp: content.timestamp,
            x: content.x.parse().unwrap_or(0),
            y: content.y.parse().unwrap_or(0),
            width: content.width.map(|w| w.parse().unwrap_or(500)),
            height: content.height.map(|h| h.parse().unwrap_or(500)),
        }
    }

    pub async fn new_for_component(
        ctx: &DalContext,
        component_id: ComponentId,
        view_id: ViewId,
    ) -> DiagramResult<Self> {
        let (node_weight, content) = Self::new_inner(ctx, view_id).await?;

        Self::add_edge_to_component(
            ctx,
            node_weight.id().into(),
            component_id,
            EdgeWeightKind::Represents,
        )
        .await?;

        Ok(Self::assemble(
            node_weight.get_geometry_node_weight()?,
            content,
        ))
    }

    async fn new_inner(
        ctx: &DalContext,
        container_view_id: ViewId,
    ) -> DiagramResult<(NodeWeight, GeometryContent)> {
        let snap = ctx.workspace_snapshot()?;
        let id = snap.generate_ulid().await?;
        let lineage_id = snap.generate_ulid().await?;

        let content = GeometryContent::V1(GeometryContentV1 {
            timestamp: Timestamp::now(),
            x: DEFAULT_COMPONENT_X_POSITION.to_string(),
            y: DEFAULT_COMPONENT_Y_POSITION.to_string(),
            width: Some(DEFAULT_COMPONENT_WIDTH.to_string()),
            height: Some(DEFAULT_COMPONENT_HEIGHT.to_string()),
        });

        let (content_address, _) = ctx.layer_db().cas().write(
            Arc::new(content.clone().into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let node_weight = NodeWeight::new_geometry(id, lineage_id, content_address);
        snap.add_or_replace_node(node_weight.clone()).await?;

        View::add_geometry_by_id(ctx, container_view_id, id.into()).await?;

        Ok((node_weight, content))
    }

    /// Returns a [`GeometryRepresents`] for a given [`GeometryId`](Geometry). Returns `None` if we
    /// are dealing with dangling geometries from older iterations of [`Views`](View).
    ///
    /// _Note_: some change sets have orphan geometries because of an old bug, so when calling this
    /// function, be careful when dealing with the `ComponentNotFoundForGeometry` error.
    pub async fn represented_id(
        ctx: &DalContext,
        geometry_id: GeometryId,
    ) -> DiagramResult<Option<GeometryRepresents>> {
        let snap = ctx.workspace_snapshot()?;

        let component_id = match snap
            .outgoing_targets_for_edge_weight_kind(
                geometry_id,
                EdgeWeightKindDiscriminants::Represents,
            )
            .await?
            .pop()
        {
            Some(component_id) => component_id,
            None => {
                // NOTE(nick): I moved Victor's comment here and made the return type optional. I
                // updated the function doc comment too. Every single caller passed on this error
                // and logged it solely at the debug level.
                //
                // NOTE(victor): The first version of views didn't delete geometries with components,
                // so we have dangling geometries in some workspaces. We should clean this up at some point,
                // but we just skip orphan geometries here to make assemble work.
                return Ok(None);
            }
        };

        let node_weight = snap.get_node_weight(component_id).await?;

        let geo_represents = match node_weight {
            NodeWeight::Action(_)
            | NodeWeight::ActionPrototype(_)
            | NodeWeight::AttributePrototypeArgument(_)
            | NodeWeight::AttributeValue(_)
            | NodeWeight::Category(_)
            | NodeWeight::Content(_)
            | NodeWeight::DependentValueRoot(_)
            | NodeWeight::Func(_)
            | NodeWeight::FuncArgument(_)
            | NodeWeight::Ordering(_)
            | NodeWeight::Prop(_)
            | NodeWeight::Secret(_)
            | NodeWeight::FinishedDependentValueRoot(_)
            | NodeWeight::InputSocket(_)
            | NodeWeight::SchemaVariant(_)
            | NodeWeight::ManagementPrototype(_)
            | NodeWeight::Geometry(_)
            | NodeWeight::View(_)
            | NodeWeight::Reason(_)
            | NodeWeight::ApprovalRequirementDefinition(_) => {
                return Err(DiagramError::GeometryCannotRepresentNodeWeight(
                    node_weight.into(),
                ));
            }
            NodeWeight::Component(w) => GeometryRepresents::Component(w.id.into()),
            NodeWeight::DiagramObject(w) => {
                let DiagramObjectKind::View(view_id) = w.object_kind();
                GeometryRepresents::View(view_id)
            }
        };

        Ok(Some(geo_represents))
    }

    pub async fn get_by_id(ctx: &DalContext, geometry_id: GeometryId) -> DiagramResult<Self> {
        let (node_weight, content) = Self::get_node_weight_and_content(ctx, geometry_id).await?;
        Ok(Self::assemble(node_weight, content))
    }

    pub async fn list_ids_by_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> DiagramResult<Vec<GeometryId>> {
        let snap = ctx.workspace_snapshot()?;

        let mut geometries = vec![];
        for geometry_idx in snap
            .incoming_sources_for_edge_weight_kind(
                component_id,
                EdgeWeightKindDiscriminants::Represents,
            )
            .await?
        {
            let node_weight = snap
                .get_node_weight(geometry_idx)
                .await?
                .get_geometry_node_weight()?;

            geometries.push(node_weight.id().into())
        }

        Ok(geometries)
    }

    /// Returns all geometries for a component in a map, keyed by the view id
    pub async fn by_view_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> DiagramResult<HashMap<ViewId, Self>> {
        let mut result = HashMap::new();
        let snap = ctx.workspace_snapshot()?;

        for geometry_idx in snap
            .incoming_sources_for_edge_weight_kind(
                component_id,
                EdgeWeightKindDiscriminants::Represents,
            )
            .await?
        {
            let NodeWeight::Geometry(geo_inner) = snap.get_node_weight(geometry_idx).await? else {
                continue;
            };

            let geo_id = geo_inner.id();

            let Some(view_idx) = snap
                .incoming_sources_for_edge_weight_kind(geo_id, EdgeWeightKindDiscriminants::Use)
                .await?
                .pop()
            else {
                continue;
            };

            let NodeWeight::View(view_inner) = snap.get_node_weight(view_idx).await? else {
                continue;
            };

            let geo = Self::get_by_id(ctx, geo_id.into()).await?;

            result.insert(view_inner.id().into(), geo);
        }

        Ok(result)
    }

    pub async fn list_by_view_id(
        ctx: &DalContext,
        view_id: ViewId,
    ) -> DiagramResult<Vec<Geometry>> {
        let snap = ctx.workspace_snapshot()?;

        let mut geometries = vec![];
        for geometry_idx in snap
            .outgoing_targets_for_edge_weight_kind(view_id, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            let node_weight = snap
                .get_node_weight(geometry_idx)
                .await?
                .get_geometry_node_weight()?;

            let content = Self::try_get_content(ctx, &node_weight.content_hash())
                .await?
                .ok_or(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?;

            geometries.push(Self::assemble(node_weight, content))
        }

        Ok(geometries)
    }

    pub async fn try_get_by_component_and_view(
        ctx: &DalContext,
        component_id: ComponentId,
        view_id: ViewId,
    ) -> DiagramResult<Option<Self>> {
        let snap = ctx.workspace_snapshot()?;

        let mut maybe_weight = None;
        for geometry_idx in snap
            .incoming_sources_for_edge_weight_kind(
                component_id,
                EdgeWeightKindDiscriminants::Represents,
            )
            .await?
        {
            let node_weight = snap
                .get_node_weight(geometry_idx)
                .await?
                .get_geometry_node_weight()?;

            let this_view_id = Self::get_view_id_by_id(ctx, node_weight.id().into()).await?;

            if this_view_id == view_id {
                maybe_weight = Some(node_weight);
            }
        }

        let Some(node_weight) = maybe_weight else {
            return Ok(None);
        };

        let content = Self::try_get_content(ctx, &node_weight.content_hash())
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(
                node_weight.id(),
            ))?;

        Ok(Some(Self::assemble(node_weight, content)))
    }

    pub async fn get_by_component_and_view(
        ctx: &DalContext,
        component_id: ComponentId,
        view_id: ViewId,
    ) -> DiagramResult<Self> {
        Self::try_get_by_component_and_view(ctx, component_id, view_id)
            .await?
            .ok_or_else(|| DiagramError::GeometryNotFoundForComponentAndView(component_id, view_id))
    }

    pub async fn get_by_object_view_and_container_view(
        ctx: &DalContext,
        object_view: ViewId,
        container_view: ViewId,
    ) -> DiagramResult<Self> {
        let diagram_object = DiagramObject::get_for_view(ctx, object_view).await?;

        let snap = ctx.workspace_snapshot()?;

        let mut maybe_weight = None;
        for geometry_idx in snap
            .incoming_sources_for_edge_weight_kind(
                diagram_object.id(),
                EdgeWeightKindDiscriminants::Represents,
            )
            .await?
        {
            let node_weight = snap
                .get_node_weight(geometry_idx)
                .await?
                .get_geometry_node_weight()?;

            let this_view_id = Self::get_view_id_by_id(ctx, node_weight.id().into()).await?;

            if this_view_id == container_view {
                maybe_weight = Some(node_weight);
            }
        }

        let Some(node_weight) = maybe_weight else {
            return Err(DiagramError::GeometryNotFoundForViewObjectAndView(
                object_view,
                container_view,
            ));
        };

        let content = Self::try_get_content(ctx, &node_weight.content_hash())
            .await?
            .ok_or(WorkspaceSnapshotError::MissingContentFromStore(
                node_weight.id(),
            ))?;

        Ok(Self::assemble(node_weight, content))
    }

    pub async fn get_view_id_by_id(ctx: &DalContext, id: GeometryId) -> DiagramResult<ViewId> {
        let snap = ctx.workspace_snapshot()?;
        let view_idx = snap
            .incoming_sources_for_edge_weight_kind(id, EdgeWeightKindDiscriminants::Use)
            .await?
            .pop()
            .ok_or(DiagramError::ViewNotFoundForGeometry(id))?;

        Ok(snap
            .get_node_weight(view_idx)
            .await?
            .get_view_node_weight()?
            .id()
            .into())
    }

    async fn get_node_weight_and_content(
        ctx: &DalContext,
        geometry_id: GeometryId,
    ) -> DiagramResult<(GeometryNodeWeight, GeometryContent)> {
        Self::try_get_node_weight_and_content(ctx, geometry_id)
            .await?
            .ok_or(DiagramError::GeometryNotFound(geometry_id))
    }

    async fn try_get_node_weight_and_content(
        ctx: &DalContext,
        geometry_id: GeometryId,
    ) -> DiagramResult<Option<(GeometryNodeWeight, GeometryContent)>> {
        let Some(node_weight) = ctx
            .workspace_snapshot()?
            .get_node_weight_opt(geometry_id)
            .await
        else {
            return Ok(None);
        };

        let hash = node_weight.content_hash();
        let component_node_weight = node_weight.get_geometry_node_weight()?;

        let content = Self::try_get_content(ctx, &hash).await?.ok_or(
            WorkspaceSnapshotError::MissingContentFromStore(geometry_id.into()),
        )?;

        Ok(Some((component_node_weight, content)))
    }

    async fn try_get_content(
        ctx: &DalContext,
        hash: &ContentHash,
    ) -> DiagramResult<Option<GeometryContent>> {
        Ok(ctx.layer_db().cas().try_read_as(hash).await?)
    }

    pub async fn update(
        &mut self,
        ctx: &DalContext,
        new_geometry: RawGeometry,
    ) -> DiagramResult<()> {
        let timestamp = Timestamp::now();
        let geometry = new_geometry.clone();

        let (hash, _) = ctx.layer_db().cas().write(
            Arc::new(
                GeometryContent::V1(GeometryContentV1 {
                    timestamp,
                    x: new_geometry.x.to_string(),
                    y: new_geometry.y.to_string(),
                    width: new_geometry.width.map(|w| w.to_string()),
                    height: new_geometry.height.map(|h| h.to_string()),
                })
                .into(),
            ),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        ctx.workspace_snapshot()?
            .update_content(self.id.into(), hash)
            .await?;

        self.x = geometry.x;
        self.y = geometry.y;
        self.width = geometry.width;
        self.height = geometry.height;
        self.timestamp = timestamp;

        Ok(())
    }

    /// Removes a [Geometry] from the graph, provided it's not the last geometry for a component
    pub async fn remove(ctx: &DalContext, geometry_id: GeometryId) -> DiagramResult<()> {
        match Self::represented_id(ctx, geometry_id).await? {
            Some(GeometryRepresents::Component(component_id)) => {
                if Self::list_ids_by_component(ctx, component_id).await?.len() == 1 {
                    let view_id = Self::get_view_id_by_id(ctx, geometry_id).await?;
                    return Err(DiagramError::DeletingLastGeometryForComponent(
                        view_id,
                        component_id,
                    ));
                }
            }
            // There's no problem in deleting all geometries for a view
            Some(GeometryRepresents::View(_)) => {}
            // There's no problem in deleting orphan geometries
            None => {}
        }

        ctx.workspace_snapshot()?
            .remove_node_by_id(geometry_id)
            .await?;

        Ok(())
    }

    pub async fn remove_all_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> DiagramResult<()> {
        let snap = ctx.workspace_snapshot()?;

        let geometries = snap
            .incoming_sources_for_edge_weight_kind(
                component_id,
                EdgeWeightKindDiscriminants::Represents,
            )
            .await?;

        for geometry_idx in geometries {
            let geometry_id = snap.get_node_weight(geometry_idx).await?.id();

            snap.remove_node_by_id(geometry_id).await?;
        }

        Ok(())
    }

    pub async fn restore_all_for_component_id(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> DiagramResult<()> {
        let base_change_set_ctx = ctx.clone_with_base().await?;

        for geo_id in Self::list_ids_by_component(&base_change_set_ctx, component_id).await? {
            let view_id = Self::get_view_id_by_id(&base_change_set_ctx, geo_id).await?;

            // Check if view exists on this changeset
            if View::try_get_by_id(ctx, view_id).await?.is_none() {
                continue;
            };

            let head_geometry = Self::get_by_id(&base_change_set_ctx, geo_id).await?;

            Self::new_for_component(ctx, component_id, view_id)
                .await?
                .update(
                    ctx,
                    RawGeometry {
                        x: head_geometry.x(),
                        y: head_geometry.y(),
                        width: head_geometry.width(),
                        height: head_geometry.height(),
                    },
                )
                .await?;
        }

        Ok(())
    }
}
