use std::env::args;
use tokio::fs;

use petgraph::dot::{Config, Dot};
use si_pkg::SiPkg;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = args();
    let path = args.nth(1).expect("usage: program <DIR>");

    println!("--- Reading object tree from dir: {path}");
    let buf = fs::read(&path).await?;
    let pkg = SiPkg::load_from_bytes(buf)?;

    let (graph, _root_idx) = pkg.as_petgraph();
    println!(
        "\n---- snip ----\n{:?}\n---- snip ----\n",
        Dot::with_config(graph, &[Config::EdgeNoLabel])
    );

    println!("--- Done.");
    Ok(())
}
