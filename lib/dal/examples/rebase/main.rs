use std::{
    env,
    fs::File,
    io::{
        Read as _,
        Write as _,
    },
};

use dal::{
    WorkspaceSnapshotGraph,
    workspace_snapshot::{
        node_weight::NodeWeight,
        split_snapshot::{
            SplitSnapshotGraphV1,
            SubGraphV1,
        },
    },
};
use petgraph::visit::{
    Dfs,
    IntoNeighborsDirected,
    IntoNodeIdentifiers,
    Reversed,
    VisitMap,
    Visitable,
};
use serde::de::DeserializeOwned;
use si_layer_cache::db::serialize;
use si_split_graph::SuperGraph;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static>>;

const USAGE: &str = "usage: cargo run --example rebase <TO_REBASE_FILE_PATH> <REBASE_BATCH_PATH>";

fn load_serialized_stuff<T: DeserializeOwned>(path: &str) -> Result<T> {
    println!("opening file: {path}");
    let mut file = File::open(path)?;
    let mut bytes = vec![];
    file.read_to_end(&mut bytes)?;

    Ok(serialize::from_bytes(&bytes)?)
}

#[allow(unused)]
fn write_snapshot_graph(path: &str, graph: &WorkspaceSnapshotGraph) -> Result<()> {
    let mut file = File::create(path)?;
    let (bytes, _) = serialize::to_vec(graph)?;
    file.write_all(&bytes)?;

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().take(10).collect();

    for i in 1..args.len() {
        let snapshot_folder = args.get(i).expect(USAGE);

        let mut read_only_graph = read_graph(snapshot_folder, true)?;
        read_only_graph.cleanup_and_merkle_tree_hash();
        let mut working_copy = read_graph(snapshot_folder, false)?;
        working_copy.cleanup_and_merkle_tree_hash();

        assert!(toposort(&read_only_graph).is_ok());
        assert!(toposort(&working_copy).is_ok());

        let updates = read_only_graph.detect_updates(&working_copy);
        let mut graph_to_update = read_only_graph.clone();
        graph_to_update.perform_updates(&updates);

        if let Err(Cycle(node_id)) = toposort(&graph_to_update) {
            println!("Cycle detected at node ID: {node_id}");
            // dbg!(
            //     graph_to_update
            //         .raw_nodes()
            //         .find(|node| node.id() == node_id)
            // );
            // dbg!(graph_to_update.brute_search_external_source_edges(node_id));
            // dbg!(
            //     read_only_graph
            //         .raw_nodes()
            //         .find(|node| node.id() == node_id)
            // );
            // dbg!(working_copy.raw_nodes().find(|node| node.id() == node_id));
            //     for subgraph in working_copy.subgraphs() {
            //         let mut subgraph_copy = subgraph.clone();
            //         if !removed.is_empty() {
            //             dbg!(removed);
            //             let externals: Vec<_> = subgraph_copy.graph().externals(Incoming).collect();
            //             for external in externals {
            //                 dbg!(subgraph_copy.graph().node_weight(external));
            //             }
            //             let externals: Vec<_> = subgraph.graph().externals(Incoming).collect();
            //             for external in externals {
            //                 dbg!(subgraph.graph().node_weight(external));
            //             }
            //         }
            //     }
            //     println!("detecto");
            //     working_copy
            //         .edges_directed(node_id, Incoming)?
            //         .for_each(|edge| {
            //             dbg!(edge);
            //         });
            //     println!("detectNO!");

            //     dbg!(working_copy.brute_search_external_source_edges(node_id));
            //     dbg!(updates.iter().find(|update| match update {
            //         si_split_graph::Update::NewNode { node_weight, .. } => node_weight.id() == node_id,
            //         si_split_graph::Update::NewEdge { edge_weight, .. } => match edge_weight {
            //             si_split_graph::SplitGraphEdgeWeight::ExternalSource { source_id, .. } =>
            //                 *source_id == node_id,
            //             _ => false,
            //         },
            //         _ => false,
            //     }));
        }
    }

    Ok(())
}

fn read_graph(
    snapshot_folder: &str,
    read_only: bool,
) -> Result<si_split_graph::SplitGraph<NodeWeight, dal::EdgeWeight, dal::EdgeWeightKindDiscriminants>>
{
    let prefix = if read_only {
        "read_only"
    } else {
        "working_copy"
    };

    let supergraph: SuperGraph =
        load_serialized_stuff(&format!("{snapshot_folder}/{prefix}_supergraph.snapshot"))?;
    let mut subgraphs = vec![];
    for i in 0..supergraph.addresses().len() {
        let subgraph: SubGraphV1 =
            load_serialized_stuff(&format!("{snapshot_folder}/{prefix}_subgraph_{i}.snapshot"))?;
        subgraphs.push(subgraph);
    }
    let split_graph = SplitSnapshotGraphV1::from_parts(supergraph, subgraphs);

    Ok(split_graph)
}

#[derive(Debug, Eq, PartialEq)]
pub struct Cycle<N>(N);

pub fn toposort<G>(g: G) -> std::result::Result<Vec<G::NodeId>, Cycle<G::NodeId>>
where
    G: IntoNeighborsDirected + IntoNodeIdentifiers + Visitable,
{
    // based on kosaraju scc
    let mut dfs = Dfs::empty(g);

    dfs.reset(g);
    let mut finished = g.visit_map();

    let mut finish_stack = Vec::new();
    for i in g.node_identifiers() {
        if dfs.discovered.is_visited(&i) {
            continue;
        }
        dfs.stack.push(i);
        while let Some(&nx) = dfs.stack.last() {
            if dfs.discovered.visit(nx) {
                // First time visiting `nx`: Push neighbors, don't pop `nx`
                for succ in g.neighbors(nx) {
                    if succ == nx {
                        // self cycle
                        return Err(Cycle(nx));
                    }
                    if !dfs.discovered.is_visited(&succ) {
                        dfs.stack.push(succ);
                    }
                }
            } else {
                dfs.stack.pop();
                if finished.visit(nx) {
                    // Second time: All reachable nodes must have been finished
                    finish_stack.push(nx);
                }
            }
        }
    }
    finish_stack.reverse();

    dfs.reset(g);
    for &i in &finish_stack {
        dfs.move_to(i);
        let mut cycle = false;
        while let Some(j) = dfs.next(Reversed(g)) {
            if cycle {
                return Err(Cycle(j));
            }
            cycle = true;
        }
    }

    Ok(finish_stack)
}
