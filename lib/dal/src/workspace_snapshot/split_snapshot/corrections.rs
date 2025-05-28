/// Split-Graph transform corrections.
///
/// Transform correction cannot be a generic trait in the si-split-graph library,
/// since it depends on the specific node weight types. For example, the ordering node
/// corrections need to know about AttributeValue container objects, and so on. But it
/// also has to be implemented for the custom node weight types.
use std::{
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
};

use petgraph::Direction::{Incoming, Outgoing};
use si_split_graph::{
    CustomEdgeWeight, CustomNodeWeight, EdgeKind, SplitGraph, SplitGraphEdgeWeight,
    SplitGraphEdgeWeightKind, SplitGraphError, SplitGraphNodeId, SplitGraphNodeWeight, Update,
    updates::ExternalSourceData,
};
use thiserror::Error;

use crate::{
    EdgeWeight, EdgeWeightKind, EdgeWeightKindDiscriminants, NodeWeightDiscriminants,
    workspace_snapshot::node_weight::{
        NodeWeight, category_node_weight::CategoryNodeKind, traits::SiVersionedNodeWeight,
    },
};

#[derive(Debug, Error)]
pub enum CorrectTransformsError {
    #[error("split graph error: {0}")]
    SplitGraph(#[from] SplitGraphError),
}

pub type CorrectTransformsResult<T> = Result<T, CorrectTransformsError>;

pub trait CorrectTransforms<N, E, K>
where
    N: CustomNodeWeight,
    E: CustomEdgeWeight<K>,
    K: EdgeKind,
{
    fn correct_transforms(
        &self,
        graph: &SplitGraph<N, E, K>,
        updates: Vec<Update<N, E, K>>,
        from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update<N, E, K>>>;
}

impl CorrectTransforms<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>
    for &SplitGraphNodeWeight<NodeWeight>
{
    fn correct_transforms(
        &self,
        graph: &SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
        updates: Vec<Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
        from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>>
    {
        match self {
            SplitGraphNodeWeight::Custom(c) => {
                c.correct_transforms(graph, updates, from_different_change_set)
            }
            SplitGraphNodeWeight::ExternalTarget { .. } => Ok(updates),
            SplitGraphNodeWeight::Ordering { id, order, .. } => correct_transforms_ordering_node(
                *id,
                self,
                order.as_slice(),
                graph,
                updates,
                from_different_change_set,
            ),
            SplitGraphNodeWeight::GraphRoot { .. } => Ok(updates),
            SplitGraphNodeWeight::SubGraphRoot { .. } => Ok(updates),
        }
    }
}

impl CorrectTransforms<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants> for &NodeWeight {
    fn correct_transforms(
        &self,
        graph: &SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
        updates: Vec<Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
        from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>>
    {
        match self {
            NodeWeight::Component(component_node_weight) => {
                component_node_weight.correct_transforms(graph, updates, from_different_change_set)
            }
            NodeWeight::DiagramObject(diagram_object_node_weight) => diagram_object_node_weight
                .correct_transforms(graph, updates, from_different_change_set),
            NodeWeight::Geometry(geometry_node_weight) => {
                geometry_node_weight.correct_transforms(graph, updates, from_different_change_set)
            }
            NodeWeight::View(view_node_weight) => {
                view_node_weight.correct_transforms(graph, updates, from_different_change_set)
            }
            NodeWeight::Secret(secret_node_weight) => {
                secret_node_weight.correct_transforms(graph, updates, from_different_change_set)
            }
            _ => Ok(updates),
        }
    }
}

impl<T> CorrectTransforms<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants> for T
where
    T: SiVersionedNodeWeight,
    T::Inner: CorrectTransforms<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
{
    fn correct_transforms(
        &self,
        graph: &SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
        updates: Vec<Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
        from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>>
    {
        self.inner()
            .correct_transforms(graph, updates, from_different_change_set)
    }
}

pub fn correct_transforms(
    graph: &SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
    mut updates: Vec<Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
    updates_are_from_different_change_set: bool,
) -> CorrectTransformsResult<Vec<Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>> {
    let mut new_nodes = BTreeMap::new();
    let mut nodes_to_interrogate = BTreeSet::new();

    for update in &updates {
        match update {
            Update::NewEdge {
                source,
                destination,
                edge_weight,
                ..
            } => {
                nodes_to_interrogate.insert(source.id);
                nodes_to_interrogate.insert(destination.id);
                if let Some(external_source_id) = edge_weight
                    .external_source_data()
                    .map(|data| data.source_id())
                {
                    nodes_to_interrogate.insert(external_source_id);
                }
            }
            Update::RemoveEdge {
                source,
                destination,
                external_source_data,
                ..
            } => {
                nodes_to_interrogate.insert(source.id);
                nodes_to_interrogate.insert(destination.id);
                if let Some(external_source_id) =
                    external_source_data.as_ref().map(|data| data.source_id())
                {
                    nodes_to_interrogate.insert(external_source_id);
                }
            }
            Update::RemoveNode { id, .. } => {
                nodes_to_interrogate.insert(*id);
            }
            Update::ReplaceNode {
                base_graph_node_id,
                node_weight,
                ..
            } => {
                nodes_to_interrogate.insert(node_weight.id());
                if let Some(base_graph_node_id) = base_graph_node_id {
                    nodes_to_interrogate.insert(*base_graph_node_id);
                }
            }
            Update::NewNode { node_weight, .. } => {
                new_nodes.insert(node_weight.id(), node_weight.clone());
            }
            Update::NewSubGraph { .. } => {}
        }
    }

    for node_id in nodes_to_interrogate {
        if let Some(node_weight) = graph
            .raw_node_weight(node_id)
            .or_else(|| new_nodes.get(&node_id))
        {
            updates = node_weight.correct_transforms(
                graph,
                updates,
                updates_are_from_different_change_set,
            )?;
        }
    }

    Ok(updates)
}

pub fn correct_transforms_ordering_node(
    id: SplitGraphNodeId,
    this_ordering_node: &SplitGraphNodeWeight<NodeWeight>,
    order: &[SplitGraphNodeId],
    graph: &SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
    mut updates: Vec<Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
    _updates_are_from_different_change_set: bool,
) -> CorrectTransformsResult<Vec<Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>> {
    // If this node doesn't exist in the current graph then it is a new node, and
    // no corrections are needed.
    if graph.node_id_to_index(id).is_none() {
        return Ok(updates);
    }

    // We need to handle the key conflicts for attribute value Contain edges
    // at the same time that we handle ordering conflicts, since we have to
    // be sure to remove the duplicate target from the AV's order, but also
    // preserve any other ordering changes that have come in from another
    // change set
    let maybe_attribute_value_container_id = graph
        .raw_edges_directed(id, Incoming)?
        .filter(|edge_ref| matches!(edge_ref.weight(), SplitGraphEdgeWeight::Ordering))
        .filter_map(|edge_ref| match graph.node_weight(edge_ref.source()) {
            Some(NodeWeight::AttributeValue(_)) => Some(edge_ref.source()),
            _ => None,
        })
        .next();

    let mut final_children: BTreeSet<SplitGraphNodeId> = order.iter().copied().collect();
    let mut replace_node_update_index = None;
    let mut new_av_contains = BTreeSet::new();

    for (update_index, update) in updates.iter().enumerate() {
        match update {
            Update::NewEdge {
                source,
                destination,
                edge_weight,
                ..
            } => {
                if source.id == id && matches!(edge_weight, SplitGraphEdgeWeight::Ordinal) {
                    final_children.insert(destination.id);
                } else if let Some(av_container_id) = maybe_attribute_value_container_id {
                    if source.id == av_container_id {
                        if let Some(EdgeWeightKind::Contain(Some(new_key))) =
                            edge_weight.custom().map(|e| e.kind())
                        {
                            new_av_contains.insert(new_key);
                        }
                    }
                }
            }
            Update::RemoveEdge {
                source,
                destination,
                edge_kind: SplitGraphEdgeWeightKind::Ordinal,
                ..
            } if source.id == id => {
                final_children.remove(&destination.id);
            }
            Update::ReplaceNode { node_weight, .. } if node_weight.id() == id => {
                replace_node_update_index = Some(update_index);
            }
            _ => {}
        }
    }

    let mut duplicate_contain_edge_target_updates: Vec<
        Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
    > = vec![];

    if let Some(av_container_id) = maybe_attribute_value_container_id {
        if let Some(split_node_index) = graph.node_id_to_index(av_container_id) {
            if let Some(subgraph_root_id) = graph.subgraph_root_id(split_node_index.subgraph()) {
                for duplicate_contain_edge_target in graph
                    .edges_directed(av_container_id, Outgoing)?
                    .filter(|edge_ref| match edge_ref.weight().kind() {
                        EdgeWeightKind::Contain(Some(key)) => new_av_contains.contains(key),
                        _ => false,
                    })
                    .filter_map(|edge_ref| graph.raw_node_weight(edge_ref.target()))
                {
                    // We need to produce a remove edge update for each duplicate here.
                    // *And*, if the target is an ExternalTarget then we need to find the corresponding
                    // ExternalSource edge and produce a remove edge update for it as well.
                    duplicate_contain_edge_target_updates.push(Update::RemoveEdge {
                        subgraph_root_id,
                        source: this_ordering_node.into(),
                        destination: duplicate_contain_edge_target.into(),
                        edge_kind: SplitGraphEdgeWeightKind::Custom(
                            EdgeWeightKindDiscriminants::Contain,
                        ),
                        external_source_data: None,
                    });

                    match duplicate_contain_edge_target {
                        SplitGraphNodeWeight::ExternalTarget { target, .. } => {
                            if let Some(target_node_index) = graph.node_id_to_index(*target) {
                                if let Some(target_subgraph_root_id) =
                                    graph.subgraph_root_id(target_node_index.subgraph())
                                {
                                    let source_and_target = graph
                                        .raw_edges_directed(*target, Incoming)?
                                        .find(|edge_ref| match edge_ref.weight() {
                                            SplitGraphEdgeWeight::ExternalSource {
                                                source_id,
                                                edge_kind,
                                                ..
                                            } => {
                                                *source_id == id
                                                    && *edge_kind
                                                        == EdgeWeightKindDiscriminants::Contain
                                            }
                                            _ => false,
                                        })
                                        .and_then(|edge_ref| {
                                            graph
                                                .raw_node_weight(edge_ref.source())
                                                .zip(graph.raw_node_weight(*target))
                                        });

                                    if let Some((source, target)) = source_and_target {
                                        duplicate_contain_edge_target_updates.push(
                                            Update::RemoveEdge {
                                                subgraph_root_id: target_subgraph_root_id,
                                                source: source.into(),
                                                destination: target.into(),
                                                edge_kind: SplitGraphEdgeWeightKind::ExternalSource,
                                                external_source_data: Some(ExternalSourceData {
                                                    source_id: id,
                                                    edge_kind: EdgeWeightKindDiscriminants::Contain,
                                                    // We're an ordering node, so we don't have a custom kind
                                                    source_node_kind: None,

                                                    phantom_n: PhantomData,
                                                }),
                                            },
                                        );
                                    }
                                }
                            }

                            final_children.remove(target);
                        }
                        SplitGraphNodeWeight::Custom(c) => {
                            final_children.remove(&c.id());
                        }
                        _ => {}
                    }
                }
            }
        }
    }

    // Generally, this will only be None if this is an entirely new ordering node.
    if let Some(replace_node_update_index) = replace_node_update_index {
        if let Some(Update::ReplaceNode {
            node_weight:
                SplitGraphNodeWeight::Ordering {
                    order: update_ordering,
                    ..
                },
            ..
        }) = updates.get_mut(replace_node_update_index)
        {
            let new_order = resolve_ordering(final_children, order, update_ordering);
            *update_ordering = new_order;
        };
    }

    Ok(updates)
}

fn resolve_ordering(
    final_children: BTreeSet<SplitGraphNodeId>,
    order: &[SplitGraphNodeId],
    update_order: &[SplitGraphNodeId],
) -> Vec<SplitGraphNodeId> {
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

pub fn choose_subgraph_root_for_node(
    graph: &SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
    node_in_subgraph: Option<SplitGraphNodeId>,
    category_node_id: SplitGraphNodeId,
) -> CorrectTransformsResult<SplitGraphNodeId> {
    Ok(node_in_subgraph
        .and_then(|node_in_subgraph| graph.subgraph_root_id_for_node(node_in_subgraph))
        .or_else(|| graph.subgraph_root_id_for_node(category_node_id))
        .unwrap_or(graph.root_id()?))
}

pub fn get_category_node_id(
    graph: &SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
    kind: CategoryNodeKind,
) -> CorrectTransformsResult<Option<SplitGraphNodeId>> {
    let root_id = graph.root_id()?;

    for maybe_category_node_id in
        graph.outgoing_targets(root_id, EdgeWeightKindDiscriminants::Use)?
    {
        let Some(NodeWeight::Category(inner)) = graph.node_weight(maybe_category_node_id) else {
            continue;
        };

        if inner.kind() == kind {
            return Ok(Some(maybe_category_node_id));
        }
    }

    Ok(None)
}

/// Produce the NewNode and NewEdge updates required for adding a dependent value root to the graph
pub fn add_dependent_value_root_updates(
    graph: &SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
    value_ids: &BTreeSet<SplitGraphNodeId>,
) -> CorrectTransformsResult<Vec<Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>> {
    let mut updates = vec![];

    if let Some(category_node_id) =
        get_category_node_id(graph, CategoryNodeKind::DependentValueRoots)?
    {
        let existing_dvu_nodes: BTreeSet<_> = graph
            .edges_directed(category_node_id, Outgoing)?
            .filter_map(|edge_ref| {
                graph
                    .node_weight(edge_ref.target())
                    .and_then(|weight| match weight {
                        NodeWeight::DependentValueRoot(inner) => Some(inner.value_id()),
                        _ => None,
                    })
            })
            .collect();

        for value_id in value_ids {
            if existing_dvu_nodes.contains(value_id) {
                continue;
            }

            let id = SplitGraphNodeId::new();
            let lineage_id = SplitGraphNodeId::new();
            let new_dvu_node = si_split_graph::SplitGraphNodeWeight::Custom(
                NodeWeight::new_dependent_value_root(id, lineage_id, *value_id),
            );

            let new_node_subgraph_root_id = choose_subgraph_root_for_node(
                graph,
                existing_dvu_nodes.last().copied(),
                category_node_id,
            )?;

            updates.push(Update::NewNode {
                subgraph_root_id: new_node_subgraph_root_id,
                node_weight: new_dvu_node,
            });

            updates.extend(Update::new_edge_between_nodes_updates(
                category_node_id,
                choose_subgraph_root_for_node(graph, Some(category_node_id), category_node_id)?,
                NodeWeightDiscriminants::Category,
                id,
                new_node_subgraph_root_id,
                NodeWeightDiscriminants::DependentValueRoot,
                EdgeWeight::new(EdgeWeightKind::new_use()),
                graph.root_id()?,
            ));
        }
    }

    Ok(updates)
}
