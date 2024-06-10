use std::{collections::HashMap, env, fs::File, io::prelude::*};

use si_layer_cache::db::serialize;

use dal::{
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        node_weight::{NodeWeight, NodeWeightDiscriminants},
        vector_clock::HasVectorClocks,
    },
    EdgeWeightKindDiscriminants, WorkspaceSnapshotGraph,
};
use tokio::time::Instant;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static>>;

#[tokio::main]
async fn main() -> Result<()> {
    let mut args = env::args();
    let snap_path = args.nth(1).expect("usage: program <SNAPSHOT_FILE_PATH>");

    let mut snap_file = File::open(snap_path)?;

    let mut snap_bytes = vec![];
    snap_file.read_to_end(&mut snap_bytes)?;

    println!("snap bytes compressed: {}", snap_bytes.len());
    let decompressed = serialize::decompress_to_vec(&snap_bytes)?;
    println!("decompressed: {}", decompressed.len());

    let now = Instant::now();
    let graph: WorkspaceSnapshotGraph = serialize::from_bytes(&snap_bytes)?;
    println!("deserialization took: {:?}", now.elapsed());
    let inner_graph = graph.graph();

    let mut edge_kind_counts = HashMap::new();
    let mut node_kind_counts = HashMap::new();

    let mut edge_count = 0;
    let mut node_count = 0;
    let mut edge_vector_clock_first_seen_entries = 0;
    let mut edge_vector_clock_write_entries = 0;

    for edge_weight in inner_graph.edge_weights() {
        edge_count += 1;
        edge_vector_clock_first_seen_entries += edge_weight.vector_clock_first_seen().len();
        edge_vector_clock_write_entries += edge_weight.vector_clock_write().len();

        let kind: EdgeWeightKindDiscriminants = edge_weight.kind().into();
        let kind_string = format!("{:?}", kind);

        edge_kind_counts
            .entry(kind_string)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    let mut node_first_seen_entries = 0;
    let mut node_recently_seen_entries = 0;
    let mut node_write_entries = 0;

    for node_weight in inner_graph.node_weights() {
        node_count += 1;

        node_first_seen_entries += node_weight.vector_clock_first_seen().len();
        node_recently_seen_entries += node_weight.vector_clock_recently_seen().len();
        node_write_entries += node_weight.vector_clock_write().len();

        let kind_string = {
            if let NodeWeight::Content(content_node) = node_weight {
                let cad_discrim: ContentAddressDiscriminants =
                    content_node.content_address().into();
                cad_discrim.to_string()
            } else {
                let kind: NodeWeightDiscriminants = node_weight.into();
                kind.to_string()
            }
        };

        node_kind_counts
            .entry(kind_string)
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    println!("edges: {edge_count}, nodes: {node_count}");

    println!(
        "\nedge vector clock first seen entries: {edge_vector_clock_first_seen_entries}, {} per edge",
        edge_vector_clock_first_seen_entries / edge_count
    );
    println!(
        "edge vector clock write entries: {edge_vector_clock_write_entries}, {} per edge",
        edge_vector_clock_write_entries / edge_count
    );

    // 128 bit id, 64 bit timestamp = 24bytes
    let rough_bytes = (edge_vector_clock_first_seen_entries + edge_vector_clock_write_entries) * 24;

    println!(
        "edge vector clocks are ~{} bytes, which is {}% of the total snapshot",
        rough_bytes,
        (rough_bytes as f64 / decompressed.len() as f64) * 100.0
    );

    println!("\nEdge kinds:");

    for (k, v) in edge_kind_counts {
        println!("\t{k}: {v}");
    }

    println!(
        "\nnode vector clock first seen entries: {node_first_seen_entries}, {} per node",
        node_first_seen_entries / node_count
    );
    println!(
        "node vector clock recently seen entries: {node_recently_seen_entries}, {} per node",
        node_recently_seen_entries / node_count
    );
    println!(
        "node vector clock write entries: {node_write_entries}, {} per node",
        node_write_entries / node_count
    );

    println!("\nNode kinds:");

    for (k, v) in node_kind_counts {
        println!("\t{k}: {v}");
    }

    Ok(())
}
