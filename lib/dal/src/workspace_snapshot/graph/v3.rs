use std::{
    collections::{
        HashMap,
        HashSet,
        VecDeque,
        hash_map::Entry,
    },
    fs::File,
    io::Write,
    sync::{
        Arc,
        Mutex,
    },
};

use petgraph::{
    algo,
    prelude::*,
    stable_graph::{
        Edges,
        Neighbors,
    },
    visit::DfsEvent,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::{
    ContentHash,
    ulid::Ulid,
};
use si_layer_cache::db::serialize;
use telemetry::prelude::*;
use ulid::Generator;

use crate::{
    EdgeWeight,
    EdgeWeightKind,
    EdgeWeightKindDiscriminants,
    NodeWeightDiscriminants,
    workspace_snapshot::{
        CategoryNodeKind,
        LineageId,
        OrderingNodeWeight,
        content_address::{
            ContentAddress,
            ContentAddressDiscriminants,
        },
        graph::{
            MerkleTreeHash,
            WorkspaceSnapshotGraphError,
            WorkspaceSnapshotGraphResult,
            detector::Update,
        },
        node_weight::{
            CategoryNodeWeight,
            NodeWeight,
        },
    },
};

#[derive(Default, Deserialize, Serialize, Clone)]
pub struct WorkspaceSnapshotGraphV3 {
    graph: StableDiGraph<NodeWeight, EdgeWeight>,
    node_index_by_id: HashMap<Ulid, NodeIndex>,
    node_indices_by_lineage_id: HashMap<LineageId, HashSet<NodeIndex>>,
    root_index: NodeIndex,

    #[serde(skip)]
    ulid_generator: Arc<Mutex<Generator>>,
    #[serde(skip)]
    touched_node_indices: HashSet<NodeIndex>,
}

impl std::fmt::Debug for WorkspaceSnapshotGraphV3 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WorkspaceSnapshotGraph")
            .field("root_index", &self.root_index)
            .field("node_index_by_id", &self.node_index_by_id)
            .field("graph", &self.graph)
            .finish()
    }
}

impl WorkspaceSnapshotGraphV3 {
    pub fn new() -> WorkspaceSnapshotGraphResult<Self> {
        let mut graph: StableDiGraph<NodeWeight, EdgeWeight> =
            StableDiGraph::with_capacity(1024, 1024);
        let mut generator = Generator::new();

        let root_node = NodeWeight::new_content(
            generator.generate()?.into(),
            generator.generate()?.into(),
            ContentAddress::Root,
        );

        let node_id = root_node.id();
        let lineage_id = root_node.lineage_id();
        let root_index = graph.add_node(root_node);

        let mut result = Self {
            root_index,
            graph,
            ulid_generator: Arc::new(Mutex::new(generator)),
            ..Default::default()
        };

        result.add_node_finalize(node_id, lineage_id, root_index)?;

        Ok(result)
    }

    /// Add a node to the list of touched nodes, so that it is taken into
    /// account when recalculating the merkle tree hash for this graph. If a
    /// node weight is modified, or if a an outgoing edge is added or removed
    /// to/from this node, you must touch the node, or the merkel tree hash will
    /// not be updated correctly.
    pub fn touch_node(&mut self, node_index: NodeIndex) {
        self.touched_node_indices.insert(node_index);
    }

    pub fn new_from_parts(
        graph: StableDiGraph<NodeWeight, EdgeWeight>,
        node_index_by_id: HashMap<Ulid, NodeIndex>,
        node_indices_by_lineage_id: HashMap<LineageId, HashSet<NodeIndex>>,
        root_index: NodeIndex,
    ) -> Self {
        Self {
            graph,
            node_index_by_id,
            node_indices_by_lineage_id,
            root_index,
            ulid_generator: Arc::new(Mutex::new(Generator::new())),
            touched_node_indices: HashSet::new(),
        }
    }

    pub fn root(&self) -> NodeIndex {
        self.root_index
    }

    /// Access the internal petgraph for this snapshot
    pub fn graph(&self) -> &StableGraph<NodeWeight, EdgeWeight> {
        &self.graph
    }

    pub(crate) fn node_index_by_id(&self) -> &HashMap<Ulid, NodeIndex> {
        &self.node_index_by_id
    }

    pub(crate) fn node_indices_by_lineage_id(&self) -> &HashMap<Ulid, HashSet<NodeIndex>> {
        &self.node_indices_by_lineage_id
    }

    pub fn generate_ulid(&self) -> WorkspaceSnapshotGraphResult<Ulid> {
        Ok(self
            .ulid_generator
            .lock()
            .map_err(|e| WorkspaceSnapshotGraphError::MutexPoison(e.to_string()))?
            .generate()?
            .into())
    }

    pub async fn update_node_id(
        &mut self,
        current_idx: NodeIndex,
        new_id: impl Into<Ulid>,
        new_lineage_id: LineageId,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let new_id = new_id.into();

        self.graph
            .node_weight_mut(current_idx)
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?
            .set_id_and_lineage(new_id, new_lineage_id);

        self.add_node_finalize(new_id, new_lineage_id, current_idx)?;

        Ok(())
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

    #[inline(always)]
    pub fn get_latest_node_idx(
        &self,
        node_idx: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let node_id = self.get_node_weight(node_idx)?.id();
        self.get_node_index_by_id(node_id)
    }

    fn add_edge_inner(
        &mut self,
        from_node_index: NodeIndex,
        edge_weight: EdgeWeight,
        to_node_index: NodeIndex,
        cycle_check: bool,
    ) -> WorkspaceSnapshotGraphResult<EdgeIndex> {
        if cycle_check {
            self.add_temp_edge_cycle_check(from_node_index, edge_weight.clone(), to_node_index)?;
        }

        self.touch_node(from_node_index);

        // Add the new edge to the new version of the "from" node.
        let edge_index = self
            .graph
            .update_edge(from_node_index, to_node_index, edge_weight);

        Ok(edge_index)
    }

    fn add_temp_edge_cycle_check(
        &mut self,
        from_node_index: NodeIndex,
        edge_weight: EdgeWeight,
        to_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let temp_edge = self
            .graph
            .update_edge(from_node_index, to_node_index, edge_weight.clone());

        let would_create_a_cycle = !self.is_acyclic_directed();
        self.graph.remove_edge(temp_edge);

        if would_create_a_cycle {
            // if you want to find out how the two nodes are already connected,
            // this will give you that info..

            // let paths: Vec<Vec<NodeIndex>> = petgraph::algo::all_simple_paths(
            //     &self.graph,
            //     to_node_index,
            //     from_node_index,
            //     0,
            //     None,
            // )
            // .collect();

            // for path in paths {
            //     for node_index in path {
            //         let node_weight = self.get_node_weight(node_index).expect("should exist");
            //         dbg!(node_weight);
            //     }
            // }

            Err(WorkspaceSnapshotGraphError::CreateGraphCycle)
        } else {
            Ok(())
        }
    }

    pub fn add_edge_with_cycle_check(
        &mut self,
        from_node_index: NodeIndex,
        edge_weight: EdgeWeight,
        to_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<EdgeIndex> {
        self.add_edge_inner(from_node_index, edge_weight, to_node_index, true)
    }

    pub fn add_edge(
        &mut self,
        from_node_index: NodeIndex,
        edge_weight: EdgeWeight,
        to_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<EdgeIndex> {
        // Temporarily add the edge to the existing tree to see if it would create a cycle.
        // Configured to run only in tests because it has a major perf impact otherwise
        #[cfg(test)]
        {
            self.add_temp_edge_cycle_check(from_node_index, edge_weight.clone(), to_node_index)?;
        }

        self.add_edge_inner(from_node_index, edge_weight, to_node_index, false)
    }

    pub fn remove_node_id(&mut self, id: impl Into<Ulid>) {
        self.node_index_by_id.remove(&id.into());
    }

    fn add_node_finalize(
        &mut self,
        node_id: Ulid,
        lineage_id: Ulid,
        node_idx: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<()> {
        // Update the accessor maps using the new index.
        self.node_index_by_id.insert(node_id, node_idx);
        self.node_indices_by_lineage_id
            .entry(lineage_id)
            .and_modify(|set| {
                set.insert(node_idx);
            })
            .or_insert_with(|| HashSet::from([node_idx]));
        self.update_merkle_tree_hash(node_idx)?;
        self.touch_node(node_idx);

        Ok(())
    }

    /// Adds this node to the graph, or replaces it if a node with the same id
    /// already exists.  Then, adds it to the list of touched nodes so that the
    /// merkle tree hash for it, and any parents, is recalculated.
    pub fn add_or_replace_node(
        &mut self,
        node: NodeWeight,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let node_id = node.id();
        let lineage_id = node.lineage_id();
        let node_idx = self
            .get_node_index_by_id_opt(node_id)
            .and_then(|current_index| {
                self.graph.node_weight_mut(current_index).map(|weight_mut| {
                    node.clone_into(weight_mut);
                    current_index
                })
            });

        let node_idx = match node_idx {
            Some(swapped_node_idx) => swapped_node_idx,
            None => self.graph.add_node(node),
        };

        self.add_node_finalize(node_id, lineage_id, node_idx)?;

        Ok(node_idx)
    }

    pub fn add_category_node(
        &mut self,
        id: Ulid,
        lineage_id: Ulid,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let inner_weight = CategoryNodeWeight::new(id, lineage_id, kind);
        let new_node_index = self.add_or_replace_node(NodeWeight::Category(inner_weight))?;
        Ok(new_node_index)
    }

    pub fn get_category_node(
        &self,
        source: Option<Ulid>,
        kind: CategoryNodeKind,
    ) -> WorkspaceSnapshotGraphResult<Option<(Ulid, NodeIndex)>> {
        let source_index = match source {
            Some(provided_source) => self.get_node_index_by_id(provided_source)?,
            None => self.root_index,
        };

        // TODO(nick): ensure that two target category nodes of the same kind don't exist for the
        // same source node.
        for edgeref in self.graph.edges_directed(source_index, Outgoing) {
            let maybe_category_node_index = edgeref.target();
            let maybe_category_node_weight = self.get_node_weight(maybe_category_node_index)?;

            if let NodeWeight::Category(category_node_weight) = maybe_category_node_weight {
                if category_node_weight.kind() == kind {
                    return Ok(Some((category_node_weight.id(), maybe_category_node_index)));
                }
            }
        }

        Ok(None)
    }

    pub fn edges_directed(
        &self,
        node_index: NodeIndex,
        direction: Direction,
    ) -> Edges<'_, EdgeWeight, Directed, u32> {
        self.graph.edges_directed(node_index, direction)
    }

    pub fn neighbors_directed(
        &self,
        node_index: NodeIndex,
        direction: Direction,
    ) -> Neighbors<'_, EdgeWeight> {
        self.graph.neighbors_directed(node_index, direction)
    }

    pub fn find_edge(&self, from_idx: NodeIndex, to_idx: NodeIndex) -> Option<&EdgeWeight> {
        self.graph
            .find_edge(from_idx, to_idx)
            .and_then(|edge_idx| self.graph.edge_weight(edge_idx))
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

    pub fn nodes(&self) -> impl Iterator<Item = (&NodeWeight, NodeIndex)> {
        self.graph.node_indices().filter_map(|node_idx| {
            self.get_node_weight_opt(node_idx)
                .map(|weight| (weight, node_idx))
        })
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

    pub fn add_ordered_edge(
        &mut self,
        from_node_index: NodeIndex,
        edge_weight: EdgeWeight,
        to_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<(EdgeIndex, Option<EdgeIndex>)> {
        let new_edge_index = self.add_edge(from_node_index, edge_weight, to_node_index)?;

        // Find the ordering node of the "container" if there is one, and add the thing pointed to
        // by the `to_node_index` to the ordering. Also point the ordering node at the thing with
        // an `Ordinal` edge, so that Ordering nodes must be touched *after* the things they order
        // in a depth first search
        let maybe_ordinal_edge_index = if let Some(container_ordering_node_index) =
            self.ordering_node_index_for_container(from_node_index)?
        {
            let ordinal_edge_index = self.add_edge(
                container_ordering_node_index,
                EdgeWeight::new(EdgeWeightKind::Ordinal),
                to_node_index,
            )?;

            let element_id = self
                .node_index_to_id(to_node_index)
                .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?;

            if let NodeWeight::Ordering(ordering_node_weight) =
                self.get_node_weight_mut(container_ordering_node_index)?
            {
                ordering_node_weight.push_to_order(element_id);
                self.touch_node(container_ordering_node_index);
            }

            Some(ordinal_edge_index)
        } else {
            None
        };

        Ok((new_edge_index, maybe_ordinal_edge_index))
    }

    pub fn add_ordered_node(
        &mut self,
        node: NodeWeight,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let new_node_index = self.add_or_replace_node(node)?;

        let ordering_node_id = self.generate_ulid()?;
        let ordering_node_lineage_id = self.generate_ulid()?;
        let ordering_node_index = self.add_or_replace_node(NodeWeight::Ordering(
            OrderingNodeWeight::new(ordering_node_id, ordering_node_lineage_id),
        ))?;

        let edge_index = self.add_edge(
            new_node_index,
            EdgeWeight::new(EdgeWeightKind::Ordering),
            ordering_node_index,
        )?;
        let (source, _) = self.edge_endpoints(edge_index)?;
        Ok(source)
    }

    /// Remove any orphaned nodes from the graph, then recalculate the merkle
    /// tree hash based on the nodes touched. *ALWAYS* call this before
    /// persisting a snapshot
    pub fn cleanup_and_merkle_tree_hash(&mut self) -> WorkspaceSnapshotGraphResult<()> {
        self.cleanup();
        self.recalculate_entire_merkle_tree_hash_based_on_touched_nodes()?;

        Ok(())
    }

    /// Remove any orphaned nodes from the graph. If you are about to persist
    /// the graph, or calculate updates based on this graph and another one, then
    /// you want to call `Self::cleanup_and_merkle_tree_hash` instead.
    pub fn cleanup(&mut self) {
        let start = tokio::time::Instant::now();

        // We want to remove all of the "garbage" we've accumulated while operating on the graph.
        // Anything that is no longer reachable from the current `self.root_index` should be
        // removed as it is no longer referenced by anything in the current version of the graph.
        // Fortunately, we don't need to walk the graph to find out if something is reachable from
        // the root, since `has_path_connecting` is slow (depth-first search). Any node that does
        // *NOT* have any incoming edges (aside from the `self.root_index` node) is not reachable,
        // by definition. Finding the list of nodes with no incoming edges is very fast. If we
        // remove all nodes (that are not the `self.root_index` node) that do not have any
        // incoming edges, and we keep doing this until the only one left is the `self.root_index`
        // node, then all remaining nodes are reachable from `self.root_index`.
        let mut old_root_ids: HashSet<NodeIndex>;
        loop {
            old_root_ids = self
                .graph
                .externals(Incoming)
                .filter(|node_id| *node_id != self.root_index)
                .collect();
            if old_root_ids.is_empty() {
                break;
            }

            for stale_node_index in &old_root_ids {
                self.graph.remove_node(*stale_node_index);
            }
        }
        debug!("Removed stale NodeIndex: {:?}", start.elapsed());

        // After we retain the nodes, collect the remaining ids and indices.
        let remaining_node_ids: HashSet<Ulid> = self.graph.node_weights().map(|n| n.id()).collect();
        debug!(
            "Got remaining node IDs: {} ({:?})",
            remaining_node_ids.len(),
            start.elapsed()
        );
        let remaining_node_indices: HashSet<NodeIndex> = self.graph.node_indices().collect();
        debug!(
            "Got remaining NodeIndex: {} ({:?})",
            remaining_node_indices.len(),
            start.elapsed()
        );

        // Cleanup the node index by id map.
        self.node_index_by_id
            .retain(|id, _index| remaining_node_ids.contains(id));
        debug!("Removed stale node_index_by_id: {:?}", start.elapsed());

        // Cleanup the node indices by lineage id map.
        self.node_indices_by_lineage_id
            .iter_mut()
            .for_each(|(_lineage_id, node_indices)| {
                node_indices.retain(|node_index| remaining_node_indices.contains(node_index));
            });
        self.node_indices_by_lineage_id
            .retain(|_lineage_id, node_indices| !node_indices.is_empty());
        debug!(
            "Removed stale node_indices_by_lineage_id: {:?}",
            start.elapsed()
        );
    }

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

    #[allow(dead_code)]
    pub fn dot(&self) {
        // NOTE(nick): copy the output and execute this on macOS. It will create a file in the
        // process and open a new tab in your browser.
        // ```
        // pbpaste | dot -Tsvg -o foo.svg && open foo.svg
        // ```
        let current_root_weight = self
            .get_node_weight(self.root_index)
            .expect("this should be impossible and this code should only be used for debugging");
        println!(
            "Root Node Weight: {current_root_weight:?}\n{:?}",
            petgraph::dot::Dot::with_config(&self.graph, &[petgraph::dot::Config::EdgeNoLabel])
        );
    }

    /// Produces a subgraph of self that includes only the parent and child trees
    /// of `subgraph_root`. Useful for producing a manageable slice of the graph
    /// for debugging.
    pub fn subgraph(&self, subgraph_root: NodeIndex) -> Option<Self> {
        let mut subgraph: StableDiGraph<NodeWeight, EdgeWeight> = StableDiGraph::new();
        let mut index_map = HashMap::new();
        let mut node_index_by_id = HashMap::new();
        let mut node_indices_by_lineage_id = HashMap::new();
        let mut new_root = None;

        let mut add_node_to_idx = |node_id: Ulid, lineage_id: Ulid, node_idx: NodeIndex| {
            node_index_by_id.insert(node_id, node_idx);
            node_indices_by_lineage_id
                .entry(lineage_id)
                .and_modify(|set: &mut HashSet<NodeIndex>| {
                    set.insert(node_idx);
                })
                .or_insert_with(|| HashSet::from([node_idx]));
        };

        let mut parent_q = VecDeque::from([subgraph_root]);
        let node_weight = self.graph.node_weight(subgraph_root)?.to_owned();
        let sub_id = node_weight.id();
        let sub_lineage_id = node_weight.lineage_id();
        let new_subgraph_root = subgraph.add_node(node_weight);
        add_node_to_idx(sub_id, sub_lineage_id, new_subgraph_root);
        index_map.insert(subgraph_root, new_subgraph_root);

        // Walk to parent
        while let Some(node_idx) = parent_q.pop_front() {
            let mut has_parents = false;
            for edge_ref in self.edges_directed(node_idx, Incoming) {
                has_parents = true;
                let source_idx = edge_ref.source();
                let source_node_weight = self.graph.node_weight(source_idx)?.to_owned();
                let id = source_node_weight.id();
                let lineage_id = source_node_weight.lineage_id();
                let new_source_idx = match index_map.get(&source_idx).copied() {
                    Some(node_idx) => node_idx,
                    None => {
                        let new_source_idx = subgraph.add_node(source_node_weight);
                        index_map.insert(source_idx, new_source_idx);
                        node_index_by_id.insert(id, new_source_idx);
                        node_indices_by_lineage_id
                            .entry(lineage_id)
                            .and_modify(|set: &mut HashSet<NodeIndex>| {
                                set.insert(new_source_idx);
                            })
                            .or_insert_with(|| HashSet::from([new_source_idx]));

                        new_source_idx
                    }
                };

                let edge_weight = edge_ref.weight().to_owned();

                let current_node_idx_in_sub = index_map.get(&node_idx).copied()?;
                if subgraph
                    .find_edge(new_source_idx, current_node_idx_in_sub)
                    .is_none()
                {
                    subgraph.add_edge(new_source_idx, current_node_idx_in_sub, edge_weight);
                }

                parent_q.push_back(source_idx);
            }
            if !has_parents {
                new_root = Some(index_map.get(&node_idx).copied()?);
            }
        }

        // Walk to leaves from subgraph_root
        let mut child_q: VecDeque<NodeIndex> = VecDeque::from([subgraph_root]);
        while let Some(node_idx) = child_q.pop_front() {
            for edge_ref in self.edges_directed(node_idx, Outgoing) {
                let target_idx = edge_ref.target();
                let target_node_weight = self.graph.node_weight(target_idx)?.to_owned();
                let id = target_node_weight.id();
                let lineage_id = target_node_weight.lineage_id();
                let new_target_idx = match index_map.get(&target_idx).copied() {
                    Some(node_idx) => node_idx,
                    None => {
                        let new_target_idx = subgraph.add_node(target_node_weight);
                        index_map.insert(target_idx, new_target_idx);
                        node_index_by_id.insert(id, new_target_idx);
                        node_indices_by_lineage_id
                            .entry(lineage_id)
                            .and_modify(|set: &mut HashSet<NodeIndex>| {
                                set.insert(new_target_idx);
                            })
                            .or_insert_with(|| HashSet::from([new_target_idx]));

                        new_target_idx
                    }
                };

                let edge_weight = edge_ref.weight().to_owned();
                let current_node_idx_in_sub = index_map.get(&edge_ref.source()).copied()?;
                if subgraph
                    .find_edge(current_node_idx_in_sub, new_target_idx)
                    .is_none()
                {
                    subgraph.add_edge(current_node_idx_in_sub, new_target_idx, edge_weight);
                }

                child_q.push_back(target_idx);
            }
        }

        Some(Self {
            graph: subgraph,
            node_index_by_id,
            node_indices_by_lineage_id,
            root_index: new_root?,
            ..Default::default()
        })
    }

    /// Write the graph to disk. This *MAY PANIC*. Use only for debugging.
    #[allow(clippy::disallowed_methods)]
    pub fn write_to_disk(&self, file_suffix: &str) {
        let (serialized, _) = serialize::to_vec(self).expect("unable to serialize");
        let filename = format!("{}-{}", Ulid::new(), file_suffix);

        let home_env = std::env::var("HOME").expect("No HOME environment variable set");
        let home = std::path::Path::new(&home_env);
        let mut file = File::create(home.join(&filename)).expect("could not create file");
        file.write_all(&serialized).expect("could not write file");

        println!("Wrote graph to {}", home.join(&filename).display());
    }

    #[allow(clippy::disallowed_methods)]
    pub fn tiny_dot_to_file(&self, suffix: Option<&str>) {
        let suffix = suffix.unwrap_or("dot");
        // NOTE(nick): copy the output and execute this on macOS. It will create a file in the
        // process and open a new tab in your browser.
        // ```
        // GRAPHFILE=<filename-without-extension>; cat $GRAPHFILE.txt | dot -Tsvg -o processed-$GRAPHFILE.svg; open processed-$GRAPHFILE.svg
        // ```
        let dot = petgraph::dot::Dot::with_attr_getters(
            &self.graph,
            &[
                petgraph::dot::Config::NodeNoLabel,
                petgraph::dot::Config::EdgeNoLabel,
            ],
            &|_, edgeref| {
                let discrim: EdgeWeightKindDiscriminants = edgeref.weight().kind().into();
                let color = match discrim {
                    EdgeWeightKindDiscriminants::Action => "black",
                    EdgeWeightKindDiscriminants::ActionPrototype => "black",
                    EdgeWeightKindDiscriminants::ApprovalRequirementDefinition => "black",
                    EdgeWeightKindDiscriminants::AuthenticationPrototype => "black",
                    EdgeWeightKindDiscriminants::Contain => "blue",
                    EdgeWeightKindDiscriminants::DiagramObject => "black",
                    EdgeWeightKindDiscriminants::FrameContains => "black",
                    EdgeWeightKindDiscriminants::ManagementPrototype => "pink",
                    EdgeWeightKindDiscriminants::Manages => "pink",
                    EdgeWeightKindDiscriminants::Ordering => "gray",
                    EdgeWeightKindDiscriminants::Ordinal => "gray",
                    EdgeWeightKindDiscriminants::Prop => "orange",
                    EdgeWeightKindDiscriminants::Prototype => "green",
                    EdgeWeightKindDiscriminants::PrototypeArgument => "green",
                    EdgeWeightKindDiscriminants::PrototypeArgumentValue => "green",
                    EdgeWeightKindDiscriminants::Proxy => "gray",
                    EdgeWeightKindDiscriminants::Represents => "black",
                    EdgeWeightKindDiscriminants::Root => "black",
                    EdgeWeightKindDiscriminants::Socket => "red",
                    EdgeWeightKindDiscriminants::SocketValue => "purple",
                    EdgeWeightKindDiscriminants::Use => "black",
                    EdgeWeightKindDiscriminants::ValidationOutput => "darkcyan",
                };

                match edgeref.weight().kind() {
                    EdgeWeightKind::Contain(key) => {
                        let key = key
                            .as_deref()
                            .map(|key| format!(" ({key}"))
                            .unwrap_or("".into());
                        format!(
                            "label = \"{discrim:?}{key}\"\nfontcolor = {color}\ncolor = {color}"
                        )
                    }
                    _ => format!("label = \"{discrim:?}\"\nfontcolor = {color}\ncolor = {color}"),
                }
            },
            &|_, (node_index, node_weight)| {
                let (label, color) = match node_weight {
                    NodeWeight::Action(_) => ("Action".to_string(), "cyan"),
                    NodeWeight::ActionPrototype(_) => ("Action Prototype".to_string(), "cyan"),
                    NodeWeight::Content(weight) => {
                        let discrim = ContentAddressDiscriminants::from(weight.content_address());
                        let color = match discrim {
                            // Some of these should never happen as they have their own top-level
                            // NodeWeight variant.
                            ContentAddressDiscriminants::ActionPrototype => "green",
                            ContentAddressDiscriminants::ApprovalRequirementDefinition => "black",
                            ContentAddressDiscriminants::AttributePrototype => "green",
                            ContentAddressDiscriminants::Component => "black",
                            ContentAddressDiscriminants::DeprecatedAction => "green",
                            ContentAddressDiscriminants::DeprecatedActionBatch => "green",
                            ContentAddressDiscriminants::DeprecatedActionRunner => "green",
                            ContentAddressDiscriminants::Func => "black",
                            ContentAddressDiscriminants::FuncArg => "black",
                            ContentAddressDiscriminants::Geometry => "black",
                            ContentAddressDiscriminants::InputSocket => "red",
                            ContentAddressDiscriminants::JsonValue => "fuchsia",
                            ContentAddressDiscriminants::ManagementPrototype => "black",
                            ContentAddressDiscriminants::Module => "yellow",
                            ContentAddressDiscriminants::OutputSocket => "red",
                            ContentAddressDiscriminants::Prop => "orange",
                            ContentAddressDiscriminants::Root => "black",
                            ContentAddressDiscriminants::Schema => "black",
                            ContentAddressDiscriminants::SchemaVariant => "black",
                            ContentAddressDiscriminants::Secret => "black",
                            ContentAddressDiscriminants::StaticArgumentValue => "green",
                            ContentAddressDiscriminants::ValidationOutput => "darkcyan",
                            ContentAddressDiscriminants::ValidationPrototype => "black",
                            ContentAddressDiscriminants::View => "black",
                        };
                        (discrim.to_string(), color)
                    }
                    NodeWeight::AttributePrototypeArgument(apa) => (
                        format!(
                            "Attribute Prototype Argument{}",
                            apa.targets()
                                .map(|targets| format!(
                                    "\nsource: {}\nto: {}",
                                    targets.source_component_id, targets.destination_component_id
                                ))
                                .unwrap_or("".to_string())
                        ),
                        "green",
                    ),
                    NodeWeight::AttributeValue(_) => ("Attribute Value".to_string(), "blue"),
                    NodeWeight::Category(category_node_weight) => match category_node_weight.kind()
                    {
                        CategoryNodeKind::Action => ("Actions (Category)".to_string(), "black"),
                        CategoryNodeKind::Component => {
                            ("Components (Category)".to_string(), "black")
                        }
                        CategoryNodeKind::DeprecatedActionBatch => {
                            ("Action Batches (Category)".to_string(), "black")
                        }
                        CategoryNodeKind::Func => ("Funcs (Category)".to_string(), "black"),
                        CategoryNodeKind::Schema => ("Schemas (Category)".to_string(), "black"),
                        CategoryNodeKind::Secret => ("Secrets (Category)".to_string(), "black"),
                        CategoryNodeKind::Module => ("Modules (Category)".to_string(), "black"),
                        CategoryNodeKind::DependentValueRoots => {
                            ("Dependent Values (Category)".into(), "black")
                        }
                        CategoryNodeKind::View => ("Views (Category)".into(), "black"),
                        CategoryNodeKind::DiagramObject => {
                            ("Diagram Objects (Category)".into(), "black")
                        }
                    },
                    NodeWeight::Component(component) => (
                        "Component".to_string(),
                        if component.to_delete() {
                            "gray"
                        } else {
                            "black"
                        },
                    ),
                    NodeWeight::Func(func_node_weight) => {
                        (format!("Func\n{}", func_node_weight.name()), "black")
                    }
                    NodeWeight::FuncArgument(func_arg_node_weight) => (
                        format!("Func Arg\n{}", func_arg_node_weight.name()),
                        "black",
                    ),
                    NodeWeight::Geometry(_) => ("Geometry\n".to_string(), "green"),
                    NodeWeight::InputSocket(_) => ("Input Socket".to_string(), "black"),
                    NodeWeight::Ordering(_) => {
                        (NodeWeightDiscriminants::Ordering.to_string(), "gray")
                    }
                    NodeWeight::Prop(prop_node_weight) => {
                        (format!("Prop\n{}", prop_node_weight.name()), "orange")
                    }
                    NodeWeight::SchemaVariant(_) => ("Schema Variant".to_string(), "black"),
                    NodeWeight::Secret(secret_node_weight) => (
                        format!("Secret\n{}", secret_node_weight.encrypted_secret_key()),
                        "black",
                    ),
                    NodeWeight::DependentValueRoot(node_weight) => (
                        format!("UnfinishedDependentValue\n{}", node_weight.value_id()),
                        "purple",
                    ),
                    NodeWeight::FinishedDependentValueRoot(node_weight) => (
                        format!("FinishedDependentValue\n{}", node_weight.value_id()),
                        "red",
                    ),
                    NodeWeight::View(_) => ("View\n".to_string(), "black"),
                    NodeWeight::ManagementPrototype(_) => {
                        ("ManagementPrototype".to_string(), "black")
                    }
                    NodeWeight::DiagramObject(_) => ("DiagramObject".to_string(), "black"),
                    NodeWeight::ApprovalRequirementDefinition(_) => {
                        ("ApprovalRequirementDefinition".to_string(), "black")
                    }
                };
                let color = color.to_string();
                let id = node_weight.id();
                format!(
                    "label = \"\n\n{label}\n{node_index:?}\n{id}\n\n{:?}\n{:?}\"\nfontcolor = {color}\ncolor = {color}",
                    node_weight.merkle_tree_hash(),
                    node_weight.node_hash(),
                )
            },
        );
        let filename_no_extension = format!("{}-{}", Ulid::new(), suffix);

        let home_str = std::env::var("HOME").expect("could not find home directory via env");
        let home = std::path::Path::new(&home_str);

        let mut file = File::create(home.join(format!("{filename_no_extension}.txt")))
            .expect("could not create file");
        file.write_all(format!("{dot:?}").as_bytes())
            .expect("could not write file");
        println!("dot output stored in file (filename without extension: {filename_no_extension})");
    }

    #[inline(always)]
    pub fn get_node_index_by_id(
        &self,
        id: impl Into<Ulid>,
    ) -> WorkspaceSnapshotGraphResult<NodeIndex> {
        let id = id.into();

        self.node_index_by_id
            .get(&id)
            .copied()
            .ok_or(WorkspaceSnapshotGraphError::NodeWithIdNotFound(id))
    }

    #[inline(always)]
    pub fn get_node_index_by_id_opt(&self, id: impl Into<Ulid>) -> Option<NodeIndex> {
        let id = id.into();

        self.node_index_by_id.get(&id).copied()
    }

    pub fn get_node_index_by_lineage(&self, lineage_id: Ulid) -> HashSet<NodeIndex> {
        self.node_indices_by_lineage_id
            .get(&lineage_id)
            .cloned()
            .unwrap_or_default()
    }

    pub fn node_index_to_id(&self, node_idx: NodeIndex) -> Option<Ulid> {
        self.graph
            .node_weight(node_idx)
            .map(|node_weight| node_weight.id())
    }

    pub fn get_node_weight_opt(&self, node_index: NodeIndex) -> Option<&NodeWeight> {
        self.graph.node_weight(node_index)
    }

    pub fn get_node_weight(
        &self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<&NodeWeight> {
        self.get_node_weight_opt(node_index)
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)
    }

    pub fn get_node_weight_by_id_opt(&self, id: impl Into<Ulid>) -> Option<&NodeWeight> {
        self.get_node_index_by_id_opt(id)
            .and_then(|index| self.get_node_weight_opt(index))
    }

    /// Gets a mutable reference to the node weight at `node_index`. If you
    /// modify this node, you must also touch it by calling `Self::touch_node`
    /// so that its merkle tree hash and the merkle tree hash of its parents are
    /// both updated.
    fn get_node_weight_mut(
        &mut self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<&mut NodeWeight> {
        self.graph
            .node_weight_mut(node_index)
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)
    }

    pub fn get_edge_weight_opt(
        &self,
        edge_index: EdgeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<&EdgeWeight>> {
        Ok(self.graph.edge_weight(edge_index))
    }

    pub fn import_component_subgraph(
        &mut self,
        other: &WorkspaceSnapshotGraphV3,
        component_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<()> {
        // * DFS event-based traversal.
        //   * DfsEvent::Discover(attribute_prototype_argument_node_index, _):
        //     If APA has targets, skip & return Control::Prune, since we don't want to bring in
        //     Components other than the one specified. Only arguments linking Inout & Output
        //     Sockets will have targets (the source & destination ComponentIDs).
        //   * DfsEvent::Discover(func_node_index, _):
        //     Add edge from Funcs Category node to imported Func node.
        let mut edges_by_tail = HashMap::new();
        petgraph::visit::depth_first_search(&other.graph, Some(component_node_index), |event| {
            self.import_component_subgraph_process_dfs_event(other, &mut edges_by_tail, event)
        })?;

        Ok(())
    }

    /// This assumes that the SchemaVariant for the Component is already present in [`self`][Self].
    fn import_component_subgraph_process_dfs_event(
        &mut self,
        other: &WorkspaceSnapshotGraphV3,
        edges_by_tail: &mut HashMap<NodeIndex, Vec<(NodeIndex, EdgeWeight)>>,
        event: DfsEvent<NodeIndex>,
    ) -> WorkspaceSnapshotGraphResult<petgraph::visit::Control<()>> {
        match event {
            // We only check to see if we can prune graph traversal in the node discovery event.
            // The "real" work is done in the node finished event.
            DfsEvent::Discover(other_node_index, _) => {
                let other_node_weight = other.get_node_weight(other_node_index)?;

                // AttributePrototypeArguments with targets connect Input & Output Sockets, and we
                // don't want to import either the Component on the other end of the connection, or
                // the connection itself. Unfortunately, we can't prune when looking at the
                // relevant edge, as that would prune _all remaining edges_ outgoing from the
                // AttributePrototype.
                if NodeWeightDiscriminants::AttributePrototypeArgument == other_node_weight.into() {
                    let apa_node_weight =
                        other_node_weight.get_attribute_prototype_argument_node_weight()?;
                    if apa_node_weight.targets().is_some() {
                        return Ok(petgraph::visit::Control::Prune);
                    }
                }

                // When we hit something that already exists, we're pretty much guaranteed to have
                // left "the component" and have gone into already-existing Funcs or the Schema
                // Variant.
                if self
                    .find_equivalent_node(other_node_weight.id(), other_node_weight.lineage_id())?
                    .is_some()
                {
                    return Ok(petgraph::visit::Control::Prune);
                }

                Ok(petgraph::visit::Control::Continue)
            }
            // We wait to do the "real" import work in the Finish event as these happen in
            // post-order, so we are guaranteed that all of the targets of the outgoing edges
            // have been imported.
            DfsEvent::Finish(other_node_index, _) => {
                // See if we already have the node from other in self.
                let other_node_weight = other.get_node_weight(other_node_index)?;
                // Even though we prune when the equivalent node is_some() in the Discover event,
                // we will still get a Finish event for the node that returned Control::Prune in
                // its Discover event.
                if self
                    .find_equivalent_node(other_node_weight.id(), other_node_weight.lineage_id())?
                    .is_none()
                {
                    // AttributePrototypeArguments for cross-component connections will still have
                    // their DfsEvent::Finish fire, and won't already exist in self, but we do not
                    // want to import them.
                    if let NodeWeight::AttributePrototypeArgument(
                        attribute_prototype_argument_node_weight,
                    ) = other_node_weight
                    {
                        if attribute_prototype_argument_node_weight.targets().is_some() {
                            return Ok(petgraph::visit::Control::Prune);
                        }
                    }

                    // Import the node.
                    self.add_or_replace_node(other_node_weight.clone())?;

                    // Create all edges with this node as the tail.
                    if let Entry::Occupied(edges) = edges_by_tail.entry(other_node_index) {
                        for (other_head_node_index, edge_weight) in edges.get() {
                            // Need to get this on every iteration as the node index changes as we
                            // add edges.
                            let self_node_index =
                                self.get_node_index_by_id(other_node_weight.id())?;
                            let other_head_node_weight =
                                other.get_node_weight(*other_head_node_index)?;
                            let self_head_node_index =
                                self.get_node_index_by_id(other_head_node_weight.id())?;
                            self.add_edge(
                                self_node_index,
                                edge_weight.clone(),
                                self_head_node_index,
                            )?;
                        }
                    }

                    // Funcs and Components have incoming edges from their Category nodes that we
                    // want to exist, but won't be discovered by the graph traversal on its own.
                    let category_kind = if NodeWeightDiscriminants::Func == other_node_weight.into()
                    {
                        Some(CategoryNodeKind::Func)
                    } else if NodeWeightDiscriminants::Component == other_node_weight.into() {
                        Some(CategoryNodeKind::Component)
                    } else {
                        None
                    };

                    if let Some(category_node_kind) = category_kind {
                        let self_node_index = self.get_node_index_by_id(other_node_weight.id())?;
                        let (_, category_node_idx) = self
                            .get_category_node(None, category_node_kind)?
                            .ok_or(WorkspaceSnapshotGraphError::CategoryNodeNotFound(
                                CategoryNodeKind::Func,
                            ))?;
                        self.add_edge(
                            category_node_idx,
                            EdgeWeight::new(EdgeWeightKind::new_use()),
                            self_node_index,
                        )?;
                    }
                }

                Ok(petgraph::visit::Control::Continue)
            }
            DfsEvent::BackEdge(tail_node_index, head_node_index)
            | DfsEvent::CrossForwardEdge(tail_node_index, head_node_index)
            | DfsEvent::TreeEdge(tail_node_index, head_node_index) => {
                // We'll keep track of the edges we encounter, instead of doing something with them
                // right away as the common case (TreeEdge) is that we'll encounter the edge before
                // we encounter the node for the head (target) of the edge. We can't actually do
                // anything about importing the edge until we've also imported the head node.
                for edgeref in other
                    .graph
                    .edges_connecting(tail_node_index, head_node_index)
                {
                    edges_by_tail
                        .entry(tail_node_index)
                        .and_modify(|entry| {
                            entry.push((head_node_index, edgeref.weight().clone()));
                        })
                        .or_insert_with(|| vec![(head_node_index, edgeref.weight().clone())]);
                }
                Ok(petgraph::visit::Control::Continue)
            }
        }
    }

    pub fn is_acyclic_directed(&self) -> bool {
        // Using this because "is_cyclic_directed" is recursive.
        algo::toposort(&self.graph, None).is_ok()
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    /// Returns an `Option<Vec<NodeIndex>>`. If there is an ordering node, then the return will be a
    /// [`Some`], where the [`Vec`] is populated with the [`NodeIndex`] of the nodes specified by
    /// the ordering node, in the order defined by the ordering node. If there is not an ordering
    /// node, then the return will be [`None`].
    pub fn ordered_children_for_node(
        &self,
        container_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<Vec<NodeIndex>>> {
        let mut ordered_child_indexes = Vec::new();
        if let Some(container_ordering_index) =
            self.ordering_node_index_for_container(container_node_index)?
        {
            if let NodeWeight::Ordering(ordering_weight) =
                self.get_node_weight(container_ordering_index)?
            {
                for ordered_id in ordering_weight.order() {
                    if let Some(child_index) = self.node_index_by_id.get(ordered_id).copied() {
                        ordered_child_indexes.push(child_index);
                    }
                }
            }
        } else {
            return Ok(None);
        }

        Ok(Some(ordered_child_indexes))
    }

    pub fn ordering_node_for_container(
        &self,
        container_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<OrderingNodeWeight>> {
        Ok(
            match self.ordering_node_index_for_container(container_node_index)? {
                Some(ordering_node_idx) => match self.get_node_weight_opt(ordering_node_idx) {
                    Some(node_weight) => Some(node_weight.get_ordering_node_weight()?.clone()),
                    None => None,
                },
                None => None,
            },
        )
    }

    pub fn ordering_node_index_for_container(
        &self,
        container_node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<NodeIndex>> {
        let onto_ordering_node_indexes =
            ordering_node_indexes_for_node_index(self, container_node_index);
        if onto_ordering_node_indexes.len() > 1 {
            error!(
                "Too many ordering nodes found for container NodeIndex {:?}",
                container_node_index
            );
            return Err(WorkspaceSnapshotGraphError::TooManyOrderingForNode(
                container_node_index,
            ));
        }
        Ok(onto_ordering_node_indexes.first().copied())
    }

    pub fn prop_node_index_for_node_index(
        &self,
        node_index: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<Option<NodeIndex>> {
        let prop_node_indexes = prop_node_indexes_for_node_index(self, node_index);
        if prop_node_indexes.len() > 1 {
            error!("Too many prop nodes found for NodeIndex {:?}", node_index);
            return Err(WorkspaceSnapshotGraphError::TooManyPropForNode(node_index));
        }
        Ok(prop_node_indexes.first().copied())
    }

    /// Removes the node from the graph. Edges to this node will be
    /// automatically removed by petgraph. Be sure to remove the node id from
    /// the mappings with `Self::remove_node_id`
    pub fn remove_node(&mut self, node_index: NodeIndex) {
        let incoming_sources: Vec<_> = self
            .graph
            .neighbors_directed(node_index, Incoming)
            .collect();

        // We have to be sure that we recalculate the merkle tree hash for every
        // node that had an outgoing edge to this node
        for incoming in incoming_sources {
            self.touch_node(incoming);
        }

        self.graph.remove_node(node_index);
    }

    /// Removes an edge of the specified kind between `source_node_index` and
    /// `target_node_index`.
    ///
    /// If the source node has an associated ordering node, the function also
    /// removes the edge from the ordering node to the target node, updating the
    /// ordering node's order
    pub fn remove_edge(
        &mut self,
        source_node_index: NodeIndex,
        target_node_index: NodeIndex,
        edge_kind: EdgeWeightKindDiscriminants,
    ) -> WorkspaceSnapshotGraphResult<()> {
        self.remove_edge_of_kind(source_node_index, target_node_index, edge_kind);
        self.touch_node(source_node_index);

        if let Some(container_ordering_node_idx) =
            self.ordering_node_index_for_container(source_node_index)?
        {
            let element_id = self
                .node_index_to_id(target_node_index)
                .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?;

            if let NodeWeight::Ordering(container_ordering_node_weight) =
                self.get_node_weight_mut(container_ordering_node_idx)?
            {
                if container_ordering_node_weight.remove_from_order(element_id) {
                    self.remove_edge_of_kind(
                        container_ordering_node_idx,
                        target_node_index,
                        EdgeWeightKindDiscriminants::Ordinal,
                    );
                    self.touch_node(container_ordering_node_idx);
                }
            }
        }

        Ok(())
    }

    fn remove_edge_of_kind(
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

    pub fn update_content(
        &mut self,
        id: Ulid,
        new_content_hash: ContentHash,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let node_index = self.get_node_index_by_id(id)?;
        let node_weight = self.get_node_weight_mut(node_index)?;
        node_weight.new_content_hash(new_content_hash)?;
        self.touch_node(node_index);
        Ok(())
    }

    pub fn update_order(
        &mut self,
        container_id: Ulid,
        new_order: Vec<Ulid>,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let node_index = self
            .ordering_node_index_for_container(self.get_node_index_by_id(container_id)?)?
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?;
        let node_weight = self.get_node_weight_mut(node_index)?;
        node_weight.set_order(new_order)?;
        self.touch_node(node_index);

        Ok(())
    }

    fn update_merkle_tree_hash(
        &mut self,
        node_index_to_update: NodeIndex,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let mut hasher = MerkleTreeHash::hasher();
        hasher.update(
            self.get_node_weight(node_index_to_update)?
                .node_hash()
                .to_string()
                .as_bytes(),
        );

        // Need to make sure that ordered containers have their ordered children in the
        // order specified by the ordering graph node.
        let explicitly_ordered_children = self
            .ordered_children_for_node(node_index_to_update)?
            .unwrap_or_default();

        // Need to make sure the unordered neighbors are added to the hash in a stable order to
        // ensure the merkle tree hash is identical for identical trees.
        let mut unordered_neighbors = Vec::new();
        for neighbor_node in self
            .graph
            .neighbors_directed(node_index_to_update, Outgoing)
        {
            // Only add the neighbor if it's not one of the ones with an explicit ordering.
            if !explicitly_ordered_children.contains(&neighbor_node) {
                if let Some(neighbor_id) = self.node_index_to_id(neighbor_node) {
                    unordered_neighbors.push((neighbor_id, neighbor_node));
                }
            }
        }
        // We'll sort the neighbors by the ID in the NodeWeight, as that will result in more stable
        // results than if we sorted by the NodeIndex itself.
        unordered_neighbors.sort_by_cached_key(|(id, _index)| *id);
        // It's not important whether the explicitly ordered children are first or last, as long as
        // they are always in that position, and are always in the sequence specified by the
        // container's Ordering node.
        let mut ordered_neighbors =
            Vec::with_capacity(explicitly_ordered_children.len() + unordered_neighbors.len());
        ordered_neighbors.extend(explicitly_ordered_children);
        ordered_neighbors.extend::<Vec<NodeIndex>>(
            unordered_neighbors
                .iter()
                .map(|(_id, index)| *index)
                .collect(),
        );

        for neighbor_node in ordered_neighbors {
            hasher.update(
                self.get_node_weight(neighbor_node)?
                    .merkle_tree_hash()
                    .to_string()
                    .as_bytes(),
            );

            // The edge(s) between `node_index_to_update`, and `neighbor_node` potentially encode
            // important information related to the "identity" of `node_index_to_update`.
            for connecting_edgeref in self
                .graph
                .edges_connecting(node_index_to_update, neighbor_node)
            {
                match connecting_edgeref.weight().kind() {
                    // This is the key for an entry in a map.
                    EdgeWeightKind::Contain(Some(key)) => hasher.update(key.as_bytes()),

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
                    | EdgeWeightKind::ActionPrototype
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
                    | EdgeWeightKind::Represents
                    | EdgeWeightKind::Root
                    | EdgeWeightKind::SocketValue
                    | EdgeWeightKind::ValidationOutput
                    | EdgeWeightKind::ManagementPrototype
                    | EdgeWeightKind::Manages
                    | EdgeWeightKind::DiagramObject
                    | EdgeWeightKind::ApprovalRequirementDefinition => {}
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

    /// Does a depth first post-order walk to recalculate the entire merkle tree
    /// hash. This operation can be expensive, so should be done only when we
    /// know it needs to be (like when migrating snapshots between versions)
    pub fn recalculate_entire_merkle_tree_hash(&mut self) -> WorkspaceSnapshotGraphResult<()> {
        let mut dfs = petgraph::visit::DfsPostOrder::new(&self.graph, self.root_index);

        while let Some(node_index) = dfs.next(&self.graph) {
            self.update_merkle_tree_hash(node_index)?;
        }

        Ok(())
    }

    /// Does a dfs post-order walk of the DAG, recalculating the merkle tree
    /// hash for any nodes we have touched while working on the graph. Should be
    /// more efficient than recalculating the entire merkle tree hash, since we
    /// will only update the hash for the branches of the graph that have been
    /// touched and thus need to be recalculated.
    pub fn recalculate_entire_merkle_tree_hash_based_on_touched_nodes(
        &mut self,
    ) -> WorkspaceSnapshotGraphResult<()> {
        let mut dfs = petgraph::visit::DfsPostOrder::new(&self.graph, self.root_index);

        let mut discovered_nodes = HashSet::new();

        while let Some(node_index) = dfs.next(&self.graph) {
            if self.touched_node_indices.contains(&node_index)
                || discovered_nodes.contains(&node_index)
            {
                self.update_merkle_tree_hash(node_index)?;
                self.graph
                    .neighbors_directed(node_index, Incoming)
                    .for_each(|node_idx| {
                        discovered_nodes.insert(node_idx);
                    });
            }
        }

        self.touched_node_indices.clear();

        Ok(())
    }

    pub fn perform_updates(&mut self, updates: &[Update]) -> WorkspaceSnapshotGraphResult<()> {
        for update in updates {
            match update {
                Update::NewEdge {
                    source,
                    destination,
                    edge_weight,
                } => {
                    let source_idx = self.get_node_index_by_id_opt(source.id);
                    let destination_idx = self.get_node_index_by_id_opt(destination.id);

                    if let (Some(source_idx), Some(destination_idx)) = (source_idx, destination_idx)
                    {
                        if let EdgeWeightKind::Use { is_default: true } = edge_weight.kind() {
                            ensure_only_one_default_use_edge(self, source_idx)?;
                        }

                        self.add_edge_inner(
                            source_idx,
                            edge_weight.clone(),
                            destination_idx,
                            false,
                        )?;
                    }
                }
                Update::RemoveEdge {
                    source,
                    destination,
                    edge_kind,
                } => {
                    let source_idx = self.get_node_index_by_id_opt(source.id);
                    let destination_idx = self.get_node_index_by_id_opt(destination.id);

                    if let (Some(source_idx), Some(destination_idx)) = (source_idx, destination_idx)
                    {
                        self.remove_edge(source_idx, destination_idx, *edge_kind)?;
                    }
                }
                Update::NewNode { node_weight } => {
                    if self.get_node_index_by_id_opt(node_weight.id()).is_none() {
                        self.add_or_replace_node(node_weight.to_owned())?;
                    }
                }
                Update::ReplaceNode { node_weight } => {
                    if self.get_node_index_by_id_opt(node_weight.id()).is_some() {
                        self.add_or_replace_node(node_weight.to_owned())?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Update node weight in place with a lambda. Use with caution. Generally
    /// we treat node weights as immutable and replace them by creating a new
    /// node with a new node weight and replacing references to point to the new
    /// node.
    #[allow(dead_code)]
    pub(crate) fn update_node_weight<L>(
        &mut self,
        node_idx: NodeIndex,
        lambda: L,
    ) -> WorkspaceSnapshotGraphResult<()>
    where
        L: FnOnce(&mut NodeWeight) -> WorkspaceSnapshotGraphResult<()>,
    {
        let node_weight = self
            .graph
            .node_weight_mut(node_idx)
            .ok_or(WorkspaceSnapshotGraphError::NodeWeightNotFound)?;

        lambda(node_weight)?;
        self.touch_node(node_idx);

        Ok(())
    }
}

fn ordering_node_indexes_for_node_index(
    snapshot: &WorkspaceSnapshotGraphV3,
    node_index: NodeIndex,
) -> Vec<NodeIndex> {
    snapshot
        .graph
        .edges_directed(node_index, Outgoing)
        .filter_map(|edge_reference| {
            if edge_reference.weight().kind() == &EdgeWeightKind::Ordering
                && matches!(
                    snapshot.get_node_weight(edge_reference.target()),
                    Ok(NodeWeight::Ordering(_))
                )
            {
                return Some(edge_reference.target());
            }

            None
        })
        .collect()
}

fn prop_node_indexes_for_node_index(
    snapshot: &WorkspaceSnapshotGraphV3,
    node_index: NodeIndex,
) -> Vec<NodeIndex> {
    snapshot
        .graph
        .edges_directed(node_index, Outgoing)
        .filter_map(|edge_reference| {
            if edge_reference.weight().kind() == &EdgeWeightKind::Prop
                && matches!(
                    snapshot.get_node_weight(edge_reference.target()),
                    Ok(NodeWeight::Prop(_))
                )
            {
                return Some(edge_reference.target());
            }
            None
        })
        .collect()
}

fn ensure_only_one_default_use_edge(
    graph: &mut WorkspaceSnapshotGraphV3,
    source_idx: NodeIndex,
) -> WorkspaceSnapshotGraphResult<()> {
    let existing_default_targets: Vec<NodeIndex> = graph
        .edges_directed(source_idx, Outgoing)
        .filter(|edge_ref| {
            matches!(
                edge_ref.weight().kind(),
                EdgeWeightKind::Use { is_default: true }
            )
        })
        .map(|edge_ref| edge_ref.target())
        .collect();

    for target_idx in existing_default_targets {
        graph.remove_edge(source_idx, target_idx, EdgeWeightKindDiscriminants::Use)?;
        graph.add_edge_inner(
            source_idx,
            EdgeWeight::new(EdgeWeightKind::new_use()),
            target_idx,
            false,
        )?;
    }

    Ok(())
}
