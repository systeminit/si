use std::collections::HashSet;

use petgraph::prelude::*;
use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use super::traits::CorrectTransformsError;
use super::{NodeWeight, NodeWeightDiscriminants, NodeWeightError};
use crate::workspace_snapshot::graph::deprecated::v1::DeprecatedOrderingNodeWeightV1;
use crate::workspace_snapshot::graph::detector::Update;
use crate::workspace_snapshot::node_weight::traits::{CorrectTransforms, CorrectTransformsResult};
use crate::workspace_snapshot::node_weight::NodeWeightResult;
use crate::workspace_snapshot::NodeInformation;
use crate::{EdgeWeightKind, EdgeWeightKindDiscriminants, WorkspaceSnapshotGraphVCurrent};

#[derive(Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct OrderingNodeWeight {
    pub id: Ulid,
    pub lineage_id: Ulid,
    /// The `id` of the items, in the order that they should appear in the container.
    order: Vec<Ulid>,
    merkle_tree_hash: MerkleTreeHash,
}

impl OrderingNodeWeight {
    pub fn content_hash(&self) -> ContentHash {
        self.node_hash()
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![]
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

    pub fn new(id: Ulid, lineage_id: Ulid) -> Self {
        Self {
            id,
            lineage_id,
            ..Default::default()
        }
    }

    pub fn node_hash(&self) -> ContentHash {
        let mut content_hasher = ContentHash::hasher();
        for id in &self.order {
            let bytes = id.inner().to_bytes();
            content_hasher.update(&bytes);
        }

        content_hasher.finalize()
    }

    pub fn order(&self) -> &Vec<Ulid> {
        &self.order
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub fn set_order(&mut self, order: Vec<Ulid>) {
        self.order = order;
    }

    pub fn push_to_order(&mut self, id: Ulid) {
        self.order.push(id);
    }

    /// Returns `true` if the id passed was actually removed, `false` if not (because not in the order)
    pub fn remove_from_order(&mut self, id: Ulid) -> bool {
        let order_len = self.order.len();
        self.order.retain(|&item_id| item_id != id);
        order_len != self.order().len()
    }

    pub fn get_index_for_id(&self, id: Ulid) -> NodeWeightResult<i64> {
        let index = &self
            .order
            .iter()
            .position(|&key| key == id)
            .ok_or(NodeWeightError::MissingKeyForChildEntry(id))?;

        let ret: i64 = (*index)
            .try_into()
            .map_err(NodeWeightError::TryFromIntError)?;
        Ok(ret)
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl std::fmt::Debug for OrderingNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("OrderingNodeWeight")
            .field("id", &self.id.to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field(
                "order",
                &self
                    .order
                    .iter()
                    .map(|id| id.to_string())
                    .collect::<Vec<String>>(),
            )
            .field("content_hash", &self.content_hash())
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .finish()
    }
}

impl From<DeprecatedOrderingNodeWeightV1> for OrderingNodeWeight {
    fn from(value: DeprecatedOrderingNodeWeightV1) -> Self {
        Self {
            id: value.id,
            lineage_id: value.lineage_id,
            order: value.order,
            merkle_tree_hash: value.merkle_tree_hash,
        }
    }
}

impl From<&OrderingNodeWeight> for NodeInformation {
    fn from(value: &OrderingNodeWeight) -> Self {
        Self {
            node_weight_kind: NodeWeightDiscriminants::Ordering,
            id: value.id.into(),
        }
    }
}

impl CorrectTransforms for OrderingNodeWeight {
    fn correct_transforms(
        &self,
        graph: &WorkspaceSnapshotGraphVCurrent,
        updates: Vec<Update>,
        _from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update>> {
        let mut updates = updates;

        // We need to handle the key conflicts for attribute value Contain edges
        // at the same time that we handle ordering conflicts, since we have to
        // be sure to remove the duplicate target from the AV's order, but also
        // preserve any other ordering changes that have come in from another
        // change set
        let maybe_attribute_value_container: Option<(NodeIndex, Ulid, NodeInformation)> = graph
            .get_node_index_by_id_opt(self.id)
            .and_then(|self_idx| {
                graph
                    .edges_directed(self_idx, Incoming)
                    .filter(|edge_ref| edge_ref.weight().kind() == &EdgeWeightKind::Ordering)
                    .filter_map(|edge_ref| {
                        graph
                            .get_node_weight_opt(edge_ref.source())
                            .and_then(|source_weight| match source_weight {
                                NodeWeight::AttributeValue(_) => Some((
                                    edge_ref.source(),
                                    source_weight.id(),
                                    source_weight.into(),
                                )),
                                _ => None,
                            })
                    })
                    .next()
            });

        //
        // After this, final_children:
        // - includes all nodes that *either* had an AddEdge, *or* are in our graph's children.
        // - never includes nodes with a RemoveEdge or in our graph's children.
        // - NOTE: if the same edge has both an AddEdge and RemoveEdge, the above is not true.
        //
        let mut final_children: HashSet<Ulid> = self.order.iter().copied().collect();
        let mut replace_node_index = None;
        let mut new_av_contains = HashSet::new();
        for (index, update) in updates.iter().enumerate() {
            match update {
                // We don't do this for NewNode because we know nothing needs to be resolved.
                Update::ReplaceNode { node_weight } if node_weight.id() == self.id => {
                    replace_node_index = Some(index);
                }
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } => {
                    if source.id == self.id.into() && edge_weight.kind() == &EdgeWeightKind::Ordinal
                    {
                        final_children.insert(destination.id.into());
                    } else if let Some((_, av_container_id, _)) = maybe_attribute_value_container {
                        if source.id == av_container_id.into() {
                            if let EdgeWeightKind::Contain(Some(new_key)) = edge_weight.kind() {
                                new_av_contains.insert(new_key);
                            }
                        }
                    }
                }
                Update::RemoveEdge {
                    source,
                    destination,
                    edge_kind: EdgeWeightKindDiscriminants::Ordinal,
                } if source.id == self.id.into() => {
                    final_children.remove(&destination.id.into());
                }
                _ => (),
            }
        }

        let mut remove_duplicate_contain_edge_updates = vec![];
        if let Some((av_container_idx, _, av_node_info)) = maybe_attribute_value_container {
            graph
                .edges_directed(av_container_idx, Outgoing)
                .filter(|edge_ref| match edge_ref.weight().kind() {
                    EdgeWeightKind::Contain(Some(key)) => new_av_contains.contains(key),
                    _ => false,
                })
                .filter_map(|edge_ref| graph.get_node_weight_opt(edge_ref.target()))
                .for_each(|duplicate_key_target| {
                    remove_duplicate_contain_edge_updates.push(Update::RemoveEdge {
                        source: av_node_info,
                        destination: duplicate_key_target.into(),
                        edge_kind: EdgeWeightKindDiscriminants::Contain,
                    });
                    remove_duplicate_contain_edge_updates.push(Update::RemoveEdge {
                        source: self.into(),
                        destination: duplicate_key_target.into(),
                        edge_kind: EdgeWeightKindDiscriminants::Ordinal,
                    });

                    final_children.remove(&duplicate_key_target.id());
                });
        }

        // Generally, this will only be None if this is an entirely new ordering node.
        if let Some(replace_node_index) = replace_node_index {
            match updates.get_mut(replace_node_index) {
                Some(Update::ReplaceNode {
                    node_weight: NodeWeight::Ordering(ref mut update_ordering),
                }) => {
                    let new_order =
                        resolve_ordering(final_children, &self.order, &update_ordering.order);
                    update_ordering.set_order(new_order);
                }
                _ => {
                    return Err(CorrectTransformsError::UnexpectedNodeWeight(
                        NodeWeightDiscriminants::Ordering,
                    ))
                }
            };
        }

        updates.extend(remove_duplicate_contain_edge_updates);

        Ok(updates)
    }
}

fn resolve_ordering(
    final_children: HashSet<Ulid>,
    order: &[Ulid],
    update_order: &[Ulid],
) -> Vec<Ulid> {
    let mut final_children = final_children;

    // The final order is always:
    // - in the order of the updated node
    // - without children that were removed from our graph (in updated_order, has no AddEdge, and was not in our graph)
    // - with children that were added to our graph (not in updated_order, has no RemoveEdge, and *was* in our graph)

    //
    // Grab the child ordering from the updated node. Only include elements that are
    // supposed to be part of our children. Remove any such elements from final_order,
    // so that it will only have children left if they were *added* in our graph.
    //
    let mut final_order = update_order
        .iter()
        .filter(|id| final_children.remove(id))
        .copied()
        .collect::<Vec<_>>();

    //
    // final_children now has only children that were *added* in our graph. Add them to
    // the final order, in the order they appear in our graph.
    //
    // NOTE/TODO: we could probably put these in a better order theoretically, but that's
    // more complexity and work than it's worth for what we would buy (at least right now).
    // new_order and final_children now have the same set of children.
    //
    let added_children = final_children;
    final_order.extend(order.iter().filter(|id| added_children.contains(id)));

    final_order
}
