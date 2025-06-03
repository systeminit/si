use std::{
    env,
    fs::File,
    io::{
        Read as _,
        Write,
    },
};

use dal::{
    WorkspaceSnapshotGraph,
    workspace_snapshot::graph::validator::{
        WithGraph,
        connections::connection_migrations,
    },
};
use si_layer_cache::db::serialize;
use tokio::time::Instant;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static>>;

#[tokio::main]
async fn main() -> Result<()> {
    // To fix a snapshot, first download the snapshot from the admin portal
    // Then run this like so in dal/examples:
    // cargo run --example snapshot-fixer <PATH>
    // ex: cargo run --example snapshot-fixer ~/Downloads/head.snapshot
    let mut args = env::args();
    let snap_path = args.nth(1).expect("usage: program <SNAPSHOT_FILE_PATH>");

    let mut snap_file = File::open(snap_path)?;

    let mut snap_bytes = vec![];
    snap_file.read_to_end(&mut snap_bytes)?;

    println!("snap bytes compressed: {}", snap_bytes.len());
    let decompressed = serialize::decompress_to_vec(&snap_bytes)?;
    println!("decompressed: {}", decompressed.len());

    let now = Instant::now();
    let mut graph: WorkspaceSnapshotGraph = serialize::from_bytes(&snap_bytes)?;
    println!("deserialization took: {:?}", now.elapsed());
    // Make your edits:
    // Example: We were seeing:
    // attribute value error: attribute value 01JTXGMYKFFPY7H2ZNV7SKFQ9X has no outgoing edge to a prop or socket"
    // so we needed to remove it
    // let node_id = "01JTXGMYKFFPY7H2ZNV7SKFQ9X";
    // remove_node_by_id(&mut graph, node_id)?;

    for migration in connection_migrations(&graph, vec![]) {
        println!("{}", WithGraph(&graph, &migration));
    }
    // for issue in validate_graph(&graph)? {
    //     println!("{}", WithGraph(&graph, &issue));
    //     // Only fix ConnectionToUnknownSocket issues for now
    //     if let issue @ ValidationIssue::ConnectionToUnknownSocket { .. } = issue {
    //         issue.fix(&mut graph)?
    //     }
    // }
    // for issue in validate_graph(graph)? {
    //     // println!("{}", issue.with_graph(&graph));
    //     // Only fix ConnectionToUnknownSocket issues for now
    //     match issue {
    //         _issue @ ValidationIssue::ConnectionToUnknownSocket { .. } => {} // issue.fix(&mut graph)?,
    //         _ => {}
    //     }
    // }

    // Cleanup and update merkle tree
    graph.cleanup_and_merkle_tree_hash()?;

    // write snapshot
    write_snapshot_graph("./fixed.snapshot", &graph)?;

    // then head back to the admin portal and replace snapshot with this new fixed snapshot
    Ok(())
}

fn write_snapshot_graph(path: &str, graph: &WorkspaceSnapshotGraph) -> Result<()> {
    let mut file = File::create(path)?;
    let (bytes, _) = serialize::to_vec(graph)?;
    file.write_all(&bytes)?;

    Ok(())
}

#[allow(unused)]
fn remove_node_by_id(graph: &mut WorkspaceSnapshotGraph, id: &str) -> Result<()> {
    let node_id = si_id::ulid::Ulid::from_string(id)?;
    let node_idx = graph.get_node_index_by_id(node_id)?;
    graph.remove_node(node_idx);
    Ok(())
}
