use petgraph::{prelude::*, stable_graph::EdgeReference};
use si_id::AttributePrototypeArgumentId;

use crate::{
    workspace_snapshot::{
        content_address::ContentAddress,
        graph::{
            traits::attribute_prototype::argument::{ArgumentValue, AttributePrototypeArgumentExt},
            WorkspaceSnapshotGraphError, WorkspaceSnapshotGraphResult, WorkspaceSnapshotGraphV4,
        },
        node_weight::{traits::SiNodeWeight, NodeWeight},
    },
    EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants,
};

impl AttributePrototypeArgumentExt for WorkspaceSnapshotGraphV4 {
    fn argument_value(
        &self,
        apa_id: AttributePrototypeArgumentId,
    ) -> WorkspaceSnapshotGraphResult<Option<ArgumentValue>> {
        let apa_index = self.get_node_index_by_id(apa_id)?;
        let mut argument_values = self
            .edges_directed(apa_index, Direction::Outgoing)
            .filter_map(|edge| argument_value_for_edge(self, edge).transpose());
        let Some(argument_value) = argument_values.next() else {
            return Ok(None);
            // TODO this really should be an error
            // NOTE: this could be either PrototypeArgumentValue or PrototypeArgumentValueSubscription edges
            // return Err(WorkspaceSnapshotGraphError::NoEdgesOfKindFound(
            //     apa_index,
            //     EdgeWeightKindDiscriminants::PrototypeArgumentValue,
            // ));
        };
        if argument_values.next().is_some() {
            // NOTE: this could be either PrototypeArgumentValue or PrototypeArgumentValueSubscription edges
            return Err(WorkspaceSnapshotGraphError::TooManyEdgesOfKind(
                apa_index,
                EdgeWeightKindDiscriminants::PrototypeArgumentValue,
            ));
        }
        Ok(Some(argument_value?))
    }
}

fn argument_value_for_edge(
    graph: &WorkspaceSnapshotGraphV4,
    edge: EdgeReference<EdgeWeight>,
) -> WorkspaceSnapshotGraphResult<Option<ArgumentValue>> {
    Ok(Some(match edge.weight().kind() {
        // Handle APA -- PrototypeArgumentValue -> Prop/Secret/InputSocket/OutputSocket/StaticArgumentValue
        &EdgeWeightKind::PrototypeArgumentValue => match graph.get_node_weight(edge.target())? {
            NodeWeight::Prop(ref node) => ArgumentValue::Prop(node.id().into()),
            NodeWeight::Secret(ref node) => ArgumentValue::Secret(node.id().into()),
            NodeWeight::Content(ref node) => match node.content_address() {
                ContentAddress::InputSocket(..) => ArgumentValue::InputSocket(node.id().into()),
                ContentAddress::OutputSocket(..) => ArgumentValue::OutputSocket(node.id().into()),
                ContentAddress::StaticArgumentValue(..) => {
                    ArgumentValue::StaticArgumentValue(node.id().into())
                }
                other => {
                    return Err(WorkspaceSnapshotGraphError::UnexpectedValueSourceContent(
                        edge.source(),
                        other.into(),
                    ));
                }
            },
            NodeWeight::InputSocket(ref node) => ArgumentValue::InputSocket(node.id().into()),
            _ => {
                return Err(WorkspaceSnapshotGraphError::UnexpectedValueSourceNode(
                    edge.source(),
                    edge.weight().kind().into(),
                ));
            }
        },

        // Handle APA -- PrototypeArgumentValueSubscription(json_pointer) -> Component
        &EdgeWeightKind::PrototypeArgumentValueSubscription(json_pointer) => {
            match graph.get_node_weight(edge.target())? {
                NodeWeight::Component(ref node) => ArgumentValue::AttributeValueSubscription {
                    component_id: node.id().into(),
                    json_pointer,
                },
                _ => {
                    return Err(WorkspaceSnapshotGraphError::UnexpectedValueSourceNode(
                        edge.source(),
                        edge.weight().kind().into(),
                    ))
                }
            }
        }
        _ => return Ok(None),
    }))
}
