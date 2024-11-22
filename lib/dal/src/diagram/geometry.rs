use crate::diagram::view::{View, ViewId};
use crate::diagram::{DiagramError, DiagramResult};
use crate::layer_db_types::{GeometryContent, GeometryContentV1};
use crate::workspace_snapshot::node_weight::geometry_node_weight::GeometryNodeWeight;
use crate::workspace_snapshot::node_weight::traits::SiVersionedNodeWeight;
use crate::workspace_snapshot::node_weight::NodeWeight;
use crate::{
    id, implement_add_edge_to, ComponentId, EdgeWeightKindDiscriminants, Timestamp,
    WorkspaceSnapshotError,
};
use crate::{DalContext, EdgeWeightKind};
use jwt_simple::prelude::{Deserialize, Serialize};
use si_events::ulid::Ulid;
use si_events::ContentHash;
pub use si_frontend_types::RawGeometry;
use std::sync::Arc;

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

id!(GeometryId);

/// Represents spatial data for something to be shown on a view
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
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

    pub async fn new(
        ctx: &DalContext,
        component_id: ComponentId,
        view_id: ViewId,
    ) -> DiagramResult<Self> {
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

        Self::add_edge_to_component(ctx, id.into(), component_id, EdgeWeightKind::Represents)
            .await?;

        View::add_geometry_by_id(ctx, view_id, id.into()).await?;

        Ok(Self::assemble(
            node_weight.get_geometry_node_weight()?,
            content,
        ))
    }

    // Some changesets have orphan geometries because of an old bug, so when calling this function.
    // be careful when dealing with the ComponentNotFoundForGeometry error
    pub async fn component_id(
        ctx: &DalContext,
        geometry_id: GeometryId,
    ) -> DiagramResult<ComponentId> {
        let snap = ctx.workspace_snapshot()?;

        let component_id = snap
            .outgoing_targets_for_edge_weight_kind(
                geometry_id,
                EdgeWeightKindDiscriminants::Represents,
            )
            .await?
            .pop()
            .ok_or(DiagramError::ComponentNotFoundForGeometry(geometry_id))?;

        Ok(snap
            .get_node_weight(component_id)
            .await?
            .get_component_node_weight()?
            .id
            .into())
    }

    pub async fn get_by_id(ctx: &DalContext, component_id: GeometryId) -> DiagramResult<Self> {
        let (node_weight, content) = Self::get_node_weight_and_content(ctx, component_id).await?;
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
        let id: Ulid = geometry_id.into();

        let Some(node_index) = ctx.workspace_snapshot()?.get_node_index_by_id_opt(id).await else {
            return Ok(None);
        };

        let node_weight = ctx
            .workspace_snapshot()?
            .get_node_weight(node_index)
            .await?;

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
        match Self::component_id(ctx, geometry_id).await {
            Ok(id) => {
                if Self::list_ids_by_component(ctx, id).await?.len() == 1 {
                    let view_id = Self::get_view_id_by_id(ctx, geometry_id).await?;
                    return Err(DiagramError::DeletingLastGeometryForComponent(view_id, id));
                }
            }
            // There's no problem in deleting orphan geometries
            Err(DiagramError::ComponentNotFoundForGeometry(_)) => {}
            Err(e) => return Err(e),
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
}
