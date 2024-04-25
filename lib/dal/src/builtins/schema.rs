use si_pkg::SiPkg;
use telemetry::prelude::*;

use crate::module::Module;
use crate::pkg::{import_pkg_from_pkg, ImportOptions};
use crate::{BuiltinsError, BuiltinsResult, DalContext};

pub async fn migrate_pkg(
    ctx: &DalContext,
    pkg_filename: &str,
    schemas: Option<Vec<String>>,
) -> BuiltinsResult<()> {
    info!("Migrate: {pkg_filename}");
    let pkgs_path = ctx.pkgs_path().ok_or(BuiltinsError::MissingPkgsPath)?;

    let pkg_path = pkgs_path.join(pkg_filename);
    let pkg = SiPkg::load_from_file(pkg_path).await?;

    let root_hash = pkg.hash()?.to_string();
    if Module::find_by_root_hash(ctx, root_hash).await?.is_none() {
        import_pkg_from_pkg(
            ctx,
            &pkg,
            schemas.map(|schemas| ImportOptions {
                schemas: Some(schemas),
                ..Default::default()
            }),
        )
        .await?;
    }

    Ok(())
}
