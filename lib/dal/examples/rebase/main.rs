use std::{env, fs::File, io::prelude::*};

use serde::de::DeserializeOwned;
use si_layer_cache::db::serialize;

use dal::{
    workspace_snapshot::graph::{correct_transforms::correct_transforms, RebaseBatch},
    WorkspaceSnapshotGraphV2,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static>>;

const USAGE: &str = "usage: cargo run --example rebase <TO_REBASE_FILE_PATH> <REBASE_BATCH_PATH>";

fn load_snapshot_graph<T: DeserializeOwned>(path: &str) -> Result<T> {
    let mut file = File::open(path)?;
    let mut bytes = vec![];
    file.read_to_end(&mut bytes)?;

    Ok(serialize::from_bytes(&bytes)?)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().take(3).map(Into::into).collect();
    let to_rebase_path = args.get(1).expect(USAGE);
    let rebase_batch_path = args.get(2).expect(USAGE);

    let mut to_rebase_graph: WorkspaceSnapshotGraphV2 = load_snapshot_graph(to_rebase_path)?;
    let rebase_batch: RebaseBatch = load_snapshot_graph(rebase_batch_path)?;

    let corrected_transforms =
        correct_transforms(&to_rebase_graph, rebase_batch.updates().to_vec(), false)?;

    to_rebase_graph.perform_updates(&corrected_transforms)?;

    dbg!(to_rebase_graph.node_count());

    Ok(())
}

// 01J2F7HKZFMTN6GVKXE73044AT
