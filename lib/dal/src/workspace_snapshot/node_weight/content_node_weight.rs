use std::collections::{
    BTreeMap,
    HashMap,
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
use si_split_graph::{
    SplitGraphNodeId,
    SplitGraphNodeWeight,
};

use super::{
    NodeWeight,
    traits::{
        CorrectTransformsResult,
        SiVersionedNodeWeight,
    },
};
use crate::{
    ComponentId,
    EdgeWeight,
    EdgeWeightKindDiscriminants,
    SocketArity,
    WorkspaceSnapshotGraphVCurrent,
    workspace_snapshot::{
        NodeInformation,
        content_address::{
            ContentAddress,
            ContentAddressDiscriminants,
        },
        graph::{
            LineageId,
            detector::Update,
        },
        node_weight::{
            NodeWeightError,
            NodeWeightResult,
            traits::CorrectTransforms,
        },
        split_snapshot,
    },
};

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct ContentNodeWeight {
    /// The stable local ID of the object in question. Mainly used by external things like
    /// the UI to be able to say "do X to _this_ thing" since the `NodeIndex` is an
    /// internal implementation detail, and the content ID wrapped by the
    /// [`NodeWeightKind`] changes whenever something about the node itself changes (for
    /// example, the name, or type of a [`Prop`].)
    pub id: Ulid,
    /// Globally stable ID for tracking the "lineage" of a thing to determine whether it
    /// should be trying to receive updates.
    pub lineage_id: LineageId,
    /// What type of thing is this node representing, and what is the content hash used to
    /// retrieve the data for this specific node.
    pub content_address: ContentAddress,
    /// [Merkle tree](https://en.wikipedia.org/wiki/Merkle_tree) hash for the graph
    /// starting with this node as the root. Mainly useful in quickly determining "has
    /// something changed anywhere in this (sub)graph".
    pub merkle_tree_hash: MerkleTreeHash,
    /// The first time a [`ChangeSet`] has "seen" this content. This is useful for determining
    /// whether the absence of this content on one side or the other of a rebase/merge is because
    /// the content is new, or because one side deleted it.
    pub to_delete: bool,
}

impl ContentNodeWeight {
    pub fn new(id: Ulid, lineage_id: Ulid, content_address: ContentAddress) -> Self {
        Self {
            id,
            lineage_id,
            content_address,
            merkle_tree_hash: MerkleTreeHash::default(),
            to_delete: false,
        }
    }

    pub fn content_address(&self) -> ContentAddress {
        self.content_address
    }

    pub fn content_address_discriminants(&self) -> ContentAddressDiscriminants {
        self.content_address.into()
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
    pub fn to_delete(&self) -> bool {
        self.to_delete
    }

    pub fn lineage_id(&self) -> Ulid {
        self.lineage_id
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn new_content_hash(&mut self, content_hash: ContentHash) -> NodeWeightResult<()> {
        let new_address = match &self.content_address {
            ContentAddress::DeprecatedAction(_) => ContentAddress::DeprecatedAction(content_hash),
            ContentAddress::DeprecatedActionBatch(_) => {
                ContentAddress::DeprecatedActionBatch(content_hash)
            }
            ContentAddress::DeprecatedActionRunner(_) => {
                ContentAddress::DeprecatedActionRunner(content_hash)
            }
            ContentAddress::ActionPrototype(_) => ContentAddress::ActionPrototype(content_hash),
            ContentAddress::ApprovalRequirementDefinition(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "ApprovalRequirement".to_string(),
                    "Content".to_string(),
                ));
            }
            ContentAddress::AttributePrototype(_) => {
                ContentAddress::AttributePrototype(content_hash)
            }
            ContentAddress::Component(_) => ContentAddress::Component(content_hash),
            ContentAddress::OutputSocket(_) => ContentAddress::OutputSocket(content_hash),
            ContentAddress::FuncArg(_) => ContentAddress::FuncArg(content_hash),
            ContentAddress::Func(_) => ContentAddress::Func(content_hash),
            ContentAddress::Geometry(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "Geometry".to_string(),
                    "Content".to_string(),
                ));
            }
            ContentAddress::InputSocket(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "InputSocket".to_string(),
                    "Content".to_string(),
                ));
            }
            ContentAddress::JsonValue(_) => ContentAddress::JsonValue(content_hash),
            ContentAddress::Module(_) => ContentAddress::Module(content_hash),
            ContentAddress::Prop(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "Prop".to_string(),
                    "Content".to_string(),
                ));
            }
            ContentAddress::Root => return Err(NodeWeightError::CannotUpdateRootNodeContentHash),
            ContentAddress::Schema(_) => ContentAddress::Schema(content_hash),
            ContentAddress::SchemaVariant(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "SchemaVariant".to_string(),
                    "Content".to_string(),
                ));
            }
            ContentAddress::Secret(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "Secret".to_string(),
                    "Content".to_string(),
                ));
            }
            ContentAddress::StaticArgumentValue(_) => {
                ContentAddress::StaticArgumentValue(content_hash)
            }
            ContentAddress::ValidationPrototype(_) => {
                ContentAddress::ValidationPrototype(content_hash)
            }
            ContentAddress::ValidationOutput(_) => ContentAddress::ValidationOutput(content_hash),
            ContentAddress::ManagementPrototype(_) => {
                ContentAddress::ManagementPrototype(content_hash)
            }
            ContentAddress::View(_) => {
                return Err(NodeWeightError::InvalidContentAddressForWeightKind(
                    "Geometry".to_string(),
                    "Content".to_string(),
                ));
            }
        };

        self.content_address = new_address;

        Ok(())
    }

    pub fn node_hash(&self) -> ContentHash {
        self.content_hash()
    }
    pub fn set_to_delete(&mut self, to_delete: bool) -> bool {
        self.to_delete = to_delete;
        self.to_delete
    }

    pub fn set_merkle_tree_hash(&mut self, new_hash: MerkleTreeHash) {
        self.merkle_tree_hash = new_hash;
    }

    pub const fn exclusive_outgoing_edges(&self) -> &[EdgeWeightKindDiscriminants] {
        &[]
    }
}

impl std::fmt::Debug for ContentNodeWeight {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.debug_struct("ContentNodeWeight")
            .field("id", &self.id.to_string())
            .field("lineage_id", &self.lineage_id.to_string())
            .field("content_address", &self.content_address)
            .field("merkle_tree_hash", &self.merkle_tree_hash)
            .finish()
    }
}

fn remove_outgoing_prototype_argument_value_targets_to_dest(
    graph: &WorkspaceSnapshotGraphVCurrent,
    prototype_idx: NodeIndex,
    source: NodeInformation,
    destination_component_id: ComponentId,
) -> impl Iterator<Item = Update> + '_ {
    graph
        .edges_directed(prototype_idx, Outgoing)
        .filter(move |edge_ref| {
            EdgeWeightKindDiscriminants::PrototypeArgument == edge_ref.weight().kind().into()
        })
        .filter_map(move |edge_ref| {
            graph
                .get_node_weight_opt(edge_ref.target())
                .and_then(|weight| match weight {
                    NodeWeight::AttributePrototypeArgument(apa_inner) => {
                        apa_inner.targets().and_then(|targets| {
                            (targets.destination_component_id == destination_component_id)
                                .then_some(weight)
                        })
                    }
                    _ => None,
                })
                .map(|target_weight| Update::RemoveEdge {
                    source,
                    destination: target_weight.into(),
                    edge_kind: EdgeWeightKindDiscriminants::PrototypeArgument,
                })
        })
}

fn protect_arity_for_input_socket(
    graph: &WorkspaceSnapshotGraphVCurrent,
    mut updates: Vec<Update>,
    self_node: &ContentNodeWeight,
) -> Vec<Update> {
    let mut new_updates = vec![];

    if let Some(self_idx) = graph.get_node_index_by_id_opt(self_node.id()) {
        if let Some(input_socket_inner) = graph
            .edges_directed(self_idx, Incoming)
            .filter(|edge_ref| {
                EdgeWeightKindDiscriminants::Prototype == edge_ref.weight().kind().into()
            })
            .filter_map(|edge_ref| graph.get_node_weight_opt(edge_ref.source()))
            .next()
            .and_then(|node_weight| match node_weight {
                NodeWeight::InputSocket(inner) => Some(inner.inner()),
                _ => None,
            })
        {
            if input_socket_inner.arity() != SocketArity::One {
                return updates;
            }

            let mut new_node_map = HashMap::new();

            for update in &updates {
                match update {
                    Update::NewNode { node_weight } => {
                        if let NodeWeight::AttributePrototypeArgument(apa_inner) = node_weight {
                            new_node_map.insert(node_weight.id(), apa_inner);
                        }
                    }
                    Update::NewEdge {
                        source,
                        destination,
                        edge_weight,
                    } => {
                        if source.id == self_node.id().into()
                            && EdgeWeightKindDiscriminants::PrototypeArgument
                                == edge_weight.kind().into()
                        {
                            if let Some(&new_apa) = new_node_map.get(&destination.id.into()) {
                                let targets = match new_apa.targets() {
                                    Some(targets) => targets,
                                    None => {
                                        // No targets, then we don't want you
                                        continue;
                                    }
                                };

                                new_updates.extend(
                                    remove_outgoing_prototype_argument_value_targets_to_dest(
                                        graph,
                                        self_idx,
                                        *source,
                                        targets.destination_component_id,
                                    ),
                                );
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    updates.extend(new_updates);

    updates
}

impl CorrectTransforms for ContentNodeWeight {
    fn correct_transforms(
        &self,
        graph: &WorkspaceSnapshotGraphVCurrent,
        updates: Vec<Update>,
        _from_different_change_set: bool,
    ) -> CorrectTransformsResult<Vec<Update>> {
        Ok(match self.content_address_discriminants() {
            ContentAddressDiscriminants::AttributePrototype => {
                protect_arity_for_input_socket(graph, updates, self)
            }
            _ => updates,
        })
    }
}

impl
    split_snapshot::corrections::CorrectTransforms<
        NodeWeight,
        EdgeWeight,
        EdgeWeightKindDiscriminants,
    > for ContentNodeWeight
{
    fn correct_transforms(
        &self,
        graph: &si_split_graph::SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
        updates: Vec<si_split_graph::Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
        _from_different_change_set: bool,
    ) -> split_snapshot::corrections::CorrectTransformsResult<
        Vec<si_split_graph::Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
    > {
        Ok(match self.content_address_discriminants() {
            ContentAddressDiscriminants::AttributePrototype => {
                protect_arity_for_input_socket_split(graph, updates, self)?
            }
            _ => updates,
        })
    }
}

fn remove_outgoing_prototype_argument_value_targets_to_dest_split(
    graph: &si_split_graph::SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
    prototype_id: SplitGraphNodeId,
    destination_component_id: ComponentId,
) -> split_snapshot::corrections::CorrectTransformsResult<
    Vec<si_split_graph::Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
> {
    let mut remove_updates = vec![];

    let triplets: Vec<_> = graph
        .edges_directed_for_edge_weight_kind(
            prototype_id,
            Outgoing,
            EdgeWeightKindDiscriminants::PrototypeArgument,
        )?
        .filter(|edge_ref| {
            graph
                .node_weight(edge_ref.target())
                .is_some_and(|weight| match weight {
                    NodeWeight::AttributePrototypeArgument(apa_inner) => {
                        apa_inner.targets().is_some_and(|targets| {
                            targets.destination_component_id == destination_component_id
                        })
                    }
                    _ => false,
                })
        })
        .map(|edge_ref| edge_ref.triplet())
        .collect();

    for (source_id, kind, target_id) in triplets {
        remove_updates.extend(si_split_graph::Update::remove_edge_updates(
            graph, source_id, kind, target_id,
        )?);
    }

    Ok(remove_updates)
}

fn protect_arity_for_input_socket_split(
    graph: &si_split_graph::SplitGraph<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>,
    mut updates: Vec<si_split_graph::Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
    self_node: &ContentNodeWeight,
) -> split_snapshot::corrections::CorrectTransformsResult<
    Vec<si_split_graph::Update<NodeWeight, EdgeWeight, EdgeWeightKindDiscriminants>>,
> {
    let mut new_updates = vec![];

    if !graph.node_exists(self_node.id()) {
        return Ok(updates);
    }

    let Some(input_socket_inner) = graph
        .incoming_sources(self_node.id(), EdgeWeightKindDiscriminants::Prototype)?
        .filter_map(|source_id| graph.node_weight(source_id))
        .next()
        .and_then(|node_weight| match node_weight {
            NodeWeight::InputSocket(inner) => Some(inner.inner()),
            _ => None,
        })
    else {
        return Ok(updates);
    };

    if input_socket_inner.arity() != SocketArity::One {
        return Ok(updates);
    }

    let mut new_node_map = BTreeMap::new();

    for update in &updates {
        match update {
            si_split_graph::Update::NewNode { node_weight, .. } => {
                if let SplitGraphNodeWeight::Custom(NodeWeight::AttributePrototypeArgument(
                    apa_inner,
                )) = node_weight
                {
                    new_node_map.insert(node_weight.id(), apa_inner);
                }
            }
            si_split_graph::Update::NewEdge { destination, .. }
                if update.source_has_id(self_node.id())
                    && update
                        .is_of_custom_edge_kind(EdgeWeightKindDiscriminants::PrototypeArgument) =>
            {
                if let Some(&new_apa) = new_node_map.get(&destination.id) {
                    let targets = match new_apa.targets() {
                        Some(targets) => targets,
                        None => {
                            // No targets, then we don't want you
                            continue;
                        }
                    };

                    new_updates.extend(
                        remove_outgoing_prototype_argument_value_targets_to_dest_split(
                            graph,
                            self_node.id(),
                            targets.destination_component_id,
                        )?,
                    );
                }
            }
            _ => {}
        }
    }

    updates.extend(new_updates);

    Ok(updates)
}
