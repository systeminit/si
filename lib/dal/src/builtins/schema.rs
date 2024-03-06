use si_pkg::SiPkg;
use telemetry::prelude::*;

use crate::installed_pkg::InstalledPkg;
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
    if InstalledPkg::find_by_hash(ctx, &root_hash).await?.is_none() {
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

// /// A _private_ item containing useful metadata alongside a [`FuncId`](crate::Func). This is used by
// /// the [`MigrationDriver`].
// #[derive(Copy, Clone, Debug)]
// pub struct FuncCacheItem {
//     pub func_id: FuncId,
//     pub func_binding_id: FuncBindingId,
//     pub func_binding_return_value_id: FuncBindingReturnValueId,
//     pub func_argument_id: FuncArgumentId,
// }
//
// /// This _private_ driver providing caches and helper methods for efficiently creating builtin
// /// [`Schemas`](crate::Schema).
// #[derive(Default)]
// pub struct MigrationDriver {
//     pub func_item_cache: HashMap<String, FuncCacheItem>,
//     pub func_id_cache: HashMap<String, FuncId>,
// }
//
// impl MigrationDriver {
//     /// Create a [`driver`](Self) with commonly used, cached data.
//     pub async fn new(ctx: &DalContext) -> BuiltinsResult<Self> {
//         let mut driver = Self::default();
//
//         driver
//             .add_func_item(
//                 ctx,
//                 "si:identity".to_string(),
//                 serde_json::json![{ "identity": null }],
//                 "identity".to_string(),
//                 vec![],
//             )
//             .await?;
//
//         for builtin_func_name in ["si:validation"] {
//             driver
//                 .add_func_id(ctx, builtin_func_name.to_string())
//                 .await?;
//         }
//
//         Ok(driver)
//     }
//
//     /// Add a `FuncCacheItem` for a given [`Func`](crate::Func) name.
//     pub async fn add_func_item(
//         &mut self,
//         ctx: &DalContext,
//         func_name: String,
//         func_binding_args: Value,
//         func_argument_name: String,
//         before: Vec<BeforeFunction>,
//     ) -> BuiltinsResult<()> {
//         let func: Func = Func::find_by_attr(ctx, "name", &func_name)
//             .await?
//             .pop()
//             .ok_or_else(|| FuncError::NotFoundByName(func_name.clone()))?;
//         let func_id = *func.id();
//         let (func_binding, func_binding_return_value) =
//             FuncBinding::create_and_execute(ctx, func_binding_args, func_id, before).await?;
//         let func_argument = FuncArgument::find_by_name_for_func(ctx, &func_argument_name, func_id)
//             .await?
//             .ok_or_else(|| {
//                 BuiltinsError::BuiltinMissingFuncArgument(func_name.clone(), func_argument_name)
//             })?;
//         self.func_item_cache.insert(
//             func_name,
//             FuncCacheItem {
//                 func_id,
//                 func_binding_id: *func_binding.id(),
//                 func_binding_return_value_id: *func_binding_return_value.id(),
//                 func_argument_id: *func_argument.id(),
//             },
//         );
//
//         Ok(())
//     }
//
//     /// Add a [`FuncId`](crate::Func) for a given [`Func`](crate::Func) name.
//     pub async fn add_func_id(&mut self, ctx: &DalContext, func_name: String) -> BuiltinsResult<()> {
//         let func = Func::find_by_attr(ctx, "name", &func_name)
//             .await?
//             .pop()
//             .ok_or_else(|| FuncError::NotFoundByName(func_name.clone()))?;
//         self.func_id_cache.insert(func_name, *func.id());
//         Ok(())
//     }
//
//     /// Get a `FuncCacheItem` (from the cache) for a given [`Func`](crate::Func) name.
//     pub fn get_func_item(&self, name: impl AsRef<str>) -> Option<FuncCacheItem> {
//         self.func_item_cache.get(name.as_ref()).copied()
//     }
//
//     /// Get a [`FuncId`](crate::Func) (from the cache) for a given [`Func`](crate::Func) name.
//     pub fn get_func_id(&self, name: impl AsRef<str>) -> Option<FuncId> {
//         self.func_id_cache.get(name.as_ref()).copied()
//     }
//
//     /// Find a single [`Func`](crate::Func) and [`FuncArgument`](crate::FuncArgument) by providing
//     /// the name for each, respectively.
//     pub async fn find_func_and_single_argument_by_names(
//         &self,
//         ctx: &DalContext,
//         func_name: &str,
//         func_argument_name: &str,
//     ) -> BuiltinsResult<(FuncId, FuncArgumentId)> {
//         Self::find_func_and_single_argument_by_names_raw(ctx, func_name, func_argument_name).await
//     }
//
//     pub async fn find_func_and_single_argument_by_names_raw(
//         ctx: &DalContext,
//         func_name: &str,
//         func_argument_name: &str,
//     ) -> BuiltinsResult<(FuncId, FuncArgumentId)> {
//         // NOTE(nick): we may eventually want to make "self" mutable and perform auto caching.
//         let func_name = func_name.to_string();
//         let func = Func::find_by_attr(ctx, "name", &func_name)
//             .await?
//             .pop()
//             .ok_or_else(|| SchemaError::FuncNotFound(func_name.clone()))?;
//         let func_id = *func.id();
//         let func_argument = FuncArgument::find_by_name_for_func(ctx, func_argument_name, func_id)
//             .await?
//             .ok_or_else(|| {
//                 BuiltinsError::BuiltinMissingFuncArgument(func_name, func_argument_name.to_string())
//             })?;
//         Ok((func_id, *func_argument.id()))
//     }
// }
