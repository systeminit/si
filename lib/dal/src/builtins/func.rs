use si_pkg::SiPkg;

use crate::module::Module;
use crate::{
    BuiltinsResult, DalContext, func::intrinsics::IntrinsicFunc, pkg::import_pkg_from_pkg,
};
use telemetry::prelude::*;

/// We want the src/builtins/func/** files to be available at run time inside of the Docker container
/// that we build, but it would be nice to not have to include arbitrary bits of the source tree when
/// building it. This allows us to compile the builtins into the binary, so they're already available
/// in memory.
///
/// The instances of this end up in a magic `ASSETS` array const.
#[allow(dead_code)]
#[iftree::include_file_tree("paths = '/src/builtins/func/**'")]
pub struct FuncBuiltin {
    relative_path: &'static str,
    contents_str: &'static str,
}

#[allow(dead_code)]
static FUNC_BUILTIN_BY_PATH: once_cell::sync::Lazy<std::collections::HashMap<&str, &FuncBuiltin>> =
    once_cell::sync::Lazy::new(|| {
        ASSETS
            .iter()
            .map(|func_builtin| (func_builtin.relative_path, func_builtin))
            .collect()
    });

pub async fn migrate_intrinsics_for_tests(ctx: &DalContext) -> BuiltinsResult<()> {
    let intrinsics_pkg_spec = IntrinsicFunc::pkg_spec()?;
    let _name = intrinsics_pkg_spec.name.to_owned();
    let intrinsics_pkg = SiPkg::load_from_spec(intrinsics_pkg_spec)?;

    if Module::find_by_root_hash(ctx, &intrinsics_pkg.hash()?.to_string())
        .await?
        .is_none()
    {
        info!("importing");
        import_pkg_from_pkg(ctx, &intrinsics_pkg, None).await?;
        info!("imported, commiting");
        ctx.blocking_commit().await?;
        info!("commit finished");
    }

    Ok(())
}

#[instrument(skip_all)]
pub async fn migrate_intrinsics_no_commit(ctx: &DalContext) -> BuiltinsResult<()> {
    let intrinsics_pkg_spec = IntrinsicFunc::pkg_spec()?;
    let intrinsics_pkg = SiPkg::load_from_spec(intrinsics_pkg_spec)?;
    import_pkg_from_pkg(ctx, &intrinsics_pkg, None).await?;
    Ok(())
}
