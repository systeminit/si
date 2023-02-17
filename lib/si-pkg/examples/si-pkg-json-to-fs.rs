use std::{env::args, fs};

use object_tree::{ObjectTree, TreeFileSystemWriter};
use si_pkg::schema::Schema;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = args();
    let input = args.nth(1).expect("usage: program <JSON_FILE> <DEST_DIR>");
    let dst = args.next().expect("usage: program <JSON_FILE> <DEST_DIR>");

    let schema: Schema = {
        let buf = fs::read_to_string(&input)?;
        serde_json::from_str(&buf)?
    };
    let tree = ObjectTree::create_from_root(schema.domain.into()).expect("failed to hash tree");

    println!("--- Writing object tree to: {dst}");
    fs::create_dir_all(&dst)?;
    TreeFileSystemWriter::physical(&dst).write(&tree).await?;

    println!("--- Done.");
    Ok(())
}
