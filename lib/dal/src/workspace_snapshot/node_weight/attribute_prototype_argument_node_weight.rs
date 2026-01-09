use std::collections::HashSet;

use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    Timestamp,
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};
use si_id::ComponentId;
use telemetry::prelude::*;

use crate::{
    EdgeWeightKindDiscriminants,
    workspace_snapshot::{
        graph::{
            LineageId,
            detector::Update,
        },
        node_weight::traits::CorrectTransforms,
    },
};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct AttributePrototypeArgumentNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    pub merkle_tree_hash: MerkleTreeHash,
    targets: Option<ArgumentTargets>,
    // NOTE (jkeiser) this is the only node that has timestamp; should we even have it?
    timestamp: Timestamp,
}

/// When this `AttributePrototypeArgument` represents a connection between two
/// components, we need to know which components are being connected.
///
/// TODO (jkeiser) this currently exists solely to allow old graphs to be deserialized.
/// Remove when we move to a new snapshot version.
#[derive(Copy, Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Hash)]
struct ArgumentTargets {
    source_component_id: ComponentId,
    destination_component_id: ComponentId,
}

impl AttributePrototypeArgumentNodeWeight {
    pub fn new(id: Ulid, lineage_id: Ulid) -> Self {
        Self {
            id,
            lineage_id,
            merkle_tree_hash: MerkleTreeHash::default(),
            targets: None,
            timestamp: Timestamp::now(),
        }
    }

    // Used only for the deserialization test, to ensure we can read old graphs.
    pub fn new_for_deserialization_test() -> Self {
        Self {
            id: Ulid::new(),
            lineage_id: Ulid::new(),
            merkle_tree_hash: MerkleTreeHash::default(),
            targets: Some(ArgumentTargets {
                source_component_id: ComponentId::new(),
                destination_component_id: ComponentId::new(),
            }),
            timestamp: Timestamp::now(),
        }
    }

    pub fn timestamp(&self) -> &Timestamp {
        &self.timestamp
    }

    pub fn set_timestamp(&mut self, timestamp: Timestamp) {
        self.timestamp = timestamp;
    }

    pub fn content_hash(&self) -> ContentHash {
        self.node_hash()
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![]
    }

    pub fn node_hash(&self) -> ContentHash {
        let mut content_hasher = ContentHash::hasher();
        // This makes the node_hash() the same as before (we used "" when we had targets = None)
        content_hasher.update("".as_bytes());
        content_hasher.finalize()
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

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[
            EdgeWeightKindDiscriminants::Use,
            EdgeWeightKindDiscriminants::PrototypeArgumentValue,
        ]
    }
}

impl std::fmt::Debug for AttributePrototypeArgumentNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("AttributePrototypeArgumentNodeWeight")
            .field("id", &self.id().to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("node_hash", &self.node_hash())
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .finish()
    }
}

impl CorrectTransforms for AttributePrototypeArgumentNodeWeight {
    fn correct_transforms(
        &self,
        graph: &crate::WorkspaceSnapshotGraphVCurrent,
        mut updates: Vec<crate::workspace_snapshot::graph::detector::Update>,
        _from_different_change_set: bool,
    ) -> super::traits::CorrectTransformsResult<
        Vec<crate::workspace_snapshot::graph::detector::Update>,
    > {
        // If we are in the current graph, don't modify the updates.
        if graph.get_node_index_by_id_opt(self.id).is_some() {
            return Ok(updates);
        }

        // Setup our caches needed to both determine which updates to remove and to perform the
        // removals.
        let mut destinations_that_do_not_exist: HashSet<Ulid> = HashSet::new();
        let mut all_new_nodes = HashSet::new();
        let mut indices_for_new_node_updates_for_ourself = Vec::new();

        // For each update, we need to collect all destinations that do not exist in the current
        // graph where we are the source for subscriptions. Along the way, we need to collect all
        // new nodes as well as cache the indices for all new node updates to remove.
        for (idx, update) in updates.iter().enumerate() {
            match update {
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } if source.id == self.id.into()
                    && graph.get_node_index_by_id_opt(destination.id).is_none()
                    && EdgeWeightKindDiscriminants::from(edge_weight.kind())
                        == EdgeWeightKindDiscriminants::ValueSubscription =>
                {
                    destinations_that_do_not_exist.insert(destination.id.into());
                }
                Update::NewNode { node_weight } => {
                    all_new_nodes.insert(node_weight.id());

                    // This should happen one or zero times.
                    if node_weight.id() == self.id {
                        indices_for_new_node_updates_for_ourself.push(idx);
                    }
                }
                _ => {}
            }
        }

        // If the destinations that no longer exist will not be included, then we need to remove the
        // update for ourself.
        if !destinations_that_do_not_exist.is_subset(&all_new_nodes) {
            if indices_for_new_node_updates_for_ourself.len() > 1 {
                warn!(
                    attribute_prototype_argument_id=%self.id,
                    update_count=%indices_for_new_node_updates_for_ourself.len(),
                    "unexpected multiple new node updates for ourself"
                );
            }

            // Reverse the indices vec to remove from the back first.
            indices_for_new_node_updates_for_ourself.reverse();
            for idx in indices_for_new_node_updates_for_ourself {
                updates.remove(idx);
            }
        }

        Ok(updates)
    }
}
