use std::collections::HashSet;

use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use crate::{
    action::ActionState,
    workspace_snapshot::{
        graph::{deprecated::v1::DeprecatedActionNodeWeightV1, detector::Update, LineageId},
        node_weight::{traits::CorrectTransforms, NodeWeight},
        NodeId, NodeInformation,
    },
    ChangeSetId, EdgeWeightKindDiscriminants, NodeWeightDiscriminants,
    WorkspaceSnapshotGraphVCurrent,
};

use super::traits::CorrectTransformsResult;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ActionNodeWeight {
    pub id: Ulid,
    state: ActionState,
    originating_change_set_id: ChangeSetId,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
}

impl ActionNodeWeight {
    pub fn new(originating_change_set_id: ChangeSetId, id: Ulid, lineage_id: Ulid) -> Self {
        Self {
            id,
            state: ActionState::Queued,
            originating_change_set_id,
            lineage_id,
            merkle_tree_hash: MerkleTreeHash::default(),
        }
    }

    pub fn content_hash(&self) -> ContentHash {
        self.node_hash()
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![]
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn set_state(&mut self, state: ActionState) {
        self.state = state;
    }

    pub fn state(&self) -> ActionState {
        self.state
    }

    pub fn originating_change_set_id(&self) -> ChangeSetId {
        self.originating_change_set_id
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn node_hash(&self) -> ContentHash {
        let mut content_hasher = ContentHash::hasher();
        content_hasher.update(&self.id.inner().to_bytes());
        content_hasher.update(&self.lineage_id.inner().to_bytes());
        content_hasher.update(self.state.to_string().as_bytes());
        content_hasher.update(self.originating_change_set_id.to_string().as_bytes());

        content_hasher.finalize()
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl From<DeprecatedActionNodeWeightV1> for ActionNodeWeight {
    fn from(value: DeprecatedActionNodeWeightV1) -> Self {
        Self {
            id: value.id,
            state: value.state,
            originating_change_set_id: value.originating_change_set_id,
            lineage_id: value.lineage_id,
            merkle_tree_hash: value.merkle_tree_hash,
        }
    }
}

impl From<&ActionNodeWeight> for NodeInformation {
    fn from(value: &ActionNodeWeight) -> Self {
        Self {
            node_weight_kind: NodeWeightDiscriminants::Action,
            id: value.id.into(),
        }
    }
}

impl CorrectTransforms for ActionNodeWeight {
    fn correct_transforms(
        &self,
        graph: &WorkspaceSnapshotGraphVCurrent,
        mut updates: Vec<Update>,
        _from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update>> {
        // An action's Use edge should be exclusive for both the component and
        // the prototype. The generic exclusive edge logic assumes there can be
        // one and only one edge of the given kind, so we have to do a custom
        // implementation here for actions, since they should have just one use
        // edge to a component and one use edge to a prototype. This is simpler
        // than rewriting all the graphs to have distinct edge kinds for action
        // prototypes and/or the components the action is for.

        struct ActionUseTargets {
            component: Option<NodeId>,
            prototype: Option<NodeId>,
        }

        let mut removal_set = HashSet::new();
        let mut new_action_use_targets = ActionUseTargets {
            component: None,
            prototype: None,
        };

        for update in &updates {
            match update {
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } if source.id == self.id().into()
                    && EdgeWeightKindDiscriminants::Use == edge_weight.kind().into() =>
                {
                    removal_set.remove(&destination.id);
                    match destination.node_weight_kind {
                        NodeWeightDiscriminants::ActionPrototype => {
                            new_action_use_targets.prototype = Some(destination.id);
                        }
                        NodeWeightDiscriminants::Component => {
                            new_action_use_targets.component = Some(destination.id);
                        }
                        // If there's a use to some other thing, ignore it.
                        // Maybe some more functionality was added. What we care
                        // about is component and prototype targets
                        _ => {}
                    }
                }
                Update::RemoveEdge {
                    source,
                    destination,
                    edge_kind,
                } if source.id == self.id().into()
                    && EdgeWeightKindDiscriminants::Use == *edge_kind =>
                {
                    removal_set.insert(destination.id);
                }
                _ => {}
            }
        }

        if let Some(node_idx) = graph.get_node_index_by_id_opt(self.id) {
            updates.extend(
                graph
                    .edges_directed(node_idx, Outgoing)
                    .filter(|edge_ref| {
                        EdgeWeightKindDiscriminants::Use == edge_ref.weight().kind().into()
                    })
                    .filter_map(|edge_ref| {
                        graph.get_node_weight_opt(edge_ref.target()).and_then(
                            |destination_weight| {
                                let should_remove = match destination_weight {
                                    NodeWeight::ActionPrototype(_) => {
                                        new_action_use_targets.prototype
                                    }
                                    NodeWeight::Component(_) => new_action_use_targets.component,
                                    _ => None,
                                }
                                .map(|new_destination_id| {
                                    new_destination_id != destination_weight.id().into()
                                        && !removal_set.contains(&destination_weight.id().into())
                                })
                                .unwrap_or(false);

                                should_remove.then_some(Update::RemoveEdge {
                                    source: self.into(),
                                    destination: destination_weight.into(),
                                    edge_kind: EdgeWeightKindDiscriminants::Use,
                                })
                            },
                        )
                    }),
            )
        }

        Ok(updates)
    }
}
