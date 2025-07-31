use serde::{
    Deserialize,
    Serialize,
};
use si_events::ulid::Ulid;

use crate::{
    DalContext,
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    diagram::{
        DiagramError,
        DiagramResult,
        view::ViewId,
    },
    implement_add_edge_to,
    workspace_snapshot::node_weight::{
        NodeWeight,
        category_node_weight::CategoryNodeKind,
        diagram_object_node_weight::{
            DiagramObjectKind,
            DiagramObjectNodeWeight,
        },
        traits::SiVersionedNodeWeight,
    },
};

/// Represents spatial data for something to be shown on a view
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct DiagramObject {
    id: Ulid,
    object_kind: DiagramObjectKind,
}

impl DiagramObject {
    implement_add_edge_to!(
        source_id: Ulid,
        destination_id: Ulid,
        add_fn: add_category_edge,
        discriminant: EdgeWeightKindDiscriminants::Use,
        result: DiagramResult,
    );

    implement_add_edge_to!(
        source_id: ViewId,
        destination_id: Ulid,
        add_fn: add_view_edge,
        discriminant: EdgeWeightKindDiscriminants::DiagramObject,
        result: DiagramResult,
    );

    pub fn id(&self) -> Ulid {
        self.id
    }

    fn assemble(node_weight: DiagramObjectNodeWeight) -> Self {
        Self {
            id: node_weight.id(),
            object_kind: node_weight.object_kind(),
        }
    }

    pub async fn new_for_view(ctx: &DalContext, view_id: ViewId) -> DiagramResult<Self> {
        let snap = ctx.workspace_snapshot()?;
        let id = snap.generate_ulid().await?;
        let lineage_id = snap.generate_ulid().await?;

        let node_weight =
            NodeWeight::new_diagram_object(id, lineage_id, DiagramObjectKind::View(view_id));

        snap.add_or_replace_node(node_weight.clone()).await?;

        Self::add_view_edge(ctx, view_id, id, EdgeWeightKind::DiagramObject).await?;

        let diagram_object_category_id = snap
            .get_category_node_or_err(CategoryNodeKind::DiagramObject)
            .await?;
        Self::add_category_edge(
            ctx,
            diagram_object_category_id,
            id,
            EdgeWeightKind::new_use(),
        )
        .await?;

        Ok(Self::assemble(node_weight.get_diagram_object_weight()?))
    }

    /// Search for the view object linked to a view. If one does not exist, create one
    pub async fn get_for_view(ctx: &DalContext, view_id: ViewId) -> DiagramResult<Self> {
        let snap = ctx.workspace_snapshot()?;

        let diagram_object_idxs = snap
            .outgoing_targets_for_edge_weight_kind(
                view_id,
                EdgeWeightKindDiscriminants::DiagramObject,
            )
            .await?;

        if diagram_object_idxs.len() > 1 {
            return Err(DiagramError::DiagramObjectMoreThanOneForView(view_id));
        }

        // If we don't find one for a view, we create it
        let diagram_object = match diagram_object_idxs.first() {
            None => Self::new_for_view(ctx, view_id).await?,
            Some(idx) => {
                let node_weight = snap
                    .get_node_weight(*idx)
                    .await?
                    .get_diagram_object_weight()?;
                Self::assemble(node_weight)
            }
        };

        Ok(diagram_object)
    }
}
