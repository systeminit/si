use std::{env, fs::File, io::prelude::*};

use si_layer_cache::db::serialize;

use dal::WorkspaceSnapshotGraphV2;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error + 'static>>;

const USAGE: &str = "usage: cargo run --example rebase <TO_REBASE_FILE_PATH> <ONTO_FILE_PATH>";

fn load_snapshot_graph(path: &str) -> Result<WorkspaceSnapshotGraphV2> {
    let mut file = File::open(path)?;
    let mut bytes = vec![];
    file.read_to_end(&mut bytes)?;

    Ok(serialize::from_bytes(&bytes)?)
}

fn main() -> Result<()> {
    let args: Vec<String> = env::args().take(3).map(Into::into).collect();
    let to_rebase_path = args.get(1).expect(USAGE);
    let onto_path = args.get(2).expect(USAGE);

    let to_rebase_graph = load_snapshot_graph(to_rebase_path)?;
    let onto_graph = load_snapshot_graph(onto_path)?;

    let updates = to_rebase_graph.detect_updates(&onto_graph);

    for update in &updates {
        dbg!(update);
    }

    Ok(())
}

// 01J2F7HKZFMTN6GVKXE73044AT
