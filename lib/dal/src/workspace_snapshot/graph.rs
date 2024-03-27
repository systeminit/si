use petgraph::stable_graph::Edges;
use petgraph::{algo, prelude::*};
use serde::{Deserialize, Serialize};
use si_events::{
    deserialize_merkle_tree_hash_as_bytes, deserialize_node_weight_address_as_bytes,
    serialize_merkle_tree_hash_as_bytes, serialize_node_weight_address_as_bytes, MerkleTreeHash,
    NodeWeightAddress,
};
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;

use telemetry::prelude::*;
use thiserror::Error;
use ulid::Ulid;

use crate::workspace_snapshot::{
    edge_weight::{EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants},
    node_weight::NodeWeightError,
};

/// Ensure [`NodeIndex`] is usable by external crates.
pub use petgraph::graph::NodeIndex;
pub use petgraph::Direction;

use super::node_weight::NodeWeight;

pub type LineageId = Ulid;

#[allow(clippy::large_enum_variant)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum WorkspaceSnapshotGraphError {
    #[error("Cannot compare ordering of container elements between ordered, and un-ordered container: {0:?}, {1:?}")]
    CannotCompareOrderedAndUnorderedContainers(NodeIndex, NodeIndex),
    #[error("could not find category node used by node with index {0:?}")]
    CategoryNodeNotFound(NodeIndex),
    #[error("Unable to retrieve content for ContentHash")]
    ContentMissingForContentHash,
    #[error("Action would create a graph cycle")]
    CreateGraphCycle,
    #[error("could not find the newly imported subgraph when performing updates")]
    DestinationNotUpdatedWhenImportingSubgraph,
    #[error("Edge does not exist for EdgeIndex: {0:?}")]
    EdgeDoesNotExist(EdgeIndex),
    #[error("EdgeWeight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("EdgeWeight not found")]
    EdgeWeightNotFound,
    #[error("Problem during graph traversal: {0:?}")]
    GraphTraversal(petgraph::visit::DfsEvent<NodeIndex>),
    #[error("Incompatible node types")]
    IncompatibleNodeTypes,
    #[error("Invalid value graph")]
    InvalidValueGraph,
    #[error("NodeWeight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("node weight not found")]
    NodeWeightNotFound,
    #[error("Node with address not found: {0}")]
    NodeWithAddressNotFound(NodeWeightAddress),
    #[error("Node with ID {} not found", .0.to_string())]
    NodeWithIdNotFound(Ulid),
    #[error("No Prop found for NodeIndex {0:?}")]
    NoPropFound(NodeIndex),
    #[error("NodeIndex has too many Prop children: {0:?}")]
    TooManyPropForNode(NodeIndex),
    #[error("Workspace Snapshot has conflicts and must be rebased")]
    WorkspaceNeedsRebase,
    #[error("Workspace Snapshot has conflicts")]
    WorkspacesConflict,
}

pub type WorkspaceSnapshotGraphResult<T> = Result<T, WorkspaceSnapshotGraphError>;

#[derive(Serialize, Deserialize, Clone, Debug, Copy)]
pub struct GraphLocalNodeWeight {
    // The default serializers for these types serialize them as a string of hex digits, which is
    // twice the size of the pure byte representation. So we've implemented byte serialization so
    // that postcard will make these as small as possible
    #[serde(
        serialize_with = "serialize_node_weight_address_as_bytes",
        deserialize_with = "deserialize_node_weight_address_as_bytes"
    )]
    address: NodeWeightAddress,
    #[serde(
        serialize_with = "serialize_merkle_tree_hash_as_bytes",
        deserialize_with = "deserialize_merkle_tree_hash_as_bytes"
    )]
    merkle_tree_hash: MerkleTreeHash,
}

impl GraphLocalNodeWeight {
    fn new(address: NodeWeightAddress) -> Self {
        Self {
            address,
            merkle_tree_hash: MerkleTreeHash::nil(),
        }
    }

    pub fn address(&self) -> NodeWeightAddress {
        self.address
    }

    pub fn merkle_tree_hash(&self) -> MerkleTreeHash {
        self.merkle_tree_hash
    }

    pub fn set_merkle_tree_hash(&mut self, hash: MerkleTreeHash) {
        self.merkle_tree_hash = hash;
    }

    pub fn set_address(&mut self, hash: NodeWeightAddress) {
        self.address = hash;
    }
}

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct WorkspaceSnapshotGraph {
    graph: StableDiGraph<GraphLocalNodeWeight, EdgeWeight>,
    node_index_by_id: BTreeMap<Ulid, NodeIndex>,
    id_by_node_address: BTreeMap<NodeWeightAddress, Ulid>,
    node_indices_by_lineage_id: BTreeMap<LineageId, HashSet<NodeIndex>>,
    root_index: NodeIndex,
}

impl std::fmt::Debug for WorkspaceSnapshotGraph {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkspaceSnapshotGraph")
            .field("root_index", &self.root_index)
            .field("node_index_by_id", &self.node_index_by_id)
            .field("graph", &self.graph)
            .finish()
    }
}

impl WorkspaceSnapshotGraph {
    pub fn new(
        root_node_weight: Arc<NodeWeight>,
        root_address: NodeWeightAddress,
    ) -> WorkspaceSnapshotGraphResult<Self> {
        let mut graph = StableDiGraph::with_capacity(1, 0);
        let local_node_weight = GraphLocalNodeWeight::new(root_address);
        let root_index = graph.add_node(local_node_weight);
        let mut me = Self {
            root_index,
            graph,
            ..Default::default()
        };

        me.insert_into_maps(root_node_weight, root_index, root_address);
        Ok(me)
    }

    fn insert_into_maps(
        &mut self,
        node: Arc<NodeWeight>,
        node_index: NodeIndex,
        address: NodeWeightAddress,
    ) {
        // Update the accessor maps using the new index.
        self.id_by_node_address.insert(address, node.id());
        self.node_index_by_id.insert(node.id(), node_index);
        self.node_indices_by_lineage_id
            .entry(node.lineage_id())
            .and_modify(|set| {
                set.insert(node_index);
            })
            .or_insert_with(|| HashSet::from([node_index]));
    }

    pub(crate) fn graph(&self) -> &StableDiGraph<GraphLocalNodeWeight, EdgeWeight> {
        &self.graph
    }

    pub(crate) fn graph_mut(&mut self) -> &mut StableDiGraph<GraphLocalNodeWeight, EdgeWeight> {
        &mut self.graph
    }

    pub fn root(&self) -> NodeIndex {
        self.root_index
    }

    pub fn retain_node_index_by_id(&mut self, remaining_node_ids: HashSet<Ulid>) {
        self.node_index_by_id
            .retain(|id, _| remaining_node_ids.contains(id))
    }

    pub fn retain_node_indices_by_lineage_id(
        &mut self,
        remaining_node_indices_by_lineage_id: HashSet<NodeIndex>,
    ) {
        self.node_indices_by_lineage_id.retain(|_, node_indices| {
            node_indices
                .retain(|node_index| remaining_node_indices_by_lineage_id.contains(node_index));
            !node_indices.is_empty()
        });
    }

    pub fn retain_id_by_node_addresses(
        &mut self,
        remaining_node_addresses: HashSet<NodeWeightAddress>,
    ) {
        self.id_by_node_address
            .retain(|address, _| remaining_node_addresses.contains(address))
    }

    pub fn get_latest_node_idx_opt(
        &self,
        node_idx: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<NodeIndex>> {
        if !self.graph.contains_node(node_idx) {
            return Ok(None);
        }

        Ok(Some(self.get_latest_node_idx(node_idx)?))
    }

    pub fn get_latest_node_idx(
        &self,
        node_idx: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let node_address = self.get_node_weight(node_idx)?.address();
        let id = self.get_id_by_node_address(node_address)?;
        self.get_node_index_by_id(id)
    }

    pub fn add_edge(
        &mut self,
        from_node_index: NodeIndex,
        edge_weight: EdgeWeight,
        to_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<EdgeIndex> {
        // Add the new edge to the new version of the "from" node.
        let new_edge_index = self
            .graph
            .update_edge(from_node_index, to_node_index, edge_weight);
        self.update_merkle_tree_hash(from_node_index)?;

        Ok(new_edge_index)
    }

    pub(crate) fn remove_node_id(&mut self, id: impl Into<Ulid>) {
        self.node_index_by_id.remove(&id.into());
    }

    pub fn add_node(
        &mut self,
        node: Arc<NodeWeight>,
        node_address: NodeWeightAddress,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let local_node_weight = GraphLocalNodeWeight::new(node_address);

        // Create the node and cache the index.
        let new_node_index = self.graph.add_node(local_node_weight);

        // Update the accessor maps using the new index.
        self.insert_into_maps(node, new_node_index, node_address);
        self.update_merkle_tree_hash(new_node_index)?;

        Ok(new_node_index)
    }

    pub fn edges_directed(
        &self,
        node_index: NodeIndex,
        direction: Direction,
    ) -> Edges<'_, EdgeWeight, Directed, u32> {
        self.graph.edges_directed(node_index, direction)
    }

    pub fn edges_directed_for_edge_weight_kind(
        &self,
        node_index: NodeIndex,
        direction: Direction,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> Vec<(EdgeWeight, NodeIndex, NodeIndex)> {
        self.graph
            .edges_directed(node_index, direction)
            .filter_map(|edge_ref| {
                if edge_kind == edge_ref.weight().kind().into() {
                    Some((
                        edge_ref.weight().to_owned(),
                        edge_ref.source(),
                        edge_ref.target(),
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn nodes(&self) -> Vec<GraphLocalNodeWeight> {
        self.graph.node_weights().map(ToOwned::to_owned).collect()
    }

    pub fn edges(&self) -> impl Iterator<Item = (&EdgeWeight, NodeIndex, NodeIndex)> {
        self.graph.edge_indices().filter_map(|edge_idx| {
            self.get_edge_weight_opt(edge_idx)
                .ok()
                .flatten()
                .and_then(|weight| {
                    self.graph
                        .edge_endpoints(edge_idx)
                        .map(|(source, target)| (weight, source, target))
                })
        })
    }

    // pub async fn attribute_value_view(
    //     &self,
    //     content_store: &mut impl Store,
    //     root_index: NodeIndex,
    // ) -> WorkspaceSnapshotGraphResult<serde_json::Value> {
    //     let mut view = serde_json::json![{}];
    //     let mut nodes_to_add = VecDeque::from([(root_index, "".to_string())]);

    //     while let Some((current_node_index, write_location)) = nodes_to_add.pop_front() {
    //         let current_node_weight = self.get_node_weight(current_node_index)?;
    //         let current_node_content: serde_json::Value = content_store
    //             .get(&current_node_weight.content_hash())
    //             .await?
    //             .ok_or(WorkspaceSnapshotGraphError::ContentMissingForContentHash)?;
    //         // We don't need to care what kind the prop is, since assigning a value via
    //         // `pointer_mut` completely overwrites the existing value, regardless of any
    //         // pre-existing data types.
    //         let view_pointer = match view.pointer_mut(&write_location) {
    //             Some(pointer) => {
    //                 *pointer = current_node_content.clone();
    //                 pointer
    //             }
    //             None => {
    //                 // This is an error, and really shouldn't ever happen.
    //                 dbg!(view, write_location, current_node_content);
    //                 todo!();
    //             }
    //         };

    //         if current_node_content.is_null() {
    //             // If the value we just inserted is "null", then there shouldn't be any child
    //             // values, so don't bother looking for them in the graph to be able to add
    //             // them to the work queue.
    //             continue;
    //         }

    //         // Find the ordering if there is one, so we can add the children in the proper order.
    //         if let Some(child_ordering) = self.ordered_children_for_node(current_node_index)? {
    //             for (child_position_index, &child_node_index) in child_ordering.iter().enumerate() {
    //                 // `.enumerate()` gives us 1-indexed, but we need 0-indexed.

    //                 // We insert a JSON `Null` as a "place holder" for the write location. We need
    //                 // it to exist to be able to get a `pointer_mut` to it on the next time around,
    //                 // but we don't really care what it is, since we're going to completely
    //                 // overwrite it anyway.
    //                 for edge in self
    //                     .graph
    //                     .edges_connecting(current_node_index, child_node_index)
    //                 {
    //                     let child_position = match edge.weight().kind() {
    //                         EdgeWeightKind::Contain(Some(key)) => {
    //                             view_pointer
    //                                 .as_object_mut()
    //                                 .ok_or(WorkspaceSnapshotGraphError::InvalidValueGraph)?
    //                                 .insert(key.clone(), serde_json::json![null]);
    //                             key.clone()
    //                         }
    //                         EdgeWeightKind::Contain(None) => {
    //                             if current_node_content.is_array() {
    //                                 view_pointer
    //                                     .as_array_mut()
    //                                     .ok_or(WorkspaceSnapshotGraphError::InvalidValueGraph)?
    //                                     .push(serde_json::json![null]);
    //                                 child_position_index.to_string()
    //                             } else {
    //                                 // Get prop name
    //                                 if let NodeWeight::Prop(prop_weight) = self.get_node_weight(
    //                                     self.prop_node_index_for_node_index(child_node_index)?
    //                                         .ok_or(WorkspaceSnapshotGraphError::NoPropFound(
    //                                             child_node_index,
    //                                         ))?,
    //                                 )? {
    //                                     view_pointer
    //                                         .as_object_mut()
    //                                         .ok_or(WorkspaceSnapshotGraphError::InvalidValueGraph)?
    //                                         .insert(
    //                                             prop_weight.name().to_string(),
    //                                             serde_json::json![null],
    //                                         );
    //                                     prop_weight.name().to_string()
    //                                 } else {
    //                                     return Err(WorkspaceSnapshotGraphError::InvalidValueGraph);
    //                                 }
    //                             }
    //                         }
    //                         _ => continue,
    //                     };
    //                     let child_write_location = format!("{}/{}", write_location, child_position);
    //                     nodes_to_add.push_back((child_node_index, child_write_location));
    //                 }
    //             }
    //         } else {
    //             // The child nodes aren't explicitly ordered, so we'll need to come up with one of
    //             // our own. We'll sort the nodes by their `NodeIndex`, which means that when a
    //             // write last happened to them (or anywhere further towards the leaves) will
    //             // determine their sorting in oldest to most recent order.
    //             let mut child_index_to_position = HashMap::new();
    //             let mut child_indexes = Vec::new();
    //             let outgoing_edges = self.graph.edges_directed(current_node_index, Outgoing);
    //             for edge_ref in outgoing_edges {
    //                 match edge_ref.weight().kind() {
    //                     EdgeWeightKind::Contain(Some(key)) => {
    //                         view_pointer
    //                             .as_object_mut()
    //                             .ok_or(WorkspaceSnapshotGraphError::InvalidValueGraph)?
    //                             .insert(key.clone(), serde_json::json![null]);
    //                         child_index_to_position.insert(edge_ref.target(), key.clone());
    //                         child_indexes.push(edge_ref.target());
    //                     }
    //                     EdgeWeightKind::Contain(None) => {
    //                         child_indexes.push(edge_ref.target());
    //                         if current_node_content.is_array() {
    //                             view_pointer
    //                                 .as_array_mut()
    //                                 .ok_or(WorkspaceSnapshotGraphError::InvalidValueGraph)?
    //                                 .push(serde_json::json![null]);
    //                         } else {
    //                             // Get prop name
    //                             if let NodeWeight::Prop(prop_weight) = self.get_node_weight(
    //                                 self.prop_node_index_for_node_index(edge_ref.target())?
    //                                     .ok_or(WorkspaceSnapshotGraphError::NoPropFound(
    //                                         edge_ref.target(),
    //                                     ))?,
    //                             )? {
    //                                 view_pointer
    //                                     .as_object_mut()
    //                                     .ok_or(WorkspaceSnapshotGraphError::InvalidValueGraph)?
    //                                     .insert(
    //                                         prop_weight.name().to_string(),
    //                                         serde_json::json![null],
    //                                     );
    //                                 child_index_to_position
    //                                     .insert(edge_ref.target(), prop_weight.name().to_string());
    //                                 child_indexes.push(edge_ref.target());
    //                             } else {
    //                                 return Err(WorkspaceSnapshotGraphError::InvalidValueGraph);
    //                             }
    //                         }
    //                     }
    //                     _ => continue,
    //                 }
    //             }
    //             child_indexes.sort();

    //             for (child_position_index, child_node_index) in child_indexes.iter().enumerate() {
    //                 if let Some(key) = child_index_to_position.get(child_node_index) {
    //                     nodes_to_add
    //                         .push_back((*child_node_index, format!("{}/{}", write_location, key)));
    //                 } else {
    //                     nodes_to_add.push_back((
    //                         *child_node_index,
    //                         format!("{}/{}", write_location, child_position_index),
    //                     ));
    //                 }
    //             }
    //         }
    //     }

    //     Ok(view)
    // }

    pub fn find_equivalent_node(
        &self,
        id: Ulid,
        lineage_id: Ulid,
    ) -> WorkspaceSnapshotGraphResult<Option<NodeIndex>> {
        let maybe_equivalent_node = match self.get_node_index_by_id(id) {
            Ok(node_index) => {
                let node_indices = self.get_node_index_by_lineage(lineage_id);
                if node_indices.contains(&node_index) {
                    Some(node_index)
                } else {
                    None
                }
            }
            Err(WorkspaceSnapshotGraphError::NodeWithIdNotFound(_)) => None,
            Err(e) => return Err(e),
        };
        Ok(maybe_equivalent_node)
    }

    // #[allow(dead_code)]
    // pub fn dot(&self) {
    //     // NOTE(nick): copy the output and execute this on macOS. It will create a file in the
    //     // process and open a new tab in your browser.
    //     // ```
    //     // pbpaste | dot -Tsvg -o foo.svg && open foo.svg
    //     // ```
    //     let current_root_weight = self.get_node_weight(self.root_index).unwrap();
    //     println!(
    //         "Root Node Weight: {current_root_weight:?}\n{:?}",
    //         petgraph::dot::Dot::with_config(&self.graph, &[petgraph::dot::Config::EdgeNoLabel])
    //     );
    // }

    // #[allow(dead_code)]
    // pub fn tiny_dot_to_file(&self, suffix: Option<&str>) {
    //     let suffix = suffix.unwrap_or("dot");
    //     // NOTE(nick): copy the output and execute this on macOS. It will create a file in the
    //     // process and open a new tab in your browser.
    //     // ```
    //     // GRAPHFILE=<filename-without-extension>; cat $GRAPHFILE.txt | dot -Tsvg -o processed-$GRAPHFILE.svg; open processed-$GRAPHFILE.svg
    //     // ```
    //     let dot = petgraph::dot::Dot::with_attr_getters(
    //         &self.graph,
    //         &[
    //             petgraph::dot::Config::NodeNoLabel,
    //             petgraph::dot::Config::EdgeNoLabel,
    //         ],
    //         &|_, edgeref| {
    //             let discrim: EdgeWeightKindDiscriminants = edgeref.weight().kind().into();
    //             let color = match discrim {
    //                 EdgeWeightKindDiscriminants::Action => "black",
    //                 EdgeWeightKindDiscriminants::ActionPrototype => "black",
    //                 EdgeWeightKindDiscriminants::AuthenticationPrototype => "black",
    //                 EdgeWeightKindDiscriminants::Contain => "blue",
    //                 EdgeWeightKindDiscriminants::FrameContains => "black",
    //                 EdgeWeightKindDiscriminants::Ordering => "gray",
    //                 EdgeWeightKindDiscriminants::Ordinal => "gray",
    //                 EdgeWeightKindDiscriminants::Prop => "orange",
    //                 EdgeWeightKindDiscriminants::Prototype => "green",
    //                 EdgeWeightKindDiscriminants::PrototypeArgument => "green",
    //                 EdgeWeightKindDiscriminants::PrototypeArgumentValue => "green",
    //                 EdgeWeightKindDiscriminants::Socket => "red",
    //                 EdgeWeightKindDiscriminants::SocketValue => "purple",
    //                 EdgeWeightKindDiscriminants::Proxy => "gray",
    //                 EdgeWeightKindDiscriminants::Root => "black",
    //                 EdgeWeightKindDiscriminants::Use => "black",
    //             };

    //             match edgeref.weight().kind() {
    //                 EdgeWeightKind::Contain(key) => {
    //                     let key = key
    //                         .as_deref()
    //                         .map(|key| format!(" ({key}"))
    //                         .unwrap_or("".into());
    //                     format!(
    //                         "label = \"{discrim:?}{key}\"\nfontcolor = {color}\ncolor = {color}"
    //                     )
    //                 }
    //                 _ => format!("label = \"{discrim:?}\"\nfontcolor = {color}\ncolor = {color}"),
    //             }
    //         },
    //         &|_, (node_index, node_weight)| {
    //             let (label, color) = match node_weight {
    //                 NodeWeight::Content(weight) => {
    //                     let discrim = ContentAddressDiscriminants::from(weight.content_address());
    //                     let color = match discrim {
    //                         // Some of these should never happen as they have their own top-level
    //                         // NodeWeight variant.
    //                         ContentAddressDiscriminants::Action => "green",
    //                         ContentAddressDiscriminants::ActionBatch => "green",
    //                         ContentAddressDiscriminants::ActionRunner => "green",
    //                         ContentAddressDiscriminants::ActionPrototype => "green",
    //                         ContentAddressDiscriminants::AttributePrototype => "green",
    //                         ContentAddressDiscriminants::Component => "black",
    //                         ContentAddressDiscriminants::OutputSocket => "red",
    //                         ContentAddressDiscriminants::Func => "black",
    //                         ContentAddressDiscriminants::FuncArg => "black",
    //                         ContentAddressDiscriminants::InputSocket => "red",
    //                         ContentAddressDiscriminants::JsonValue => "fuchsia",
    //                         ContentAddressDiscriminants::Prop => "orange",
    //                         ContentAddressDiscriminants::Root => "black",
    //                         ContentAddressDiscriminants::Schema => "black",
    //                         ContentAddressDiscriminants::SchemaVariant => "black",
    //                         ContentAddressDiscriminants::Secret => "black",
    //                         ContentAddressDiscriminants::StaticArgumentValue => "green",
    //                         ContentAddressDiscriminants::ValidationPrototype => "black",
    //                     };
    //                     (discrim.to_string(), color)
    //                 }
    //                 NodeWeight::AttributePrototypeArgument(apa) => (
    //                     format!(
    //                         "Attribute Prototype Argument{}",
    //                         apa.targets()
    //                             .map(|targets| format!(
    //                                 "\nsource: {}\nto: {}",
    //                                 targets.source_component_id, targets.destination_component_id
    //                             ))
    //                             .unwrap_or("".to_string())
    //                     ),
    //                     "green",
    //                 ),
    //                 NodeWeight::AttributeValue(_) => ("Attribute Value".to_string(), "blue"),
    //                 NodeWeight::Category(category_node_weight) => match category_node_weight.kind()
    //                 {
    //                     CategoryNodeKind::Component => {
    //                         ("Components (Category)".to_string(), "black")
    //                     }
    //                     CategoryNodeKind::ActionBatch => {
    //                         ("Action Batches (Category)".to_string(), "black")
    //                     }
    //                     CategoryNodeKind::Func => ("Funcs (Category)".to_string(), "black"),
    //                     CategoryNodeKind::Schema => ("Schemas (Category)".to_string(), "black"),
    //                     CategoryNodeKind::Secret => ("Secrets (Category)".to_string(), "black"),
    //                 },
    //                 NodeWeight::Component(component) => (
    //                     "Component".to_string(),
    //                     if component.to_delete() {
    //                         "gray"
    //                     } else {
    //                         "black"
    //                     },
    //                 ),
    //                 NodeWeight::Func(func_node_weight) => {
    //                     (format!("Func\n{}", func_node_weight.name()), "black")
    //                 }
    //                 NodeWeight::FuncArgument(func_arg_node_weight) => (
    //                     format!("Func Arg\n{}", func_arg_node_weight.name()),
    //                     "black",
    //                 ),
    //                 NodeWeight::Ordering(_) => {
    //                     (NodeWeightDiscriminants::Ordering.to_string(), "gray")
    //                 }
    //                 NodeWeight::Prop(prop_node_weight) => {
    //                     (format!("Prop\n{}", prop_node_weight.name()), "orange")
    //                 }
    //             };
    //             let color = color.to_string();
    //             let id = node_weight.id();
    //             format!(
    //                 "label = \"\n\n{label}\n{node_index:?}\n{id}\n\n\"\nfontcolor = {color}\ncolor = {color}",
    //             )
    //         },
    //     );
    //     let filename_no_extension = format!("{}-{}", Ulid::new().to_string(), suffix);
    //     let mut file = File::create(format!("/home/zacharyhamm/{filename_no_extension}.txt"))
    //         .expect("could not create file");
    //     file.write_all(format!("{dot:?}").as_bytes())
    //         .expect("could not write file");
    //     println!("dot output stored in file (filename without extension: {filename_no_extension})");
    // }

    #[inline(always)]
    pub(crate) fn get_node_index_by_id(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let id = id.into();
        debug!("{:?}", self.node_index_by_id);
        self.get_node_index_by_id_opt(id)
            .ok_or(WorkspaceSnapshotGraphError::NodeWithIdNotFound(id))
    }

    pub(crate) fn get_node_index_by_id_opt(&self, id: impl Into<Ulid>) -> Option<NodeIndex> {
        let id = id.into();
        self.node_index_by_id.get(&id).copied()
    }

    pub(crate) fn get_id_by_node_address(
        &self,
        address: NodeWeightAddress,
    ) -> WorkspaceSnapshotGraphResult<Ulid> {
        match self.id_by_node_address.get(&address).copied().ok_or(
            WorkspaceSnapshotGraphError::NodeWithAddressNotFound(address),
        ) {
            Ok(a) => Ok(a),
            Err(e) => {
                error!("{}", e);
                Err(e)
            }
        }
    }

    pub(crate) fn get_node_index_by_lineage(&self, lineage_id: Ulid) -> HashSet<NodeIndex> {
        self.node_indices_by_lineage_id
            .get(&lineage_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn get_node_weight_opt(
        &self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<GraphLocalNodeWeight>> {
        Ok(self.graph.node_weight(node_index).copied())
    }

    pub fn get_node_weight(
        &self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<GraphLocalNodeWeight> {
        Ok(self
            .get_node_weight_opt(node_index)?
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?
            .to_owned())
    }

    pub fn get_edge_weight_opt(
        &self,
        edge_index: EdgeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<&EdgeWeight>> {
        Ok(self.graph.edge_weight(edge_index))
    }

    pub(crate) fn has_path_to_root(&self, node: NodeIndex) -> bool {
        algo::has_path_connecting(&self.graph, self.root_index, node, None)
    }

    #[allow(dead_code)]
    pub fn is_acyclic_directed(&self) -> bool {
        // Using this because "is_cyclic_directed" is recursive.
        algo::toposort(&self.graph, None).is_ok()
    }

    #[allow(dead_code)]
    fn is_on_path_between(&self, start: NodeIndex, end: NodeIndex, node: NodeIndex) -> bool {
        algo::has_path_connecting(&self.graph, start, node, None)
            && algo::has_path_connecting(&self.graph, node, end, None)
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn address_map_len(&self) -> usize {
        self.id_by_node_address.len()
    }

    pub(crate) fn remove_node(&mut self, node_index: NodeIndex) {
        self.graph.remove_node(node_index);
    }

    pub(crate) fn remove_edge(
        &mut self,
        source_node_index: NodeIndex,
        target_node_index: NodeIndex,
        edge_kind: EdgeWeightKindDiscriminants,
    ) {
        let mut edges_to_remove = vec![];
        for edgeref in self
            .graph
            .edges_connecting(source_node_index, target_node_index)
        {
            if edge_kind == edgeref.weight().kind().into() {
                edges_to_remove.push(edgeref.id());
            }
        }
        for edge_to_remove in edges_to_remove {
            self.graph.remove_edge(edge_to_remove);
        }
    }

    pub fn edge_endpoints(
        &self,
        edge_index: EdgeIndex,
    ) -> WorkspaceSnapshotGraphResult<(NodeIndex, NodeIndex)> {
        let (source, destination) = self
            .graph
            .edge_endpoints(edge_index)
            .ok_or(WorkspaceSnapshotGraphError::EdgeDoesNotExist(edge_index))?;
        Ok((source, destination))
    }

    pub(crate) fn update_root_index(&mut self) -> WorkspaceSnapshotGraphResult<()> {
        self.root_index = self.get_latest_node_idx(self.root_index)?;
        Ok(())
    }

    pub(crate) fn update_merkle_tree_hash(
        &mut self,
        node_index_to_update: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let mut hasher = MerkleTreeHash::hasher();
        hasher.update(
            self.get_node_weight(node_index_to_update)?
                .address()
                .as_bytes(),
        );

        let mut outgoing_neighbors = vec![];
        for out_neighbor_index in self
            .graph
            .neighbors_directed(node_index_to_update, Outgoing)
        {
            outgoing_neighbors.push((
                self.get_node_weight(out_neighbor_index)?,
                out_neighbor_index,
            ));
        }

        // Ensure all outgoing children are stably sorted so that we always
        // compute the same merkle tree hash for a node that has the same
        // outgoing children. There is no need to worry about the ordering
        // node's order since the ordering node will have a new address if it
        // has an updated order
        outgoing_neighbors.sort_by_key(|(weight, _)| weight.address);

        for (neighbor_node, neighbor_node_index) in outgoing_neighbors {
            hasher.update(neighbor_node.merkle_tree_hash().as_bytes());

            // The edge(s) between `node_index_to_update`, and `neighbor_node` potentially encode
            // important information related to the "identity" of `node_index_to_update`.
            for connecting_edgeref in self
                .graph
                .edges_connecting(node_index_to_update, neighbor_node_index)
            {
                match connecting_edgeref.weight().kind() {
                    // This is the key for an entry in a map.
                    EdgeWeightKind::Contain(Some(key)) => hasher.update(key.as_bytes()),

                    // This is the kind of the action.
                    EdgeWeightKind::ActionPrototype(kind) => {
                        hasher.update(kind.to_string().as_bytes())
                    }

                    EdgeWeightKind::Use { is_default } => {
                        hasher.update(is_default.to_string().as_bytes())
                    }

                    // This is the key representing an element in a container type corresponding
                    // to an AttributePrototype
                    EdgeWeightKind::Prototype(Some(key)) => hasher.update(key.as_bytes()),

                    // Nothing to do, as these EdgeWeightKind do not encode extra information
                    // in the edge itself.
                    EdgeWeightKind::AuthenticationPrototype
                    | EdgeWeightKind::Action
                    | EdgeWeightKind::Contain(None)
                    | EdgeWeightKind::FrameContains
                    | EdgeWeightKind::PrototypeArgument
                    | EdgeWeightKind::PrototypeArgumentValue
                    | EdgeWeightKind::Socket
                    | EdgeWeightKind::Ordering
                    | EdgeWeightKind::Ordinal
                    | EdgeWeightKind::Prop
                    | EdgeWeightKind::Prototype(None)
                    | EdgeWeightKind::Proxy
                    | EdgeWeightKind::Root
                    | EdgeWeightKind::SocketValue => {}
                }
            }
        }

        let new_node_weight = self
            .graph
            .node_weight_mut(node_index_to_update)
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?;

        new_node_weight.set_merkle_tree_hash(hasher.finalize());

        Ok(())
    }

    pub(crate) fn update_node_weight_address(
        &mut self,
        node_index: NodeIndex,
        address: NodeWeightAddress,
        id: Ulid,
    ) -> WorkspaceSnapshotGraphResult<()> {
        self.graph
            .node_weight_mut(node_index)
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?
            .set_address(address);
        self.id_by_node_address.insert(address, id);
        Ok(())
    }

    /// Given the node index for a node in other, find if a node exists in self that has the same
    /// id as the node found in other.
    #[instrument("info", skip_all)]
    pub(crate) fn find_latest_idx_in_self_from_other_idx(
        &self,
        other: &WorkspaceSnapshotGraph,
        other_idx: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<NodeIndex>> {
        let node_address = other.get_node_weight(other_idx)?.address();
        let other_id = other.get_id_by_node_address(node_address)?;

        Ok(self.get_node_index_by_id(other_id).ok())
    }
}
