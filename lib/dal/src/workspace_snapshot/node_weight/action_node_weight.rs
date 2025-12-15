use std::collections::{
    BTreeSet,
    HashSet,
};

use petgraph::prelude::*;
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    merkle_tree_hash::MerkleTreeHash,
    ulid::Ulid,
};
use si_split_graph::SplitGraphNodeId;
use telemetry::prelude::*;

use super::traits::CorrectTransformsResult;
use crate::{
    ChangeSetId,
    EdgeWeight,
    EdgeWeightKindDiscriminants,
    NodeWeightDiscriminants,
    WorkspaceSnapshotGraphVCurrent,
    action::{
        ActionState,
        prototype::ActionKind,
    },
    workspace_snapshot::{
        NodeId,
        NodeInformation,
        graph::{
            LineageId,
            detector::Update,
        },
        node_weight::{
            NodeWeight,
            traits::CorrectTransforms,
        },
        split_snapshot,
    },
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ActionNodeWeight {
    pub id: Ulid,
    state: ActionState,
    originating_change_set_id: ChangeSetId,
    pub lineage_id: LineageId,
    merkle_tree_hash: MerkleTreeHash,
}

impl ActionNodeWeight {
    pub fn new(originating_change_set_id: ChangeSetId, id: Ulid, lineage_id: Ulid) -> Self {
        Self {
            id,
            state: ActionState::Queued,
            originating_change_set_id,
            lineage_id,
            merkle_tree_hash: MerkleTreeHash::default(),
        }
    }

    pub fn content_hash(&self) -> ContentHash {
        self.node_hash()
    }

    pub fn content_store_hashes(&self) -> Vec<ContentHash> {
        vec![]
    }

    pub fn id(&self) -> Ulid {
        self.id
    }

    pub fn set_state(&mut self, state: ActionState) {
        self.state = state;
    }

    pub fn state(&self) -> ActionState {
        self.state
    }

    /// Is this action in the state of [`ActionState::Dispatched`](ActionState)
    /// or [`ActionState::Running`](ActionState)?
    pub fn is_dispatched_or_running(&self) -> bool {
        matches!(self.state, ActionState::Dispatched | ActionState::Running)
    }

    pub fn originating_change_set_id(&self) -> ChangeSetId {
        self.originating_change_set_id
    }

    pub fn set_originating_change_set_id(&mut self, id: ChangeSetId) {
        self.originating_change_set_id = id;
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn node_hash(&self) -> ContentHash {
        let mut content_hasher = ContentHash::hasher();
        content_hasher.update(&self.id.inner().to_bytes());
        content_hasher.update(&self.lineage_id.inner().to_bytes());
        content_hasher.update(self.state.to_string().as_bytes());
        content_hasher.update(self.originating_change_set_id.to_string().as_bytes());

        content_hasher.finalize()
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl From<&ActionNodeWeight> for NodeInformation {
    fn from(value: &ActionNodeWeight) -> Self {
        Self {
            node_weight_kind: NodeWeightDiscriminants::Action,
            id: value.id.into(),
        }
    }
}

impl CorrectTransforms for ActionNodeWeight {
    fn correct_transforms(
        &self,
        graph: &WorkspaceSnapshotGraphVCurrent,
        updates: Vec<Update>,
        _from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update>> {
        match graph.get_node_index_by_id_opt(self.id) {
            Some(idx) => correct_transforms_already_exists(self, idx, graph, updates),
            None => correct_transforms_not_yet_exists(self.id(), graph, updates),
        }
    }
}

// An action's Use edge should be exclusive for both the component and
// the prototype. The generic exclusive edge logic assumes there can be
// one and only one edge of the given kind, so we have to do a custom
// implementation here for actions, since they should have just one use
// edge to a component and one use edge to a prototype. This is simpler
// than rewriting all the graphs to have distinct edge kinds for action
// prototypes and/or the components the action is for.
fn correct_transforms_already_exists(
    action_node_weight: &ActionNodeWeight,
    node_index: NodeIndex,
    graph: &WorkspaceSnapshotGraphVCurrent,
    mut updates: Vec<Update>,
) -> CorrectTransformsResult<Vec<Update>> {
    struct ActionUseTargets {
        component: Option<NodeId>,
        prototype: Option<NodeId>,
    }

    let mut removal_set = HashSet::new();
    let mut new_action_use_targets = ActionUseTargets {
        component: None,
        prototype: None,
    };

    for update in &updates {
        match update {
            Update::NewEdge {
                source,
                destination,
                edge_weight,
            } if source.id == action_node_weight.id().into()
                && EdgeWeightKindDiscriminants::Use == edge_weight.kind().into() =>
            {
                removal_set.remove(&destination.id);
                match destination.node_weight_kind {
                    NodeWeightDiscriminants::ActionPrototype => {
                        new_action_use_targets.prototype = Some(destination.id);
                    }
                    NodeWeightDiscriminants::Component => {
                        new_action_use_targets.component = Some(destination.id);
                    }
                    // If there's a use to some other thing, ignore it.
                    // Maybe some more functionality was added. What we care
                    // about is component and prototype targets
                    _ => {}
                }
            }
            Update::RemoveEdge {
                source,
                destination,
                edge_kind,
            } if source.id == action_node_weight.id().into()
                && EdgeWeightKindDiscriminants::Use == *edge_kind =>
            {
                removal_set.insert(destination.id);
            }
            _ => {}
        }
    }

    updates.extend(
        graph
            .edges_directed(node_index, Outgoing)
            .filter(|edge_ref| EdgeWeightKindDiscriminants::Use == edge_ref.weight().kind().into())
            .filter_map(|edge_ref| {
                graph
                    .get_node_weight_opt(edge_ref.target())
                    .and_then(|destination_weight| {
                        let should_remove = match destination_weight {
                            NodeWeight::ActionPrototype(_) => new_action_use_targets.prototype,
                            NodeWeight::Component(_) => new_action_use_targets.component,
                            _ => None,
                        }
                        .map(|new_destination_id| {
                            new_destination_id != destination_weight.id().into()
                                && !removal_set.contains(&destination_weight.id().into())
                        })
                        .unwrap_or(false);

                        should_remove.then_some(Update::RemoveEdge {
                            source: action_node_weight.into(),
                            destination: destination_weight.into(),
                            edge_kind: EdgeWeightKindDiscriminants::Use,
                        })
                    })
            }),
    );

    Ok(updates)
}

fn correct_transforms_not_yet_exists(
    id: Ulid,
    graph: &WorkspaceSnapshotGraphVCurrent,
    mut updates: Vec<Update>,
) -> CorrectTransformsResult<Vec<Update>> {
    let mut destinations_that_do_not_exist: HashSet<Ulid> = HashSet::new();
    let mut all_new_nodes = HashSet::new();
    let mut indices_for_new_node_updates_for_ourself = Vec::new();

    let mut components = HashSet::new();
    let mut action_prototype_id = None;
    let mut action_prototype_node_weight = None;
    let mut actions_that_will_be_removed: HashSet<Ulid> = HashSet::new();

    for (idx, update) in updates.iter().enumerate() {
        match update {
            Update::NewEdge {
                source,
                destination,
                edge_weight,
            } if source.id == id.into()
                && EdgeWeightKindDiscriminants::Use == edge_weight.kind().into() =>
            {
                match destination.node_weight_kind {
                    NodeWeightDiscriminants::ActionPrototype
                    | NodeWeightDiscriminants::Component => {
                        if graph.get_node_index_by_id_opt(destination.id).is_none() {
                            destinations_that_do_not_exist.insert(destination.id.into());
                        } else if destination.node_weight_kind
                            == NodeWeightDiscriminants::ActionPrototype
                        {
                            action_prototype_id = Some(destination.id.into());
                        } else {
                            components.insert(destination.id);
                        }
                    }
                    // If there's a use to some other thing, ignore it.
                    // Maybe some more functionality was added. What we care
                    // about is component and prototype targets
                    _ => {}
                }
            }
            Update::NewNode { node_weight } => {
                all_new_nodes.insert(node_weight.id());

                if node_weight.id() == id {
                    // This should happen one or zero times.
                    indices_for_new_node_updates_for_ourself.push(idx);
                } else if Some(node_weight.id()) == action_prototype_id {
                    action_prototype_node_weight = Some(node_weight.clone());
                }
            }
            Update::RemoveEdge {
                source,
                destination,
                edge_kind,
            } => {
                if (source.node_weight_kind == NodeWeightDiscriminants::Action
                    && destination.id == id.into()
                    && edge_kind == &EdgeWeightKindDiscriminants::Use)
                    || (source.node_weight_kind == NodeWeightDiscriminants::Category
                        && destination.node_weight_kind == NodeWeightDiscriminants::Action)
                {
                    actions_that_will_be_removed.insert(destination.id.into());
                }
            }
            _ => {}
        }
    }

    // If the destinations that no longer exist will not be included, then we
    // need to remove the update for ourself.
    if !destinations_that_do_not_exist.is_subset(&all_new_nodes) {
        if indices_for_new_node_updates_for_ourself.len() > 1 {
            warn!(
                action_id=%id,
                update_count=%indices_for_new_node_updates_for_ourself.len(),
                "unexpected multiple new node updates for ourself (legacy graph)"
            );
        }

        let mut removed = false;

        // Reverse the indices vec to remove from the back first.
        indices_for_new_node_updates_for_ourself.reverse();
        for idx in &indices_for_new_node_updates_for_ourself {
            updates.remove(*idx);
            removed = true;
        }

        // We removed this update, so no further corrections are necessary
        if removed {
            return Ok(updates);
        }
    }

    // If we got this far, we did not remove this action because it had a valid
    // prototype or component. But we need to confirm that the action is not a
    // duplicate of another of the "exclusive" action prototype kinds.

    let Some(action_protoype_id) = action_prototype_id else {
        return Ok(updates);
    };
    let Some(action_prototype_node_weight) = action_prototype_node_weight
        .or_else(|| graph.get_node_weight_by_id_opt(action_protoype_id).cloned())
    else {
        return Ok(updates);
    };

    // Manual actions are not exclusive
    if let NodeWeight::ActionPrototype(inner) = &action_prototype_node_weight {
        if inner.kind() == ActionKind::Manual {
            return Ok(updates);
        }
    }

    for component_id in components {
        let Some(component_node_index) = graph.get_node_index_by_id_opt(component_id) else {
            // if the component is not on the graph yet, we have nothing to
            // correct
            continue;
        };

        // Find all incoming use edges from actions to this component id. Then
        // find the action prototypes. If any of the action prototypes have the
        // same kind as the kind in `action_prototype_node_weight`, then remove
        // this action update
        for edge_ref in graph.edges_directed(component_node_index, Incoming) {
            // using edges_directed so we can keep this a iterator over refs
            // instead of cloning
            if EdgeWeightKindDiscriminants::Use != edge_ref.weight().kind().into() {
                continue;
            }

            let Some(NodeWeight::Action(action)) = graph.get_node_weight_opt(edge_ref.source())
            else {
                continue;
            };

            // If this action is about to be removed in this update set, ignore
            // it
            if actions_that_will_be_removed.contains(&action.id()) {
                continue;
            }

            for proto_edge_ref in graph.edges_directed(edge_ref.source(), Outgoing) {
                if EdgeWeightKindDiscriminants::Use != proto_edge_ref.weight().kind().into() {
                    continue;
                }

                let Some(NodeWeight::ActionPrototype(existing_prototype)) =
                    graph.get_node_weight_opt(proto_edge_ref.target())
                else {
                    continue;
                };

                let NodeWeight::ActionPrototype(our_prototype) = &action_prototype_node_weight
                else {
                    continue;
                };

                if existing_prototype.kind() == our_prototype.kind() {
                    indices_for_new_node_updates_for_ourself.reverse();
                    for idx in indices_for_new_node_updates_for_ourself {
                        updates.remove(idx);
                    }

                    return Ok(updates);
                }
            }
        }
    }

    Ok(updates)
}

impl
    split_snapshot::corrections::CorrectTransforms<
        NodeWeight,
        EdgeWeight,
        EdgeWeightKindDiscriminants,
    > for ActionNodeWeight
{
    fn correct_transforms(
        &self,
        graph: &si_split_graph::SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
        updates: Vec<si_split_graph::Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
        _from_different_change_set: bool,
    ) -> split_snapshot::corrections::CorrectTransformsResult<
        Vec<si_split_graph::Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
    > {
        if graph.node_exists(self.id()) {
            split_correct_transforms_already_exists(self.id(), graph, updates)
        } else {
            split_correct_transforms_not_yet_exists(self.id(), graph, updates)
        }
    }
}

// An action's Use edge should be exclusive for both the component and
// the prototype. The generic exclusive edge logic assumes there can be
// one and only one edge of the given kind, so we have to do a custom
// implementation here for actions, since they should have just one use
// edge to a component and one use edge to a prototype. This is simpler
// than rewriting all the graphs to have distinct edge kinds for action
// prototypes and/or the components the action is for.
fn split_correct_transforms_already_exists(
    id: Ulid,
    graph: &si_split_graph::SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
    mut updates: Vec<si_split_graph::Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
) -> split_snapshot::corrections::CorrectTransformsResult<
    Vec<si_split_graph::Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
> {
    struct ActionUseTargets {
        component: Option<SplitGraphNodeId>,
        prototype: Option<SplitGraphNodeId>,
    }

    let mut new_action_use_targets = ActionUseTargets {
        component: None,
        prototype: None,
    };

    let mut removal_set = BTreeSet::new();

    for update in &updates {
        match update {
            si_split_graph::Update::NewEdge { destination, .. }
                if update.source_has_id(id)
                    && update.is_of_custom_edge_kind(EdgeWeightKindDiscriminants::Use) =>
            {
                removal_set.remove(&destination.id);
                match destination.custom_kind {
                    Some(NodeWeightDiscriminants::ActionPrototype) => {
                        new_action_use_targets.prototype = Some(destination.id);
                    }
                    Some(NodeWeightDiscriminants::Component) => {
                        new_action_use_targets.component = Some(destination.id);
                    }
                    _ => {}
                }
            }
            si_split_graph::Update::RemoveEdge { destination, .. }
                if update.source_has_id(id)
                    && update.is_of_custom_edge_kind(EdgeWeightKindDiscriminants::Use) =>
            {
                removal_set.insert(&destination.id);
            }
            _ => {}
        }
    }

    let removals: Vec<_> = graph
        .outgoing_edges(id, EdgeWeightKindDiscriminants::Use)?
        .filter_map(|edge_ref| {
            graph
                .node_weight(edge_ref.target())
                .and_then(|destination_weight| {
                    match destination_weight {
                        NodeWeight::ActionPrototype(_) => new_action_use_targets.prototype,
                        NodeWeight::Component(_) => new_action_use_targets.component,
                        _ => None,
                    }
                    .is_some_and(|new_destination_id| {
                        new_destination_id != edge_ref.target()
                            && !removal_set.contains(&new_destination_id)
                    })
                    .then_some(edge_ref.triplet())
                })
        })
        .collect();

    for (source_id, kind, target_id) in removals {
        updates.extend(si_split_graph::Update::remove_edge_updates(
            graph, source_id, kind, target_id,
        )?);
    }

    Ok(updates)
}

// TODO(nick,zack): this needs to be re-tested when rolling out split graph. The legacy graph
// implementation for the similarly named function basically needs to be ported here. At the time
// of writing, the port was complete, but was untested due to other errors related to using split
// graph in integration tests.
fn split_correct_transforms_not_yet_exists(
    id: Ulid,
    graph: &si_split_graph::SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
    mut updates: Vec<si_split_graph::Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
) -> split_snapshot::corrections::CorrectTransformsResult<
    Vec<si_split_graph::Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
> {
    let mut destinations_that_do_not_exist: HashSet<Ulid> = HashSet::new();
    let mut all_new_nodes = HashSet::new();
    let mut indices_for_new_node_updates_for_ourself = Vec::new();

    for (idx, update) in updates.iter().enumerate() {
        match update {
            si_split_graph::Update::NewEdge { .. }
                if update.source_has_id(id)
                    && update.is_of_custom_edge_kind(EdgeWeightKindDiscriminants::Use) =>
            {
                if update
                    .destination_is_of_custom_node_kind(NodeWeightDiscriminants::ActionPrototype)
                    || update.destination_is_of_custom_node_kind(NodeWeightDiscriminants::Component)
                {
                    if let Some(destination_id) = update.destination_id() {
                        if !graph.node_exists(destination_id) {
                            destinations_that_do_not_exist.insert(destination_id);
                        }
                    }
                }
            }
            si_split_graph::Update::NewNode {
                node_weight,
                subgraph_root_id: _,
            } => {
                all_new_nodes.insert(node_weight.id());

                // This should happen one or zero times.
                if node_weight.id() == id {
                    indices_for_new_node_updates_for_ourself.push(idx);
                }
            }
            _ => {}
        }
    }

    // If the destinations that no longer exist will not be included, then we need to remove the
    // update for ourself.
    if !destinations_that_do_not_exist.is_subset(&all_new_nodes) {
        if indices_for_new_node_updates_for_ourself.len() > 1 {
            warn!(
                action_id=%id,
                update_count=%indices_for_new_node_updates_for_ourself.len(),
                "unexpected multiple new node updates for ourself (split graph)"
            );
        }

        // Reverse the indices vec to remove from the back first.
        indices_for_new_node_updates_for_ourself.reverse();
        for idx in indices_for_new_node_updates_for_ourself {
            updates.remove(idx);
        }
    }

    Ok(updates)
}
