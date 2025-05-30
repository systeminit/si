use petgraph::prelude::*;
use si_events::ulid::Ulid;

use crate::{
    AttributeValueId,
    ComponentId,
    EdgeWeightKindDiscriminants,
    InputSocketId,
    NodeWeightDiscriminants,
    SchemaVariantId,
    socket::input::InputSocketError,
    workspace_snapshot::{
        graph::{
            WorkspaceSnapshotGraphResult,
            WorkspaceSnapshotGraphV4,
            traits::socket::input::InputSocketExt,
        },
        node_weight::{
            InputSocketNodeWeight,
            NodeWeight,
            NodeWeightError,
        },
    },
};

impl InputSocketExt for WorkspaceSnapshotGraphV4 {
    fn get_input_socket(
        &self,
        input_socket_id: crate::InputSocketId,
    ) -> WorkspaceSnapshotGraphResult<InputSocketNodeWeight> {
        let node_weight = self.get_node_weight_by_id(input_socket_id)?;

        match node_weight {
            NodeWeight::InputSocket(input_socket_node_weight) => {
                Ok(input_socket_node_weight.clone())
            }
            unexpected => Err(NodeWeightError::UnexpectedNodeWeightVariant(
                unexpected.into(),
                NodeWeightDiscriminants::InputSocket,
            )
            .into()),
        }
    }

    fn list_input_sockets_for_schema_variant(
        &self,
        schema_variant_id: SchemaVariantId,
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

    fn all_attribute_value_ids_everywhere_for_input_socket_id(
        &self,
        input_socket_id: InputSocketId,
    ) -> WorkspaceSnapshotGraphResult<Vec<AttributeValueId>> {
        let input_socket_idx = self.get_node_index_by_id(input_socket_id)?;
        let mut result = Vec::new();
        for (_edge_weight, av_idx, _input_socket_idx) in self.edges_directed_for_edge_weight_kind(
            input_socket_idx,
            Direction::Incoming,
            EdgeWeightKindDiscriminants::Socket,
        ) {
            if let NodeWeight::AttributeValue(av_node_weight) = self.get_node_weight(av_idx)? {
                result.push(av_node_weight.id().into());
            }
        }

        Ok(result)
    }

    fn component_attribute_value_id_for_input_socket_id(
        &self,
        input_socket_id: InputSocketId,
        component_id: ComponentId,
    ) -> WorkspaceSnapshotGraphResult<AttributeValueId> {
        let mut result = None;
        let component_idx = self.get_node_index_by_id(Ulid::from(component_id))?;
        let input_socket_idx = self.get_node_index_by_id(Ulid::from(input_socket_id))?;

        for (_edge_weight, _component_idx, av_idx) in self.edges_directed_for_edge_weight_kind(
            component_idx,
            Direction::Outgoing,
            EdgeWeightKindDiscriminants::SocketValue,
        ) {
            for (_edge_weight, _av_idx, socket_idx) in self.edges_directed_for_edge_weight_kind(
                av_idx,
                Direction::Outgoing,
                EdgeWeightKindDiscriminants::Socket,
            ) {
                if socket_idx == input_socket_idx {
                    if result.is_some() {
                        return Err(Box::new(InputSocketError::FoundTooManyForInputSocketId(
                            input_socket_id,
                            component_id,
                        ))
                        .into());
                    }
                    result = Some(self.get_node_weight(av_idx)?.id().into());
                }
            }
        }

        if let Some(av_id) = result {
            Ok(av_id)
        } else {
            Err(
                Box::new(InputSocketError::MissingAttributeValueForComponent(
                    input_socket_id,
                    component_id,
                ))
                .into(),
            )
        }
    }

    fn input_socket_id_find_for_attribute_value_id(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> WorkspaceSnapshotGraphResult<Option<InputSocketId>> {
        let mut result = None;
        let mut found_sockets = false;
        let attribute_value_idx = self.get_node_index_by_id(Ulid::from(attribute_value_id))?;

        for (_edge_weight, _av_idx, socket_idx) in self.edges_directed_for_edge_weight_kind(
            attribute_value_idx,
            Direction::Outgoing,
            EdgeWeightKindDiscriminants::Socket,
        ) {
            if found_sockets {
                return Err(Box::new(InputSocketError::MultipleSocketsForAttributeValue(
                    attribute_value_id,
                ))
                .into());
            }
            found_sockets = true;

            let socket_node_weight = self.get_node_weight(socket_idx)?;
            if NodeWeightDiscriminants::InputSocket == socket_node_weight.into() {
                result = Some(socket_node_weight.id().into());
            }
        }

        Ok(result)
    }
}
