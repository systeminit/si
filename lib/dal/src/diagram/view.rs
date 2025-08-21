use std::{
    collections::{
        HashMap,
        HashSet,
    },
    sync::Arc,
};

use chrono::Utc;
use petgraph::Outgoing;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ComponentId,
    ContentHash,
    Timestamp,
    ulid::Ulid,
};
use si_frontend_types::RawGeometry;
pub use si_id::ViewId;

use crate::{
    ChangeSetId,
    DalContext,
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    WorkspaceSnapshotError,
    WsEvent,
    WsEventResult,
    WsPayload,
    diagram::{
        DiagramError,
        DiagramResult,
        diagram_object::DiagramObject,
        geometry::{
            Geometry,
            GeometryId,
        },
    },
    implement_add_edge_to,
    layer_db_types::{
        ViewContent,
        ViewContentV1,
    },
    workspace_snapshot::{
        node_weight::{
            NodeWeight,
            category_node_weight::CategoryNodeKind,
            traits::SiVersionedNodeWeight,
            view_node_weight::ViewNodeWeight,
        },
        traits::diagram::view::ViewExt,
    },
};

/// Represents spatial data for something to be shown on a view
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct View {
    id: ViewId,
    name: String,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl View {
    implement_add_edge_to!(
        source_id: ViewId,
        destination_id: GeometryId,
        add_fn: add_edge_to_geometry,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: DiagramResult,
    );

    implement_add_edge_to!(
        source_id: Ulid,
        destination_id: ViewId,
        add_fn: add_category_edge,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: DiagramResult,
    );

    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    pub fn id(&self) -> ViewId {
        self.id
    }

    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }

    pub async fn is_default(&self, ctx: &DalContext) -> DiagramResult<bool> {
        let default_id = Self::get_id_for_default(ctx).await?;

        Ok(default_id == self.id)
    }

    fn assemble(node_weight: ViewNodeWeight, content: ViewContent) -> Self {
        let content = content.extract();

        Self {
            id: node_weight.id().into(),
            timestamp: content.timestamp,
            name: content.name,
        }
    }

    pub async fn new(ctx: &DalContext, name: impl AsRef<str>) -> DiagramResult<Self> {
        let snap = ctx.workspace_snapshot()?;
        let id = snap.generate_ulid().await?;
        let lineage_id = snap.generate_ulid().await?;

        let content = ViewContent::V1(ViewContentV1 {
            timestamp: Timestamp::now(),
            name: name.as_ref().to_owned(),
        });

        let (content_address, _) = ctx.layer_db().cas().write(
            Arc::new(content.clone().into()),
            None,
            ctx.events_tenancy(),
            ctx.events_actor(),
        )?;

        let node_weight = NodeWeight::new_view(id, lineage_id, content_address);
        snap.add_or_replace_node(node_weight.clone()).await?;

        // Root --> View Category --> View (this)
        let view_category_id = snap
            .get_category_node_or_err(CategoryNodeKind::View)
            .await?;
        Self::add_category_edge(ctx, view_category_id, id.into(), EdgeWeightKind::new_use())
            .await?;

        // View (this) --DiagramObject-> DiagramObject <-Represents-- Geometry
        DiagramObject::new_for_view(ctx, id.into()).await?;

        Ok(Self::assemble(node_weight.get_view_node_weight()?, content))
    }

    pub async fn get_by_id(ctx: &DalContext, view_id: ViewId) -> DiagramResult<Self> {
        Self::try_get_by_id(ctx, view_id)
            .await?
            .ok_or(DiagramError::ViewNotFound(view_id))
    }

    pub async fn try_get_by_id(ctx: &DalContext, view_id: ViewId) -> DiagramResult<Option<Self>> {
        let Some((node_weight, content)) =
            Self::try_get_node_weight_and_content(ctx, view_id).await?
        else {
            return Ok(None);
        };
        Ok(Some(Self::assemble(node_weight, content)))
    }

    pub async fn find_by_name(ctx: &DalContext, name: &str) -> DiagramResult<Option<Self>> {
        for view_node_weight in Self::list_node_weights(ctx).await? {
            let content = Self::try_get_content(ctx, &view_node_weight.content_hash())
                .await?
                .ok_or(WorkspaceSnapshotError::MissingContentFromStore(
                    view_node_weight.id(),
                ))?;

            let view = Self::assemble(view_node_weight, content);

            if view.name == name {
                return Ok(Some(view));
            }
        }

        Ok(None)
    }

    pub async fn list_ids(ctx: &DalContext) -> DiagramResult<Vec<ViewId>> {
        Ok(Self::list_node_weights(ctx)
            .await?
            .into_iter()
            .map(|w| ViewId::from(w.id()))
            .collect())
    }

    async fn list_node_weights(ctx: &DalContext) -> DiagramResult<Vec<ViewNodeWeight>> {
        let snap = ctx.workspace_snapshot()?;

        let category_node = snap
            .get_category_node(CategoryNodeKind::View)
            .await?
            .ok_or(DiagramError::ViewCategoryNotFound)?;

        let mut views = vec![];
        for view_idx in snap
            .outgoing_targets_for_edge_weight_kind(category_node, EdgeWeightKindDiscriminants::Use)
            .await?
        {
            let view_node_weight = snap
                .get_node_weight(view_idx)
                .await?
                .get_view_node_weight()?;

            views.push(view_node_weight);
        }

        Ok(views)
    }

    pub async fn list(ctx: &DalContext) -> DiagramResult<Vec<Self>> {
        let mut views = vec![];
        for view_node_weight in Self::list_node_weights(ctx).await? {
            let content = Self::try_get_content(ctx, &view_node_weight.content_hash())
                .await?
                .ok_or(WorkspaceSnapshotError::MissingContentFromStore(
                    view_node_weight.id(),
                ))?;

            views.push(Self::assemble(view_node_weight, content));
        }

        Ok(views)
    }

    pub async fn get_id_for_default(ctx: &DalContext) -> DiagramResult<ViewId> {
        let snap = ctx.workspace_snapshot()?;

        let view_category_id = snap
            .get_category_node(CategoryNodeKind::View)
            .await?
            .ok_or(DiagramError::ViewCategoryNotFound)?;

        let mut maybe_default_view = None;
        for (edge, _from_idx, to_idx) in snap
            .edges_directed_for_edge_weight_kind(
                view_category_id,
                Outgoing,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?
        {
            if *edge.kind() == EdgeWeightKind::new_use_default() {
                maybe_default_view = Some(snap.get_node_weight(to_idx).await?.id())
            }
        }

        let Some(default_view) = maybe_default_view else {
            let mut current_views = Self::list(ctx).await?;
            current_views.sort_by_key(|f| f.id);
            // We should get the view with the lowest ID and return that!
            if let Some(view) = current_views.first() {
                return Ok(view.id);
            } else {
                return Err(DiagramError::DefaultViewNotFound);
            }
        };

        Ok(default_view.into())
    }

    async fn try_get_node_weight_and_content(
        ctx: &DalContext,
        view_id: ViewId,
    ) -> DiagramResult<Option<(ViewNodeWeight, ViewContent)>> {
        let id: Ulid = view_id.into();

        let node_weight = match ctx.workspace_snapshot()?.get_node_weight_opt(id).await {
            Some(node_weight) => node_weight.get_view_node_weight()?,
            None => return Ok(None),
        };

        let hash = node_weight.content_hash();

        let content = Self::try_get_content(ctx, &hash).await?.ok_or(
            WorkspaceSnapshotError::MissingContentFromStore(view_id.into()),
        )?;

        Ok(Some((node_weight, content)))
    }

    async fn try_get_content(
        ctx: &DalContext,
        hash: &ContentHash,
    ) -> DiagramResult<Option<ViewContent>> {
        Ok(ctx.layer_db().cas().try_read_as(hash).await?)
    }

    pub async fn add_geometry_by_id(
        ctx: &DalContext,
        view_id: ViewId,
        geometry_id: GeometryId,
    ) -> DiagramResult<()> {
        Self::add_edge_to_geometry(
            ctx,
            view_id,
            geometry_id,
            EdgeWeightKind::Use { is_default: false },
        )
        .await
    }

    pub async fn set_name(&mut self, ctx: &DalContext, name: impl AsRef<str>) -> DiagramResult<()> {
        let (hash, _) = ctx.layer_db().cas().write(
            Arc::new(
                ViewContent::V1(ViewContentV1 {
                    timestamp: Timestamp {
                        created_at: self.timestamp.created_at,
                        updated_at: Utc::now(),
                    },
                    name: name.as_ref().to_owned(),
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

        self.name = name.as_ref().to_string();

        Ok(())
    }

    pub async fn set_default(ctx: &DalContext, view_id: ViewId) -> DiagramResult<()> {
        let snap = ctx.workspace_snapshot()?;

        let view_category_id = snap
            .get_category_node(CategoryNodeKind::View)
            .await?
            .ok_or(DiagramError::ViewCategoryNotFound)?;

        // Update edge to old default
        for (edge, from_idx, to_idx) in snap
            .edges_directed_for_edge_weight_kind(
                view_category_id,
                Outgoing,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?
        {
            if *edge.kind() == EdgeWeightKind::new_use_default() {
                // We have found the existing previous default view
                // we now need to update that edge to be a Use
                snap.remove_edge(from_idx, to_idx, edge.kind().into())
                    .await?;

                Self::add_category_edge(
                    ctx,
                    view_category_id,
                    snap.get_node_weight(to_idx).await?.id().into(),
                    EdgeWeightKind::new_use(),
                )
                .await?;
            }
        }

        snap.remove_edge(view_category_id, view_id, EdgeWeightKindDiscriminants::Use)
            .await?;

        Self::add_category_edge(
            ctx,
            view_category_id,
            view_id,
            EdgeWeightKind::new_use_default(),
        )
        .await?;

        Ok(())
    }

    pub async fn geometry(&self, ctx: &DalContext, view_id: ViewId) -> DiagramResult<Geometry> {
        Geometry::get_by_object_view_and_container_view(ctx, self.id, view_id).await
    }

    pub async fn set_geometry(
        &mut self,
        ctx: &DalContext,
        view_id: ViewId,
        x: impl Into<isize>,
        y: impl Into<isize>,
        width: Option<impl Into<isize>>,
        height: Option<impl Into<isize>>,
    ) -> DiagramResult<Geometry> {
        let new_geometry = RawGeometry {
            x: x.into(),
            y: y.into(),
            width: width.map(|w| w.into()),
            height: height.map(|h| h.into()),
        };

        self.set_raw_geometry(ctx, new_geometry, view_id).await
    }

    pub async fn set_raw_geometry(
        &mut self,
        ctx: &DalContext,
        raw_geometry: RawGeometry,
        view_id: ViewId,
    ) -> DiagramResult<Geometry> {
        let mut geometry_pre = self.geometry(ctx, view_id).await?;
        if geometry_pre.into_raw() != raw_geometry {
            geometry_pre.update(ctx, raw_geometry).await?;
        }

        Ok(geometry_pre)
    }

    pub async fn remove(ctx: &DalContext, view_id: ViewId) -> DiagramResult<()> {
        ctx.workspace_snapshot()?
            .view_remove(view_id)
            .await
            .map_err(Into::into)
    }
}

/// Frontend representation for a [View](View) with a geometry.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ViewObjectView {
    view: ViewView,
    geometry: RawGeometry,
}

impl ViewObjectView {
    pub async fn from_view_and_geometry(
        ctx: &DalContext,
        view: View,
        geometry: RawGeometry,
    ) -> DiagramResult<Self> {
        Ok(Self {
            view: ViewView::from_view(ctx, view).await?,
            geometry,
        })
    }
}

/// Frontend representation for a [View](View).
/// Yeah, it's a silly name, but all the other frontend representation structs are *View,
/// so we either keep it or change everything.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ViewView {
    id: ViewId,
    name: String,
    is_default: bool,
    #[serde(flatten)]
    timestamp: Timestamp,
}

impl ViewView {
    pub async fn from_view(ctx: &DalContext, view: View) -> DiagramResult<Self> {
        Ok(ViewView {
            id: view.id(),
            name: view.name().to_owned(),
            is_default: view.is_default(ctx).await?,
            timestamp: view.timestamp().to_owned(),
        })
    }
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone, Default)]
pub struct ViewComponentsUpdateSingle {
    pub added: HashMap<ComponentId, RawGeometry>,
    pub removed: HashSet<ComponentId>,
}

pub type ViewComponentsUpdateList = HashMap<ViewId, ViewComponentsUpdateSingle>;

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ViewComponentsUpdatePayload {
    change_set_id: ChangeSetId,
    updates_by_view: ViewComponentsUpdateList,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ViewWsPayload {
    change_set_id: ChangeSetId,
    view: ViewView,
}

#[derive(Debug, Deserialize, Eq, PartialEq, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ViewDeletedPayload {
    change_set_id: ChangeSetId,
    view_id: ViewId,
}

impl WsEvent {
    pub async fn view_created(ctx: &DalContext, view: ViewView) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ViewCreated(ViewWsPayload {
                change_set_id: ctx.change_set_id(),
                view,
            }),
        )
        .await
    }

    pub async fn view_updated(ctx: &DalContext, view: ViewView) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ViewUpdated(ViewWsPayload {
                change_set_id: ctx.change_set_id(),
                view,
            }),
        )
        .await
    }

    pub async fn view_deleted(ctx: &DalContext, view_id: ViewId) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ViewDeleted(ViewDeletedPayload {
                change_set_id: ctx.change_set_id(),
                view_id,
            }),
        )
        .await
    }

    pub async fn view_components_update(
        ctx: &DalContext,
        updates_by_view: ViewComponentsUpdateList,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ViewComponentsUpdate(ViewComponentsUpdatePayload {
                change_set_id: ctx.change_set_id(),
                updates_by_view,
            }),
        )
        .await
    }
}
