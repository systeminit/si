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
use std::sync::Arc;

const DEFAULT_COMPONENT_X_POSITION: &str = "0";
const DEFAULT_COMPONENT_Y_POSITION: &str = "0";
const DEFAULT_COMPONENT_WIDTH: &str = "500";
const DEFAULT_COMPONENT_HEIGHT: &str = "500";

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct RawGeometry {
    pub x: String,
    pub y: String,
    pub width: Option<String>,
    pub height: Option<String>,
}

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
    x: String,
    y: String,
    width: Option<String>,
    height: Option<String>,
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

    pub fn x(&self) -> &str {
        &self.x
    }

    pub fn y(&self) -> &str {
        &self.y
    }

    pub fn width(&self) -> Option<&str> {
        self.width.as_deref()
    }

    pub fn height(&self) -> Option<&str> {
        self.height.as_deref()
    }

    fn assemble(node_weight: GeometryNodeWeight, content: GeometryContent) -> Self {
        let content = content.extract();

        Self {
            id: node_weight.id().into(),
            timestamp: content.timestamp,
            x: content.x,
            y: content.y,
            width: content.width,
            height: content.height,
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

        let (content_address, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(content.clone().into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

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

    pub async fn get_by_id(ctx: &DalContext, component_id: GeometryId) -> DiagramResult<Self> {
        let (node_weight, content) = Self::get_node_weight_and_content(ctx, component_id).await?;
        Ok(Self::assemble(node_weight, content))
    }

    pub async fn list_ids_by_component(
        ctx: &DalContext,
        component_id: ComponentId,
    ) -> DiagramResult<Vec<ViewId>> {
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

    pub async fn get_by_component_and_view(
        ctx: &DalContext,
        component_id: ComponentId,
        view_id: ViewId,
    ) -> DiagramResult<Self> {
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
            return Err(DiagramError::GeometryNotFoundForComponentAndView(
                component_id,
                view_id,
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
        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(
                    GeometryContent::V1(GeometryContentV1 {
                        timestamp: Timestamp::now(),
                        x: new_geometry.x,
                        y: new_geometry.y,
                        width: new_geometry.width,
                        height: new_geometry.height,
                    })
                    .into(),
                ),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        ctx.workspace_snapshot()?
            .update_content(self.id.into(), hash)
            .await?;

        Ok(())
    }
}
