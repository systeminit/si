use std::{env, fs::File, io::Read as _};

use si_layer_cache::db::serialize;

use dal::WorkspaceSnapshotGraph;
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
    let _inner_graph = graph.graph();

    Ok(())
}
