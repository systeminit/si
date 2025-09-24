use itertools::Itertools;
use petgraph::{
    Direction::Incoming,
    prelude::*,
    stable_graph::EdgeReference,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};
use si_id::ComponentId;

use super::{
    NodeWeight,
    NodeWeightDiscriminants,
    NodeWeightDiscriminants::Component,
    NodeWeightError,
    NodeWeightResult,
    category_node_weight::CategoryNodeKind,
    traits::CorrectTransformsResult,
};
use crate::{
    EdgeWeight,
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    WorkspaceSnapshotGraphVCurrent,
    workspace_snapshot::{
        NodeInformation,
        content_address::{
            ContentAddress,
            ContentAddressDiscriminants,
        },
        graph::{
            LineageId,
            detector::Update,
        },
        node_weight::traits::CorrectTransforms,
    },
};

mod split_corrections;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ComponentNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    content_address: ContentAddress,
    merkle_tree_hash: MerkleTreeHash,
    to_delete: bool,
}

impl ComponentNodeWeight {
    pub fn new(id: Ulid, lineage_id: Ulid, content_address: ContentAddress) -> Self {
        Self {
            id,
            lineage_id,
            content_address,
            merkle_tree_hash: MerkleTreeHash::default(),
            to_delete: false,
        }
    }

    pub fn content_address(&self) -> ContentAddress {
        self.content_address
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![self.content_address.content_hash()]
    }

    pub fn content_hash(&self) -> ContentHash {
        self.content_address.content_hash()
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn to_delete(&self) -> bool {
        self.to_delete
    }

    pub fn set_to_delete(&mut self, to_delete: bool) -> &mut Self {
        self.to_delete = to_delete;
        self
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        let new_address = match &self.content_address {
            ContentAddress::Component(_) => ContentAddress::Component(content_hash),
            other => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    Into::<ContentAddressDiscriminants>::into(other).to_string(),
                    ContentAddressDiscriminants::Component.to_string(),
                ));
            }
        };

        self.content_address = new_address;

        Ok(())
    }

    pub fn node_hash(&self) -> ContentHash {
        let mut content_hasher = ContentHash::hasher();
        content_hasher.update(self.content_address.content_hash().as_bytes());
        content_hasher.update(&[u8::from(self.to_delete)]);

        content_hasher.finalize()
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub fn overwrite_id(&mut self, id: Ulid) {
        self.id = id
    }

    pub fn overwrite_lineage_id(&mut self, id: LineageId) {
        self.lineage_id = id
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[
            EdgeWeightKindDiscriminants::Use,
            EdgeWeightKindDiscriminants::Root,
        ]
    }
}

impl From<&ComponentNodeWeight> for NodeInformation {
    fn from(value: &ComponentNodeWeight) -> Self {
        Self {
            node_weight_kind: Component,
            id: value.id.into(),
        }
    }
}

impl CorrectTransforms for ComponentNodeWeight {
    fn correct_transforms(
        &self,
        graph: &WorkspaceSnapshotGraphVCurrent,
        mut updates: Vec<Update>,
        _from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update>> {
        // New components don't have any corrective transforms.
        let component_id: ComponentId = self.id.into();
        let Some(component_node_idx) = graph.get_node_index_by_id_opt(component_id) else {
            return Ok(updates);
        };

        // Helper to compactly match whether destination/source is us
        let is_self = |node: &NodeInformation| node.id == self.id.into();

        let mut component_will_be_deleted = false;
        let mut remove_edges = vec![];
        for update in &updates {
            match update {
                // Single Parent Rule: Component: FrameContains <Self> <- FrameContains: Component
                // When we're setting the parent for a component, we need to remove any existing
                // FrameContains edges to other components.
                Update::NewEdge {
                    destination,
                    edge_weight,
                    ..
                } if EdgeWeightKind::FrameContains == *edge_weight.kind()
                    && is_self(destination) =>
                {
                    // We want to remove any existing FrameContains edges and honor this AddEdge.
                    remove_edges.extend(
                        graph.incoming_edges(component_node_idx, EdgeWeightKind::FrameContains),
                    );
                }

                // If the component is being deleted, the RemoveEdges may be stale (from an old
                // snapshot) and we need to ensure that we truly delete everything. Detected by
                // noticing an edge was removed from the component category:
                //
                //   Category -> Use: <Self>
                Update::RemoveEdge {
                    source,
                    destination,
                    edge_kind,
                } if EdgeWeightKindDiscriminants::Use == *edge_kind
                    && is_self(destination)
                    && Some(CategoryNodeKind::Component) == category_kind(graph, source) =>
                {
                    component_will_be_deleted = true;
                }
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } if EdgeWeightKindDiscriminants::Use == edge_weight.kind().into()
                    && is_self(destination)
                    && Some(CategoryNodeKind::Component) == category_kind(graph, source) =>
                {
                    // If the edge was disconnected and then reconnected, the component will
                    // not be deleted.
                    component_will_be_deleted = false;
                }

                // If SchemaVariant gets set, we are upgrading a component, which disconnects
                // and reconnects prop and socket values and connections. The disconnects may
                // be stale (based on an old snapshot), so when we detect schema upgrade, we
                // redo the disconnects.
                Update::NewEdge {
                    source,
                    edge_weight,
                    destination,
                } if EdgeWeightKindDiscriminants::Use == edge_weight.kind().into()
                    && is_self(source)
                    && destination.node_weight_kind == NodeWeightDiscriminants::SchemaVariant =>
                {
                    // Root props and sockets get all new AttributeValues during upgrade, but
                    // the RemoveEdges for the old ones may be stale; RemoveEdge the real ones
                    // just in case.
                    remove_edges
                        .extend(graph.outgoing_edges(component_node_idx, EdgeWeightKind::Root));
                    remove_edges.extend(
                        graph.outgoing_edges(component_node_idx, EdgeWeightKind::SocketValue),
                    );
                }

                _ => {}
            }
        }

        if component_will_be_deleted {
            // All edges incoming to the root attribute value node (for example, ValueSubscription edges)
            // must be deleted, so that the attribute value tree disappears from the graph on cleanup.
            if let Some((_, _, root_av_idx)) = graph
                .edges_directed_for_edge_weight_kind(
                    component_node_idx,
                    Outgoing,
                    EdgeWeightKindDiscriminants::Root,
                )
                .next()
            {
                remove_edges.extend(graph.edges_directed(root_av_idx, Incoming));
            }

            // Also remove any incoming edges to the component in case there
            // is a frame contains in another change set
            remove_edges.extend(graph.edges_directed(component_node_idx, Incoming));
        }

        // Prepend any RemoveEdges so they happen *before* any NewEdge
        if !remove_edges.is_empty() {
            let old_updates = updates;
            updates = remove_edges
                .into_iter()
                .map(|edge| remove_edge(graph, edge))
                .try_collect()?;
            updates.extend(old_updates);
        }

        Ok(updates)
    }
}

/// Get the category kind (if this is a category node)
fn category_kind(
    graph: &WorkspaceSnapshotGraphVCurrent,
    category: &NodeInformation,
) -> Option<CategoryNodeKind> {
    graph
        .get_node_weight_by_id_opt(category.id)
        .and_then(|node_weight| match node_weight {
            NodeWeight::Category(inner) => Some(inner.kind()),
            _ => None,
        })
}

/// Creates an Update::RemoveEdge from an EdgeReference by looking up the nodes.
fn remove_edge(
    graph: &WorkspaceSnapshotGraphVCurrent,
    edge: EdgeReference<'_, EdgeWeight>,
) -> CorrectTransformsResult<Update> {
    let source = graph.get_node_weight(edge.source())?;
    let destination = graph.get_node_weight(edge.target())?;
    Ok(Update::RemoveEdge {
        source: source.into(),
        destination: destination.into(),
        edge_kind: edge.weight().kind().into(),
    })
}
