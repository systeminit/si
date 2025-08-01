use std::{
    env,
    fs::File,
    io::{
        Read as _,
        Write,
    },
};

use dal::{
    EdgeWeightKind,
    NodeWeightDiscriminants,
    Ulid,
    WorkspaceSnapshotGraph,
    workspace_snapshot::node_weight::NodeWeight,
};
use itertools::Itertools as _;
use petgraph::prelude::*;
use si_layer_cache::db::serialize;
use tokio::time::Instant;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static>>;

#[tokio::main]
async fn main() -> Result<()> {
    // To fix a snapshot, first download the snapshot from the admin portal
    // Then run this like so in dal/examples:
    // cargo run --example snapshot-surfer <PATH> <NODE_ID> ...
    // ex: cargo run --example snapshot-surfer ~/Downloads/head.snapshot 01JTXGMYKFFPY7H2ZNV7SKFQ9X 01JPT6SHX1X43GTM8XHQ6VJRTM
    let mut args = env::args();
    args.next(); // skip program name
    let snap_path = args
        .next()
        .expect("usage: program <SNAPSHOT_FILE_PATH> <NODE_ID> ...");

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
    println!("root id: {}", graph.get_node_weight(graph.root())?.id());
    for arg in args {
        let node_id = Ulid::from_string(&arg)?;
        let node_idx = graph.get_node_index_by_id(node_id)?;
        let ident = node_ident(&graph, node_idx)?;
        println!();
        println!("{}", "=".repeat(ident.len()));
        println!("{ident}");
        println!();
        println!("{:#?}", graph.get_node_weight(node_idx)?);
        print_edges(&graph, node_idx)?;
    }
    // let output_socket =
    //     graph.get_node_index_by_id(Ulid::from_string("01JP40FPPQBGR31K6K4WT896P4")?)?;
    // let component = graph.get_node_index_by_id(Ulid::from_string("01JPT6SHX1X43GTM8XHQ6VJRTM")?)?;
    // print_edges(&graph, output_socket);

    // Cleanup and update merkle tree
    graph.cleanup_and_merkle_tree_hash()?;

    // write snapshot
    write_snapshot_graph("./fixed.snapshot", &graph)?;

    // then head back to the admin portal and replace snapshot with this new fixed snapshot
    Ok(())
}

fn print_edges(graph: &WorkspaceSnapshotGraph, index: NodeIndex) -> Result<()> {
    let incoming: Vec<_> = graph
        .edges_directed(index, Direction::Incoming)
        .map(|edge| {
            node_ident(graph, edge.source())
                .map(|source| format!(" ← {:?} - {}", edge.weight().kind(), source))
        })
        .try_collect()?;
    println!();
    for incoming in incoming {
        println!("  {incoming}");
    }

    println!();
    let outgoing: Vec<_> = graph
        .edges_directed(index, Direction::Outgoing)
        .map(|edge| {
            node_ident(graph, edge.target())
                .map(|target| format!(" → {:?} - {}", edge.weight().kind(), target))
        })
        .try_collect()?;
    for outgoing in outgoing {
        println!("  {outgoing}");
    }
    Ok(())
}

fn node_ident(graph: &WorkspaceSnapshotGraph, index: NodeIndex) -> Result<String> {
    let node = graph.get_node_weight(index)?;
    let discrim = match node.content_address_discriminants() {
        Some(discrim) => discrim.to_string(),
        None => NodeWeightDiscriminants::from(node).to_string(),
    };
    let extra = match node {
        NodeWeight::Category(category) => Some(format!(" ({})", category.kind())),
        NodeWeight::Func(func) => Some(format!(" ({})", func.name())),
        NodeWeight::AttributeValue(_) => match graph.target_opt(index, EdgeWeightKind::Prop)? {
            Some(prop_index) => {
                let prop = graph.get_node_weight(prop_index)?.as_prop_node_weight()?;
                Some(format!(" (prop {} - {})", prop.id(), prop.name()))
            }
            None => None,
        },
        _ => None,
    };
    Ok(format!(
        "{} {}{}",
        discrim,
        node.id(),
        extra.unwrap_or("".to_string())
    ))
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
