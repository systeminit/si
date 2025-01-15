use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use crate::{
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::{
            correct_transforms::add_dependent_value_root_updates,
            deprecated::v1::DeprecatedAttributeValueNodeWeightV1, detector::Update, LineageId,
        },
        node_weight::traits::CorrectTransforms,
        NodeId,
    },
    EdgeWeightKindDiscriminants, WorkspaceSnapshotGraphVCurrent,
};

use super::{category_node_weight::CategoryNodeKind, traits::CorrectTransformsResult, NodeWeight};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AttributeValueNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    /// The unprocessed return value is the "real" result, unprocessed for any other behavior.
    /// This is potentially-maybe-only-kinda-sort-of(?) useful for non-scalar values.
    /// Example: a populated array.
    unprocessed_value: Option<ContentAddress>,
    /// The processed return value.
    /// Example: empty array.
    value: Option<ContentAddress>,
}

impl AttributeValueNodeWeight {
    pub fn new(
        id: Ulid,
        lineage_id: Ulid,
        unprocessed_value: Option<ContentAddress>,
        value: Option<ContentAddress>,
    ) -> Self {
        Self {
            id,
            lineage_id,
            merkle_tree_hash: MerkleTreeHash::default(),
            unprocessed_value,
            value,
        }
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        let mut hashes = vec![];

        if let Some(hash) = self.unprocessed_value {
            hashes.push(hash.content_hash());
        }
        if let Some(hash) = self.value {
            hashes.push(hash.content_hash());
        }

        hashes
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn unprocessed_value(&self) -> Option<ContentAddress> {
        self.unprocessed_value
    }

    pub fn set_unprocessed_value(&mut self, unprocessed_value: Option<ContentAddress>) {
        self.unprocessed_value = unprocessed_value
    }

    pub fn value(&self) -> Option<ContentAddress> {
        self.value
    }

    pub fn set_value(&mut self, value: Option<ContentAddress>) {
        self.value = value
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn content_hash(&self) -> ContentHash {
        self.node_hash()
    }

    pub fn node_hash(&self) -> ContentHash {
        let mut content_hasher = ContentHash::hasher();
        content_hasher.update(
            &self
                .unprocessed_value
                .map(|v| v.content_hash().as_bytes().to_owned())
                .unwrap_or_else(|| vec![0x00]),
        );
        content_hasher.update(
            &self
                .value
                .map(|v| v.content_hash().as_bytes().to_owned())
                .unwrap_or_else(|| vec![0x00]),
        );

        content_hasher.finalize()
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[
            EdgeWeightKindDiscriminants::Prototype,
            EdgeWeightKindDiscriminants::Prop,
            EdgeWeightKindDiscriminants::Socket,
        ]
    }
}

impl std::fmt::Debug for AttributeValueNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("AttributeValueNodeWeight")
            .field("id", &self.id().to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("value", &self.value)
            .field("unprocessed_value", &self.unprocessed_value)
            .field("node_hash", &self.node_hash())
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .finish()
    }
}

impl From<DeprecatedAttributeValueNodeWeightV1> for AttributeValueNodeWeight {
    fn from(value: DeprecatedAttributeValueNodeWeightV1) -> Self {
        Self {
            id: value.id,
            lineage_id: value.lineage_id,
            merkle_tree_hash: value.merkle_tree_hash,
            unprocessed_value: value.unprocessed_value,
            value: value.value,
        }
    }
}

impl CorrectTransforms for AttributeValueNodeWeight {
    fn correct_transforms(
        &self,
        graph: &WorkspaceSnapshotGraphVCurrent,
        mut updates: Vec<Update>,
        from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update>> {
        if !from_different_change_set {
            return Ok(updates);
        }

        let dvu_cat_node_id: Option<NodeId> = graph
            .get_category_node(None, CategoryNodeKind::DependentValueRoots)?
            .map(|(id, _)| id.into());

        let mut should_add = false;

        for update in &updates {
            match update {
                Update::NewEdge {
                    source,
                    edge_weight,
                    ..
                } if source.id == self.id().into()
                    && EdgeWeightKindDiscriminants::Prototype == edge_weight.kind().into() =>
                {
                    should_add = graph.get_node_weight_by_id_opt(source.id).is_some();
                }
                Update::RemoveEdge { source, .. } if Some(source.id) == dvu_cat_node_id => {
                    // If there is a remove edge from the dvu root then we are the result of a DVU
                    // job finishing and we should *not* re-enqueue any updates or we will
                    // potentially loop forever
                    return Ok(updates);
                }
                Update::ReplaceNode { node_weight } if node_weight.id() == self.id() => {
                    should_add = graph
                        .get_node_weight_by_id_opt(self.id())
                        .is_some_and(|weight| weight.node_hash() != node_weight.node_hash());
                }
                Update::NewNode {
                    node_weight: NodeWeight::DependentValueRoot(inner),
                } if inner.value_id() == self.id() => {
                    return Ok(updates);
                }
                _ => {}
            }
        }

        if should_add {
            updates.extend(add_dependent_value_root_updates(
                graph,
                &HashSet::from([self.id()]),
            )?);
        }

        Ok(updates)
    }
}
