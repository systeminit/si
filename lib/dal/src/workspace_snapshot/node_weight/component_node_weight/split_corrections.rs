use std::collections::{
    BTreeSet,
    HashSet,
};

use petgraph::Direction::{
    Incoming,
    Outgoing,
};
use si_id::{
    ComponentId,
    InputSocketId,
    OutputSocketId,
};
use si_split_graph::{
    SplitGraph,
    SplitGraphNodeId,
    Update,
};

use super::ComponentNodeWeight;
use crate::{
    EdgeWeight,
    EdgeWeightKindDiscriminants,
    NodeWeightDiscriminants,
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        node_weight::{
            ArgumentTargets,
            NodeWeight,
        },
        split_snapshot::{
            self,
            corrections::{
                CorrectTransformsResult,
                add_dependent_value_root_updates,
            },
        },
    },
};

type Graph = SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>;

impl
    split_snapshot::corrections::CorrectTransforms<
        NodeWeight,
        EdgeWeight,
        EdgeWeightKindDiscriminants,
    > for ComponentNodeWeight
{
    fn correct_transforms(
        &self,
        graph: &Graph,
        mut updates: Vec<Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
        _from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>>
    {
        // Net new components do not need any corrections (currently)
        if !graph.node_exists(self.id) {
            return Ok(updates);
        }

        let is_to_self = |update: &Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>| {
            update.destination_has_id(self.id)
        };

        let mut remove_edges = HashSet::new();
        let mut component_will_be_deleted = false;

        for update in &updates {
            match update {
                // Single Parent Rule: Component: FrameContains <Self> <- FrameContains: Component
                // When we're setting the parent for a component, we need to remove any existing
                // FrameContains edges to other components.
                Update::NewEdge { .. }
                    if update
                        .is_of_custom_edge_kind(EdgeWeightKindDiscriminants::FrameContains)
                        && is_to_self(update) =>
                {
                    remove_edges.extend(
                        graph
                            .incoming_edges(self.id(), EdgeWeightKindDiscriminants::FrameContains)?
                            .map(|edge_ref| edge_ref.triplet()),
                    );
                }

                // If the component is being deleted, the RemoveEdges may be stale (from an old
                // snapshot) and we need to ensure that we truly delete everything. Detected by
                // noticing an edge was removed from the component category:
                //
                //   Category -> Use: <Self>
                Update::RemoveEdge { .. }
                    if update.destination_has_id(self.id)
                        && update
                            .source_is_of_custom_node_kind(NodeWeightDiscriminants::Category) =>
                {
                    component_will_be_deleted = true;
                }
                // It's impossible for this to happen in a single rebase batch, however,
                // theoretically we could combine rebase batches.
                Update::NewEdge { .. }
                    if update.destination_has_id(self.id)
                        && update
                            .source_is_of_custom_node_kind(NodeWeightDiscriminants::Category) =>
                {
                    component_will_be_deleted = false;
                }

                // If SchemaVariant gets set, we are upgrading a component, which disconnects
                // and reconnects prop and socket values and connections. The disconnects may
                // be stale (based on an old snapshot), so when we detect schema upgrade, we
                // redo the disconnects.
                Update::NewEdge { .. }
                    if update.source_has_id(self.id)
                        && update.destination_is_of_custom_node_kind(
                            NodeWeightDiscriminants::SchemaVariant,
                        ) =>
                {
                    // All outgoing edges from the component have to be removed since they will all be
                    // reconstructed by the sv change
                    remove_edges.extend(
                        graph
                            .edges_directed(self.id, Outgoing)?
                            .map(|edge_ref| edge_ref.triplet()),
                    );

                    // Be sure to delete the root attribute value completely by removing incoming edges to it (it may have
                    // a value subscription edge)
                    if let Some(root_av_id) = graph
                        .outgoing_targets(self.id, EdgeWeightKindDiscriminants::Root)?
                        .next()
                    {
                        remove_edges.extend(
                            graph
                                .edges_directed(root_av_id, Incoming)?
                                .map(|edge_ref| edge_ref.triplet()),
                        );
                    }

                    // Input and output sockets get new prototype arguments during upgrade. So we remove all
                    // prototype arguments to be sure
                    let apas = sockets(graph, self.id)?
                        .filter_map(|socket| {
                            input_socket_connections(graph, self.id.into(), socket.into())
                                .ok()
                                .zip(
                                    output_socket_connections(graph, self.id.into(), socket.into())
                                        .ok(),
                                )
                        })
                        .flat_map(|(input_sockets, output_sockets)| {
                            input_sockets.chain(output_sockets)
                        });

                    remove_edges.extend(
                        apas.filter_map(|apa| graph.edges_directed(apa, Incoming).ok())
                            .flatten()
                            .map(|edge_ref| edge_ref.triplet()),
                    );
                }
                _ => {}
            }
        }

        if component_will_be_deleted {
            updates.extend(remove_hanging_socket_connections(graph, self.id)?);

            // The root attribute value id must be deleted if the component is being deleted
            if let Some(root_av_id) = graph
                .outgoing_targets(self.id, EdgeWeightKindDiscriminants::Root)?
                .next()
            {
                remove_edges.extend(
                    graph
                        .edges_directed(root_av_id, Incoming)?
                        .map(|edge_ref| edge_ref.triplet()),
                );
            }

            // Ensure we delete any incoming edges to the deleted component that might have been
            // added in another change set
            remove_edges.extend(
                graph
                    .edges_directed(self.id, Incoming)?
                    .map(|edge_ref| edge_ref.triplet()),
            );
        }

        for (source_id, kind, target_id) in remove_edges {
            updates.extend(Update::remove_edge_updates(
                graph, source_id, kind, target_id,
            )?);
        }

        Ok(updates)
    }
}

fn remove_hanging_socket_connections(
    graph: &Graph,
    component_id: SplitGraphNodeId,
) -> CorrectTransformsResult<Vec<Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>> {
    // To find the attribute prototype arguments that need to be removed, we
    // have to find the OutputSockets for this component. Once we find them, we
    // need to find the incoming PrototypeArgumentValue edge from the
    // AttributePrototypeArgument to that socket. Then we have to verify that
    // the argument has our component as a source. Then we can issue RemoveEdge
    // updates for all incoming edges to that attribute prototype argument. With
    // no incoming edges, the APA will be removed from the graph.

    let mut removals = vec![];
    let mut new_updates = vec![];
    let mut affected_attribute_values = BTreeSet::new();

    for socket_value_target in
        graph.outgoing_targets(component_id, EdgeWeightKindDiscriminants::SocketValue)?
    {
        for output_socket_id in graph
            .outgoing_targets(socket_value_target, EdgeWeightKindDiscriminants::Socket)?
            .filter(|target_id| {
                graph
                    .node_weight(*target_id)
                    .is_some_and(|weight| match weight {
                        NodeWeight::Content(content_node_weight) => {
                            content_node_weight.content_address_discriminants()
                                == ContentAddressDiscriminants::OutputSocket
                        }
                        _ => false,
                    })
            })
        {
            for (apa_id, destination_component_id) in graph
                .incoming_sources(
                    output_socket_id,
                    EdgeWeightKindDiscriminants::PrototypeArgumentValue,
                )?
                .filter_map(|source_id| {
                    graph
                        .node_weight(source_id)
                        .and_then(|node_weight| match node_weight {
                            NodeWeight::AttributePrototypeArgument(inner) => {
                                inner.targets().and_then(|targets| {
                                    if targets.source_component_id == component_id.into() {
                                        Some((source_id, targets.destination_component_id))
                                    } else {
                                        None
                                    }
                                })
                            }
                            _ => None,
                        })
                })
            {
                // Add remove edge updates for these attribute prototype arguments
                for edge_ref in graph.edges_directed(apa_id, Incoming)? {
                    removals.push(edge_ref.triplet());

                    for incoming_edge_ref in graph.edges_directed(edge_ref.source(), Incoming)? {
                        let maybe_av_id = edge_ref.source();
                        let Some(NodeWeight::AttributeValue(_)) =
                            graph.node_weight(incoming_edge_ref.source())
                        else {
                            continue;
                        };

                        if socket_attribute_value_belongs_to_component(
                            graph,
                            destination_component_id,
                            maybe_av_id,
                        )? {
                            affected_attribute_values.insert(maybe_av_id);
                        }
                    }
                }
            }
        }
    }

    for (source_id, kind, target_id) in removals {
        new_updates.extend(Update::remove_edge_updates(
            graph, source_id, kind, target_id,
        )?);
    }

    new_updates.extend(add_dependent_value_root_updates(
        graph,
        &affected_attribute_values,
    )?);

    Ok(new_updates)
}

fn socket_attribute_value_belongs_to_component(
    graph: &Graph,
    destination_component_id: ComponentId,
    starting_attribute_value: SplitGraphNodeId,
) -> CorrectTransformsResult<bool> {
    if !graph.node_exists(starting_attribute_value) {
        return Ok(false);
    }

    let Some(component_id) = graph
        .incoming_sources(
            starting_attribute_value,
            EdgeWeightKindDiscriminants::SocketValue,
        )?
        .next()
    else {
        return Ok(false);
    };

    Ok(component_id == destination_component_id.into())
}

// Get all Socket nodes for a given component (by going through its SocketValues).
//
// <COMPONENT> -> SocketValue -> Socket: <OUTPUT SOCKET> | <INPUT SOCKET>
fn sockets(
    graph: &Graph,
    component: SplitGraphNodeId,
) -> CorrectTransformsResult<impl Iterator<Item = SplitGraphNodeId> + '_> {
    Ok(graph
        .outgoing_targets(component, EdgeWeightKindDiscriminants::SocketValue)?
        .filter_map(|out_value| {
            graph
                .outgoing_targets(out_value, EdgeWeightKindDiscriminants::Socket)
                .ok()
        })
        .flatten())
}

/// Get all connection nodes (PrototypeArguments) to an input socket on a component.
fn input_socket_connections(
    graph: &Graph,
    component_id: ComponentId,
    input_socket: InputSocketId,
) -> CorrectTransformsResult<impl Iterator<Item = SplitGraphNodeId> + '_> {
    // From the Socket, find all PrototypeArguments representing the connection:
    // - <INPUT SOCKET> --Prototype-> --PrototypeArgument-> <ARG> --PrototypeArgumentValue-> <OUTPUT SOCKET>
    Ok(graph
        .outgoing_targets(input_socket.into(), EdgeWeightKindDiscriminants::Prototype)?
        .filter_map(|prototype| {
            graph
                .outgoing_targets(prototype, EdgeWeightKindDiscriminants::PrototypeArgument)
                .ok()
        })
        .flatten()
        .filter(move |&argument| {
            argument_targets(graph, argument)
                .is_some_and(|t| component_id == t.destination_component_id)
        }))
}

/// Get all connection nodes (PrototypeArguments) from an output socket on a component.
fn output_socket_connections(
    graph: &Graph,
    component_id: ComponentId,
    output_socket: OutputSocketId,
) -> CorrectTransformsResult<impl Iterator<Item = SplitGraphNodeId> + '_> {
    // From the output socket, walk back to the PrototypeArguments representing connections
    // - <INPUT SOCKET> --Prototype-> --PrototypeArgument-> <ARG> --PrototypeArgumentValue-> <OUTPUT SOCKET>
    Ok(graph
        .incoming_sources(
            output_socket.into(),
            EdgeWeightKindDiscriminants::PrototypeArgumentValue,
        )?
        .filter(move |&argument| {
            argument_targets(graph, argument).is_some_and(|t| component_id == t.source_component_id)
        }))
}

fn argument_targets(graph: &Graph, argument: SplitGraphNodeId) -> Option<ArgumentTargets> {
    match graph.node_weight(argument) {
        Some(NodeWeight::AttributePrototypeArgument(argument)) => argument.targets(),
        _ => None,
    }
}
