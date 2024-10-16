use petgraph::prelude::*;
use si_events::ContentHash;

use crate::{
    workspace_snapshot::{
        graph::{traits::socket::input::InputSocketExt, LineageId, WorkspaceSnapshotGraphResult},
        node_weight::{InputSocketNodeWeight, NodeWeight, NodeWeightError},
    },
    EdgeWeightKindDiscriminants, InputSocketId, NodeWeightDiscriminants, SchemaVariantId,
    SocketArity, WorkspaceSnapshotGraphV3,
};

impl InputSocketExt for WorkspaceSnapshotGraphV3 {
    fn new_input_socket(
        &mut self,
        schema_variant_id: SchemaVariantId,
        input_socket_id: InputSocketId,
        lineage_id: LineageId,
        arity: SocketArity,
        content_hash: ContentHash,
    ) -> WorkspaceSnapshotGraphResult<InputSocketNodeWeight> {
        let node_weight =
            NodeWeight::new_input_socket(input_socket_id.into(), lineage_id, arity, content_hash);
        self.add_or_replace_node(node_weight);

        self.get_input_socket(input_socket_id)
    }

    fn get_input_socket(
        &self,
        input_socket_id: crate::InputSocketId,
    ) -> WorkspaceSnapshotGraphResult<InputSocketNodeWeight> {
        let node_weight = self.get_node_weight_by_id(input_socket_id)?;

        match node_weight {
            NodeWeight::InputSocket(input_socket_node_weight) => Ok(input_socket_node_weight),
            unexpected => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                unexpected.into(),
                NodeWeightDiscriminants::InputSocket,
            )
            .into()),
        }
    }

    fn list_input_sockets_for_schema_variant(
        &self,
        schema_variant_id: crate::SchemaVariantId,
    ) -> WorkspaceSnapshotGraphResult<Vec<InputSocketNodeWeight>> {
        let schema_variant_index = self.get_node_index_by_id(schema_variant_id)?;

        let mut input_sockets = Vec::new();
        for (_edge_weight, _schema_variant_index, socket_index) in self
            .edges_directed_for_edge_weight_kind(
                schema_variant_index,
                Direction::Outgoing,
                EdgeWeightKindDiscriminants::Socket,
            )
        {
            let socket_node_weight = self.get_node_weight(socket_index)?;
            match socket_node_weight {
                NodeWeight::InputSocket(input_socket_node_weight) => {
                    input_sockets.push(input_socket_node_weight.clone())
                }
                _ => continue,
            }
        }

        Ok(input_sockets)
    }
}
