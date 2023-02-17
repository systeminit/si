use std::env::args;

use object_tree::{ObjectTree, TreeFileSystemReader};
use petgraph::dot::{Config, Dot};
use si_pkg::schema::node::PropNode;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = args();
    let src = args.nth(1).expect("usage: program <SRC_DIR>");

    println!("--- Reading object tree from physical file system: {src}");
    let tree: ObjectTree<PropNode> = TreeFileSystemReader::physical(&src).read().await?;

    let (graph, _root_idx) = tree.as_petgraph();
    println!(
        "\n---- snip ----\n{:?}\n---- snip ----\n",
        Dot::with_config(graph, &[Config::EdgeNoLabel])
    );

    println!("--- Done.");
    Ok(())
}
