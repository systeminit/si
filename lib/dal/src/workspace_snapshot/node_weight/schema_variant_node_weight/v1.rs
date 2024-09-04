use std::sync::Arc;

use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use super::{SchemaVariantNodeWeight, SchemaVariantNodeWeightError, SchemaVariantNodeWeightResult};
use crate::{
    layer_db_types::{SchemaVariantContent, SchemaVariantContentV3},
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::LineageId,
        node_weight::{
            traits::{CorrectExclusiveOutgoingEdge, CorrectTransforms, SiNodeWeight},
            ContentNodeWeight, NodeWeight, NodeWeightDiscriminants, NodeWeightError,
        },
        ContentAddressDiscriminants,
    },
    DalContext, EdgeWeightKindDiscriminants, Timestamp, WorkspaceSnapshotGraphV3,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SchemaVariantNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    is_locked: bool,
    content_address: ContentAddress,
    timestamp: Timestamp,
}

impl SchemaVariantNodeWeightV1 {
    pub fn new(id: Ulid, lineage_id: Ulid, is_locked: bool, content_hash: ContentHash) -> Self {
        Self {
            id,
            lineage_id,
            is_locked,
            content_address: ContentAddress::SchemaVariant(content_hash),
            merkle_tree_hash: MerkleTreeHash::default(),
            timestamp: Timestamp::now(),
        }
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    pub fn set_is_locked(&mut self, new_locked: bool) {
        self.is_locked = new_locked;
    }

    pub fn new_content_hash(
        &mut self,
        new_content_hash: ContentHash,
    ) -> SchemaVariantNodeWeightResult<()> {
        self.content_address = ContentAddress::SchemaVariant(new_content_hash);

        Ok(())
    }

    pub(crate) async fn try_upgrade_from_content_node_weight(
        ctx: &DalContext,
        v3_graph: &mut WorkspaceSnapshotGraphV3,
        content_node_weight: &ContentNodeWeight,
    ) -> SchemaVariantNodeWeightResult<()> {
        let content_hash = if let ContentAddress::SchemaVariant(content_hash) =
            content_node_weight.content_address()
        {
            content_hash
        } else {
            return Err(Box::new(NodeWeightError::UnexpectedContentAddressVariant(
                ContentAddressDiscriminants::SchemaVariant,
                content_node_weight.content_address_discriminants(),
            ))
            .into());
        };

        let content: SchemaVariantContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&content_hash)
            .await?
            .ok_or_else(|| {
                Box::new(NodeWeightError::MissingContentFromStore(
                    content_node_weight.id(),
                ))
            })?;

        let (v3_content, is_locked) = match content {
            SchemaVariantContent::V1(old_content) => {
                let v3_content = SchemaVariantContentV3 {
                    timestamp: old_content.timestamp,
                    ui_hidden: old_content.ui_hidden,
                    version: old_content.timestamp.created_at.to_string(),
                    display_name: old_content
                        .display_name
                        .unwrap_or_else(|| content_node_weight.id().to_string()),
                    category: old_content.category,
                    color: old_content.color,
                    component_type: old_content.component_type,
                    link: old_content.link,
                    description: old_content.description,
                    asset_func_id: old_content.asset_func_id,
                    finalized_once: old_content.finalized_once,
                    is_builtin: old_content.is_builtin,
                };

                // Locking variants didn't exist at this point, so everything should be considered
                // as locked.
                (v3_content, true)
            }
            SchemaVariantContent::V2(old_content) => {
                let v3_content = SchemaVariantContentV3 {
                    timestamp: old_content.timestamp,
                    ui_hidden: old_content.ui_hidden,
                    version: old_content.version,
                    display_name: old_content.display_name,
                    category: old_content.category,
                    color: old_content.color,
                    component_type: old_content.component_type,
                    link: old_content.link,
                    description: old_content.description,
                    asset_func_id: old_content.asset_func_id,
                    finalized_once: old_content.finalized_once,
                    is_builtin: old_content.is_builtin,
                };

                (v3_content, old_content.is_locked)
            }
            SchemaVariantContent::V3(_) => {
                return Err(SchemaVariantNodeWeightError::InvalidContentForNodeWeight(
                    content_node_weight.id(),
                ));
            }
        };

        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(SchemaVariantContent::V3(v3_content).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let new_node_weight_inner = Self::new(
            content_node_weight.id(),
            content_node_weight.lineage_id(),
            is_locked,
            hash,
        );
        let new_node_weight =
            NodeWeight::SchemaVariant(SchemaVariantNodeWeight::V1(new_node_weight_inner));

        v3_graph
            .add_or_replace_node(new_node_weight)
            .map_err(Box::new)?;

        Ok(())
    }
}

impl SiNodeWeight for SchemaVariantNodeWeightV1 {
    fn content_hash(&self) -> ContentHash {
        self.content_address.content_hash()
    }

    fn id(&self) -> Ulid {
        self.id
    }

    fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    fn node_hash(&self) -> ContentHash {
        let mut content_hasher = ContentHash::hasher();
        content_hasher.update(if self.is_locked { &[0x01] } else { &[0x00] });
        content_hasher.update(self.content_address.to_string().as_bytes());

        content_hasher.finalize()
    }

    fn node_weight_discriminant(&self) -> NodeWeightDiscriminants {
        NodeWeightDiscriminants::SchemaVariant
    }

    fn set_id(&mut self, new_id: Ulid) {
        self.id = new_id;
    }

    fn set_lineage_id(&mut self, new_id: Ulid) {
        self.lineage_id = new_id;
    }

    fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash
    }
}

impl CorrectTransforms for SchemaVariantNodeWeightV1 {}

impl CorrectExclusiveOutgoingEdge for SchemaVariantNodeWeightV1 {
    fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}
