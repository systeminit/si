use std::env::args;
use tokio::fs;

use si_pkg::{PkgSpec, SiPkg};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = args();
    let input = args.nth(1).expect("usage: program <JSON_FILE> <TARBALL>");
    let tar_file = args.next().expect("usage: program <JSON_FILE> <TARBALL>");

    let spec: PkgSpec = {
        let buf = fs::read_to_string(&input).await?;
        serde_json::from_str(&buf)?
    };
    let pkg = SiPkg::load_from_spec(spec)?;

    println!("--- Writing pkg to: {tar_file}");
    fs::write(&tar_file, pkg.write_to_bytes()?).await?;

    println!("--- Done.");
    Ok(())
}
