use std::{
    env,
    fs::File,
    io::prelude::*,
};

use dal::{
    WorkspaceSnapshotGraph,
    workspace_snapshot::node_weight::NodeWeight,
};
use petgraph::{
    Direction::Incoming,
    visit::EdgeRef,
};
use serde::de::DeserializeOwned;
use si_layer_cache::db::serialize;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static>>;

const USAGE: &str = "usage: cargo run --example rebase <TO_REBASE_FILE_PATH> <REBASE_BATCH_PATH>";

fn load_snapshot_graph<T: DeserializeOwned>(path: &str) -> Result<T> {
    let mut file = File::open(path)?;
    let mut bytes = vec![];
    file.read_to_end(&mut bytes)?;

    Ok(serialize::from_bytes(&bytes)?)
}

fn write_snapshot_graph(path: &str, graph: &WorkspaceSnapshotGraph) -> Result<()> {
    let mut file = File::create(path)?;
    let (bytes, _) = serialize::to_vec(graph)?;
    file.write_all(&bytes)?;

    Ok(())
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().take(10).map(Into::into).collect();

    for i in 1..args.len() {
        let to_rebase_path = args.get(i).expect(USAGE);

        let mut to_rebase_graph: WorkspaceSnapshotGraph = load_snapshot_graph(to_rebase_path)?;

        let mut orphaned_component_idxs = vec![];

        for (node_weight, node_idx) in to_rebase_graph.nodes() {
            if let NodeWeight::Component(_) = node_weight {
                let mut has_edge_to_category = false;
                for edge_ref in to_rebase_graph.edges_directed(node_idx, Incoming) {
                    if let NodeWeight::Category(_) =
                        to_rebase_graph.get_node_weight(edge_ref.source()).unwrap()
                    {
                        has_edge_to_category = true;
                        break;
                    }
                }

                if !has_edge_to_category {
                    orphaned_component_idxs.push(node_idx);
                }
            }
        }

        for orphaned_idx in dbg!(orphaned_component_idxs) {
            to_rebase_graph.remove_node(orphaned_idx);
        }

        to_rebase_graph.cleanup_and_merkle_tree_hash().unwrap();
        let filename = format!("{}.fixed.snapshot", to_rebase_path);
        write_snapshot_graph(&filename, &to_rebase_graph).expect("write");
        println!("wrote {filename}");
    }

    Ok(())
}
