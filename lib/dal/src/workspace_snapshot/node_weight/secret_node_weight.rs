use std::collections::HashSet;

use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash, EncryptedSecretKey};

use crate::{layer_db_types::SecretContent, workspace_snapshot::{content_address::ContentAddress, graph::{correct_transforms::add_dependent_value_root_updates, deprecated::v1::DeprecatedSecretNodeWeightV1, detect_updates::Update, LineageId}, node_weight::{category_node_weight::CategoryNodeKind, impl_has_content, traits::{CorrectTransforms, CorrectTransformsResult}, NodeHash, NodeWeight}, NodeId}, EdgeWeightKindDiscriminants, WorkspaceSnapshotGraphV2};

use super::HasContentHash as _;

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SecretNodeWeight {
    pub id: Ulid,
    pub lineage_id: LineageId,
    content_address: ContentAddress,
    pub(super) merkle_tree_hash: MerkleTreeHash,
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

    pub fn id(&self) -> Ulid {
        self.id
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

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl NodeHash for SecretNodeWeight {
    fn node_hash(&self) -> ContentHash {
        ContentHash::from(&serde_json::json![{
            "content_address": self.content_address,
            "encrypted_secret_key": self.encrypted_secret_key,
        }])
    }
}

impl_has_content! { SecretNodeWeight => SecretContent }

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

impl From<DeprecatedSecretNodeWeightV1> for SecretNodeWeight {
    fn from(value: DeprecatedSecretNodeWeightV1) -> Self {
        Self {
            id: value.id,
            lineage_id: value.lineage_id,
            content_address: value.content_address,
            merkle_tree_hash: value.merkle_tree_hash,
            encrypted_secret_key: value.encrypted_secret_key,
        }
    }
}

impl CorrectTransforms for SecretNodeWeight {
    fn correct_transforms(
        &self,
        graph: &WorkspaceSnapshotGraphV2,
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
            updates.extend(add_dependent_value_root_updates(
                graph,
                &HashSet::from([self.id()]),
            )?);
        }

        Ok(updates)
    }
}
