use std::collections::{
    BTreeSet,
    HashSet,
};

use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    EncryptedSecretKey,
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};
use si_split_graph::{
    SplitGraph,
    SplitGraphNodeWeight,
};

use super::{
    NodeWeight,
    category_node_weight::CategoryNodeKind,
    traits::CorrectTransformsResult,
};
use crate::{
    EdgeWeight,
    EdgeWeightKindDiscriminants,
    WorkspaceSnapshotGraphVCurrent,
    workspace_snapshot::{
        NodeId,
        content_address::{
            ContentAddress,
            ContentAddressDiscriminants,
        },
        graph::{
            LineageId,
            correct_transforms,
            detector::Update,
        },
        node_weight::{
            NodeWeightError,
            NodeWeightResult,
            traits::CorrectTransforms,
        },
        split_snapshot::{
            self,
            corrections::get_category_node_id,
        },
    },
};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct SecretNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    content_address: ContentAddress,
    merkle_tree_hash: MerkleTreeHash,
    encrypted_secret_key: EncryptedSecretKey,
}

impl SecretNodeWeight {
    pub fn new(
        id: Ulid,
        lineage_id: Ulid,
        content_address: ContentAddress,
        encrypted_secret_key: EncryptedSecretKey,
    ) -> Self {
        Self {
            id,
            lineage_id,
            content_address,
            merkle_tree_hash: MerkleTreeHash::default(),
            encrypted_secret_key,
        }
    }

    pub fn content_address(&self) -> ContentAddress {
        self.content_address
    }

    pub fn content_hash(&self) -> ContentHash {
        self.content_address.content_hash()
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![self.content_address.content_hash()]
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

    pub fn encrypted_secret_key(&self) -> EncryptedSecretKey {
        self.encrypted_secret_key
    }

    pub fn set_encrypted_secret_key(
        &mut self,
        encrypted_secret_key: EncryptedSecretKey,
    ) -> &mut Self {
        self.encrypted_secret_key = encrypted_secret_key;
        self
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        let new_address = match &self.content_address {
            ContentAddress::Secret(_) => ContentAddress::Secret(content_hash),
            other => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    Into::<ContentAddressDiscriminants>::into(other).to_string(),
                    ContentAddressDiscriminants::Secret.to_string(),
                ));
            }
        };

        self.content_address = new_address;

        Ok(())
    }

    pub fn node_hash(&self) -> ContentHash {
        let mut content_hasher = ContentHash::hasher();
        content_hasher.update(self.content_address.content_hash().as_bytes());
        content_hasher.update(self.encrypted_secret_key.as_bytes());

        content_hasher.finalize()
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl std::fmt::Debug for SecretNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("SecretNodeWeight")
            .field("id", &self.id().to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("content_hash", &self.content_hash())
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .field("encrypted_secret_key", &self.encrypted_secret_key)
            .finish()
    }
}

impl CorrectTransforms for SecretNodeWeight {
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
                Update::RemoveEdge { source, .. } if Some(source.id) == dvu_cat_node_id => {
                    // If there is a remove edge from the dvu root then we are the result of a DVU
                    // job finishing and we should *not* re-enqueue any updates or we will
                    // potentially loop forever
                    return Ok(updates);
                }
                Update::ReplaceNode { node_weight } if node_weight.id() == self.id() => {
                    // Only add the secret here if the secret has actually changed (this may be an
                    // update that does not change anything)
                    if let NodeWeight::Secret(updated_secret) = node_weight {
                        should_add =
                            graph
                                .get_node_weight_by_id_opt(self.id())
                                .is_some_and(|secret| match secret {
                                    NodeWeight::Secret(inner) => {
                                        inner.encrypted_secret_key()
                                            != updated_secret.encrypted_secret_key()
                                    }
                                    _ => false,
                                })
                    }
                }
                Update::NewNode { node_weight } => match node_weight {
                    NodeWeight::DependentValueRoot(inner) => {
                        // Are we already going to calculate a dvu for this?
                        if inner.value_id() == self.id() {
                            return Ok(updates);
                        }
                    }
                    NodeWeight::Secret(_) if node_weight.id() == self.id() => {
                        // Only add the secret here if the node is actually new
                        should_add = graph.get_node_weight_by_id_opt(self.id()).is_none();
                    }
                    _ => {}
                },
                _ => {}
            }
        }

        if should_add {
            updates.extend(correct_transforms::add_dependent_value_root_updates(
                graph,
                &HashSet::from([self.id()]),
            )?);
        }

        Ok(updates)
    }
}

impl
    split_snapshot::corrections::CorrectTransforms<
        NodeWeight,
        EdgeWeight,
        EdgeWeightKindDiscriminants,
    > for SecretNodeWeight
{
    fn correct_transforms(
        &self,
        graph: &SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
        mut updates: Vec<
            si_split_graph::Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
        >,
        from_different_change_set: bool,
    ) -> split_snapshot::corrections::CorrectTransformsResult<
        Vec<si_split_graph::Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
    > {
        if !from_different_change_set {
            return Ok(updates);
        }

        let dvu_cat_id = get_category_node_id(graph, CategoryNodeKind::DependentValueRoots)?;
        let mut should_add = false;

        for update in &updates {
            match update {
                si_split_graph::Update::RemoveEdge { .. } if update.source_id() == dvu_cat_id => {
                    return Ok(updates);
                }
                si_split_graph::Update::NewNode { node_weight, .. } => {
                    if let SplitGraphNodeWeight::Custom(NodeWeight::DependentValueRoot(
                        dvu_root_inner,
                    )) = node_weight
                    {
                        // This is already a DVU root, so nothing to do
                        if dvu_root_inner.value_id() == self.id() {
                            return Ok(updates);
                        }
                    } else if let SplitGraphNodeWeight::Custom(NodeWeight::Secret(_)) = node_weight
                    {
                        should_add = node_weight.id() == self.id();
                    }
                }
                si_split_graph::Update::ReplaceNode { node_weight, .. }
                    if node_weight.id() == self.id() =>
                {
                    if let SplitGraphNodeWeight::Custom(NodeWeight::Secret(updated_secret)) =
                        node_weight
                    {
                        should_add =
                            graph
                                .node_weight(self.id())
                                .is_some_and(|secret| match secret {
                                    NodeWeight::Secret(inner) => {
                                        inner.encrypted_secret_key()
                                            != updated_secret.encrypted_secret_key()
                                    }
                                    _ => false,
                                });
                    }
                }

                _ => {}
            }
        }

        if should_add {
            updates.extend(
                split_snapshot::corrections::add_dependent_value_root_updates(
                    graph,
                    &BTreeSet::from([self.id()]),
                )?,
            );
        }

        Ok(updates)
    }
}
