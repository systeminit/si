use std::{env, fs::File, io::prelude::*};

use si_layer_cache::db::serialize;

use dal::{
    workspace_snapshot::node_weight::NodeWeight, NodeWeightDiscriminants, WorkspaceSnapshotGraphV1,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static>>;

const USAGE: &str = "usage: cargo run --example rebase <TO_REBASE_FILE_PATH> <ONTO_FILE_PATH>";

fn load_snapshot_graph(path: &str) -> Result<WorkspaceSnapshotGraphV1> {
    let mut file = File::open(path)?;
    let mut bytes = vec![];
    file.read_to_end(&mut bytes)?;

    Ok(serialize::from_bytes(&bytes)?)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().take(3).map(Into::into).collect();
    let to_rebase_path = args.get(1).expect(USAGE);
    let onto_path = args.get(2).expect(USAGE);

    let mut to_rebase_graph = load_snapshot_graph(&to_rebase_path)?;
    let onto_graph = load_snapshot_graph(&onto_path)?;

    let to_rebase_vector_clock_id = dbg!(to_rebase_graph
        .max_recently_seen_clock_id(None)
        .expect("Unable to find a vector clock id in to_rebase"));
    let onto_vector_clock_id = dbg!(onto_graph
        .max_recently_seen_clock_id(None)
        .expect("Unable to find a vector clock id in onto"));

    let conflicts_and_updates = to_rebase_graph.detect_conflicts_and_updates(
        dbg!(to_rebase_vector_clock_id),
        &onto_graph,
        onto_vector_clock_id,
    )?;

    let mut last_ordering_node = None;
    for update in &conflicts_and_updates.updates {
        match update {
            dal::workspace_snapshot::update::Update::NewEdge {
                source,
                destination,
                ..
            } => {
                if matches!(source.node_weight_kind, NodeWeightDiscriminants::Ordering) {
                    if let Some(ordering_node) = &last_ordering_node {
                        if let NodeWeight::Ordering(ordering) = ordering_node {
                            dbg!(destination, ordering.order());
                        }
                    }
                }
            }
            dal::workspace_snapshot::update::Update::RemoveEdge { .. } => {}
            dal::workspace_snapshot::update::Update::ReplaceSubgraph { onto, .. } => {
                if matches!(onto.node_weight_kind, NodeWeightDiscriminants::Ordering) {
                    last_ordering_node = onto_graph
                        .get_node_weight_opt(onto.index)
                        .expect("couldn't get node")
                        .map(ToOwned::to_owned);
                }
            }
            dal::workspace_snapshot::update::Update::MergeCategoryNodes { .. } => {}
        }
    }

    dbg!(to_rebase_graph.perform_updates(
        to_rebase_vector_clock_id,
        &onto_graph,
        &conflicts_and_updates.updates,
    ))?;

    Ok(())
}

// 01J2F7HKZFMTN6GVKXE73044AT
