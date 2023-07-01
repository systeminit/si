use std::{env::args, path::Path};
use tokio::fs;

use si_pkg::{PkgSpec, SchemaVariantSpecPropRoot, SiPkg, SiPkgError, SiPkgProp};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = args();
    let input = args.nth(1).expect("usage: program <JSON_FILE> <DEST_DIR>");
    let dst = args.next().expect("usage: program <JSON_FILE> <DEST_DIR>");

    let spec: PkgSpec = {
        let buf = fs::read_to_string(&input).await?;
        serde_json::from_str(&buf)?
    };
    let pkg = SiPkg::load_from_spec(spec)?;

    println!("--- Writing pkg to: {dst}");
    fs::create_dir_all(&dst).await?;
    fs::write(
        Path::new(&dst).join(format!("{}.sipkg", pkg.metadata()?.name())),
        pkg.write_to_bytes()?,
    )
    .await?;

    let schema = pkg.schema_by_name("kuberneteslike")?;
    dbg!(&schema);

    for variant in schema.variants()? {
        variant
            .visit_prop_tree(SchemaVariantSpecPropRoot::Domain, process_prop, None, &())
            .await?;
    }

    println!("--- Done.");
    Ok(())
}

async fn process_prop(
    prop: SiPkgProp<'_>,
    _parent_id: Option<()>,
    _context: &(),
) -> Result<Option<()>, SiPkgError> {
    dbg!(prop);
    Ok(None)
}
