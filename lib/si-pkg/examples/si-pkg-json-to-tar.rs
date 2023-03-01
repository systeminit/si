use std::{env::args, fs};

use si_pkg::{spec::Package, SiPkg};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = args();
    let input = args.nth(1).expect("usage: program <JSON_FILE> <TARBALL>");
    let tar_file = args.next().expect("usage: program <JSON_FILE> <TARBALL>");

    let spec: Package = {
        let buf = fs::read_to_string(&input)?;
        serde_json::from_str(&buf)?
    };
    let pkg = SiPkg::load_from_spec(spec)?;

    println!("--- Writing pkg to: {tar_file}");
    pkg.write_to_file(tar_file).await?;

    println!("--- Done.");
    Ok(())
}
