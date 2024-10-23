use std::sync::Arc;

use serde::{Deserialize, Serialize};
use si_events::{merkle_tree_hash::MerkleTreeHash, ulid::Ulid, ContentHash};

use super::{InputSocketNodeWeight, InputSocketNodeWeightError, InputSocketNodeWeightResult};
use crate::workspace_snapshot::graph::WorkspaceSnapshotGraphV3;
use crate::{
    layer_db_types::{InputSocketContent, InputSocketContentV2},
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::LineageId,
        node_weight::{
            traits::{CorrectExclusiveOutgoingEdge, CorrectTransforms, SiNodeWeight},
            ContentNodeWeight, NodeWeight, NodeWeightDiscriminants, NodeWeightError,
        },
        ContentAddressDiscriminants,
    },
    DalContext, EdgeWeightKindDiscriminants, SocketArity, Timestamp,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, dal_macros::SiNodeWeight)]
#[si_node_weight(discriminant = NodeWeightDiscriminants::InputSocket)]
pub struct InputSocketNodeWeightV1 {
    pub id: Ulid,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
    #[si_node_weight(node_hash = "self.arity.to_string().as_bytes()")]
    arity: SocketArity,
    #[si_node_weight(node_hash = "self.content_address.content_hash().as_bytes()")]
    content_address: ContentAddress,
    timestamp: Timestamp,
}

impl InputSocketNodeWeightV1 {
    pub fn arity(&self) -> SocketArity {
        self.arity
    }

    pub fn new(id: Ulid, lineage_id: Ulid, arity: SocketArity, content_hash: ContentHash) -> Self {
        Self {
            id,
            lineage_id,
            arity,
            content_address: ContentAddress::InputSocket(content_hash),
            merkle_tree_hash: MerkleTreeHash::default(),
            timestamp: Timestamp::now(),
        }
    }

    pub fn new_content_hash(&mut self, new_content_hash: ContentHash) {
        self.content_address = ContentAddress::InputSocket(new_content_hash);
    }

    pub(crate) async fn try_upgrade_from_content_node_weight(
        ctx: &DalContext,
        v3_graph: &mut WorkspaceSnapshotGraphV3,
        content_node_weight: &ContentNodeWeight,
    ) -> InputSocketNodeWeightResult<()> {
        let content_hash = if let ContentAddress::InputSocket(content_hash) =
            content_node_weight.content_address()
        {
            content_hash
        } else {
            return Err(Box::new(NodeWeightError::UnexpectedContentAddressVariant(
                ContentAddressDiscriminants::InputSocket,
                content_node_weight.content_address_discriminants(),
            ))
            .into());
        };

        let content: InputSocketContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&content_hash)
            .await?
            .ok_or_else(|| {
                Box::new(NodeWeightError::MissingContentFromStore(
                    content_node_weight.id(),
                ))
            })?;

        let (v2_content, arity) = match content {
            InputSocketContent::V1(old_content) => {
                let v2_content = InputSocketContentV2 {
                    timestamp: old_content.timestamp,
                    name: old_content.name.clone(),
                    inbound_type_definition: old_content.inbound_type_definition.clone(),
                    outbound_type_definition: old_content.outbound_type_definition.clone(),
                    kind: old_content.kind,
                    required: old_content.required,
                    ui_hidden: old_content.ui_hidden,
                    connection_annotations: old_content.connection_annotations.clone(),
                };

                (v2_content, old_content.arity)
            }
            // InputSocketContent::V2 was never stored inside a NodeWeight::Content, and doesn't
            // have all the required information on its own to generate an InputSocketNodeWeight.
            InputSocketContent::V2(_) => {
                return Err(InputSocketNodeWeightError::InvalidContentForNodeWeight(
                    content_node_weight.id(),
                ));
            }
        };

        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(InputSocketContent::V2(v2_content).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let new_node_weight_inner = Self::new(
            content_node_weight.id(),
            content_node_weight.lineage_id(),
            arity,
            hash,
        );

        let new_node_weight =
            NodeWeight::InputSocket(InputSocketNodeWeight::V1(new_node_weight_inner));

        v3_graph
            .add_or_replace_node(new_node_weight)
            .map_err(Box::new)?;

        Ok(())
    }
}

impl CorrectTransforms for InputSocketNodeWeightV1 {}

impl CorrectExclusiveOutgoingEdge for InputSocketNodeWeightV1 {
    fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}
