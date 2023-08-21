use serde_json::Value;
use si_pkg::SiPkg;
use std::collections::{HashMap, HashSet};
use strum::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;

use crate::func::argument::{FuncArgument, FuncArgumentId};
use crate::installed_pkg::InstalledPkg;
use crate::pkg::{import_pkg_from_pkg, ImportOptions};
use crate::{
    func::{
        binding::{FuncBinding, FuncBindingId},
        binding_return_value::FuncBindingReturnValueId,
    },
    BuiltinsError, BuiltinsResult, DalContext, Func, FuncError, FuncId, SchemaError,
    SelectedTestBuiltinSchemas, StandardModel,
};

mod test_exclusive_fallout;
mod test_exclusive_starfield;

/// Migrate [`Schemas`](crate::Schema) for production use.
pub async fn migrate_for_production(ctx: &DalContext) -> BuiltinsResult<()> {
    info!("migrating schemas");

    migrate_pkg(ctx, super::SI_AWS_PKG, None).await?;
    migrate_pkg(ctx, super::SI_AWS_EC2_PKG, None).await?;
    migrate_pkg(ctx, super::SI_DOCKER_IMAGE_PKG, None).await?;
    migrate_pkg(ctx, super::SI_COREOS_PKG, None).await?;
    migrate_pkg(ctx, super::SI_GENERIC_FRAME_PKG, None).await?;
    migrate_pkg(ctx, super::SI_AWS_IAM_PKG, None).await?;

    Ok(())
}

#[remain::sorted]
#[derive(Debug, Copy, Clone, AsRefStr, Display, EnumIter, EnumString, Eq, PartialEq)]
pub enum BuiltinSchema {
    Fallout,
    Starfield,
}

impl BuiltinSchema {
    pub fn real_schema_name(&self) -> &'static str {
        match self {
            BuiltinSchema::Fallout => "fallout",
            BuiltinSchema::Starfield => "starfield",
        }
    }
}

pub async fn migrate_schema(
    ctx: &DalContext,
    schema: BuiltinSchema,
    driver: &MigrationDriver,
) -> BuiltinsResult<()> {
    match schema {
        BuiltinSchema::Fallout => {
            driver.migrate_test_exclusive_fallout(ctx).await?;
        }
        BuiltinSchema::Starfield => {
            driver.migrate_test_exclusive_starfield(ctx).await?;
        }
    }

    Ok(())
}

/// Migrate [`Schemas`](crate::Schema) for use in tests.
pub async fn migrate_for_tests(
    ctx: &DalContext,
    selected_test_builtin_schemas: SelectedTestBuiltinSchemas,
) -> BuiltinsResult<()> {
    // Determine what to migrate based on the selected test builtin schemas provided.
    let (migrate_all, migrate_test_exclusive, specific_builtin_schemas) =
        match selected_test_builtin_schemas {
            SelectedTestBuiltinSchemas::All => {
                info!("migrating schemas for tests");
                (true, false, HashSet::new())
            }
            SelectedTestBuiltinSchemas::None => {
                info!("skipping migrating schemas for tests");
                return Ok(());
            }
            SelectedTestBuiltinSchemas::Some(provided_set) => {
                info!("migrating schemas for tests based on a provided set of names");
                debug!("provided set of builtin schemas: {:?}", &provided_set);
                (false, false, provided_set)
            }
            SelectedTestBuiltinSchemas::Test => {
                info!("migrating test-exclusive schemas solely");
                (false, true, HashSet::new())
            }
        };

    // Once we know what to migrate, create the driver.
    let driver = MigrationDriver::new(ctx).await?;
    ctx.blocking_commit().await?;

    if migrate_all {
        migrate_pkg(ctx, super::SI_AWS_PKG, None).await?;
        migrate_pkg(ctx, super::SI_AWS_EC2_PKG, None).await?;
        migrate_pkg(ctx, super::SI_COREOS_PKG, None).await?;
        migrate_pkg(ctx, super::SI_DOCKER_IMAGE_PKG, None).await?;
        for test_schema in [BuiltinSchema::Starfield, BuiltinSchema::Fallout] {
            migrate_schema(ctx, test_schema, &driver).await?;
            ctx.blocking_commit().await?;
        }
    } else if migrate_test_exclusive {
        for test_schema in [BuiltinSchema::Starfield, BuiltinSchema::Fallout] {
            migrate_schema(ctx, test_schema, &driver).await?;
            ctx.blocking_commit().await?;
        }
    } else {
        let schemas: Vec<String> = specific_builtin_schemas
            .iter()
            .map(|s| s.to_owned())
            .collect();
        migrate_pkg(ctx, super::SI_AWS_PKG, Some(schemas.to_owned())).await?;
        migrate_pkg(ctx, super::SI_AWS_EC2_PKG, Some(schemas.to_owned())).await?;
        migrate_pkg(ctx, super::SI_COREOS_PKG, Some(schemas.to_owned())).await?;
        migrate_pkg(ctx, super::SI_DOCKER_IMAGE_PKG, Some(schemas.to_owned())).await?;
        for test_schema in [BuiltinSchema::Starfield, BuiltinSchema::Fallout] {
            if specific_builtin_schemas.contains(test_schema.real_schema_name()) {
                migrate_schema(ctx, test_schema, &driver).await?;
                ctx.blocking_commit().await?;
            }
        }
    }

    Ok(())
}

pub async fn migrate_pkg(
    ctx: &DalContext,
    pkg_filename: &str,
    schemas: Option<Vec<String>>,
) -> BuiltinsResult<()> {
    let pkgs_path = ctx.pkgs_path().ok_or(BuiltinsError::MissingPkgsPath)?;

    let pkg_path = pkgs_path.join(pkg_filename);
    let pkg = SiPkg::load_from_file(pkg_path).await?;

    let root_hash = pkg.hash()?.to_string();
    if InstalledPkg::find_by_hash(ctx, &root_hash).await?.is_none() {
        import_pkg_from_pkg(
            ctx,
            &pkg,
            pkg_filename,
            schemas.map(|schemas| ImportOptions {
                schemas: Some(schemas),
                ..Default::default()
            }),
        )
        .await?;
    }

    Ok(())
}

/// A _private_ item containing useful metadata alongside a [`FuncId`](crate::Func). This is used by
/// the [`MigrationDriver`].
#[derive(Copy, Clone, Debug)]
pub struct FuncCacheItem {
    pub func_id: FuncId,
    pub func_binding_id: FuncBindingId,
    pub func_binding_return_value_id: FuncBindingReturnValueId,
    pub func_argument_id: FuncArgumentId,
}

/// This _private_ driver providing caches and helper methods for efficiently creating builtin
/// [`Schemas`](crate::Schema).
#[derive(Default)]
pub struct MigrationDriver {
    pub func_item_cache: HashMap<String, FuncCacheItem>,
    pub func_id_cache: HashMap<String, FuncId>,
}

impl MigrationDriver {
    /// Create a [`driver`](Self) with commonly used, cached data.
    pub async fn new(ctx: &DalContext) -> BuiltinsResult<Self> {
        let mut driver = Self::default();

        driver
            .add_func_item(
                ctx,
                "si:identity".to_string(),
                serde_json::json![{ "identity": null }],
                "identity".to_string(),
            )
            .await?;

        for builtin_func_name in ["si:validation"] {
            driver
                .add_func_id(ctx, builtin_func_name.to_string())
                .await?;
        }

        Ok(driver)
    }

    /// Add a `FuncCacheItem` for a given [`Func`](crate::Func) name.
    pub async fn add_func_item(
        &mut self,
        ctx: &DalContext,
        func_name: String,
        func_binding_args: Value,
        func_argument_name: String,
    ) -> BuiltinsResult<()> {
        let func: Func = Func::find_by_attr(ctx, "name", &func_name)
            .await?
            .pop()
            .ok_or_else(|| FuncError::NotFoundByName(func_name.clone()))?;
        let func_id = *func.id();
        let (func_binding, func_binding_return_value) =
            FuncBinding::create_and_execute(ctx, func_binding_args, func_id).await?;
        let func_argument = FuncArgument::find_by_name_for_func(ctx, &func_argument_name, func_id)
            .await?
            .ok_or_else(|| {
                BuiltinsError::BuiltinMissingFuncArgument(func_name.clone(), func_argument_name)
            })?;
        self.func_item_cache.insert(
            func_name,
            FuncCacheItem {
                func_id,
                func_binding_id: *func_binding.id(),
                func_binding_return_value_id: *func_binding_return_value.id(),
                func_argument_id: *func_argument.id(),
            },
        );

        Ok(())
    }

    /// Add a [`FuncId`](crate::Func) for a given [`Func`](crate::Func) name.
    pub async fn add_func_id(&mut self, ctx: &DalContext, func_name: String) -> BuiltinsResult<()> {
        let func = Func::find_by_attr(ctx, "name", &func_name)
            .await?
            .pop()
            .ok_or_else(|| FuncError::NotFoundByName(func_name.clone()))?;
        self.func_id_cache.insert(func_name, *func.id());
        Ok(())
    }

    /// Get a `FuncCacheItem` (from the cache) for a given [`Func`](crate::Func) name.
    pub fn get_func_item(&self, name: impl AsRef<str>) -> Option<FuncCacheItem> {
        self.func_item_cache.get(name.as_ref()).copied()
    }

    /// Get a [`FuncId`](crate::Func) (from the cache) for a given [`Func`](crate::Func) name.
    pub fn get_func_id(&self, name: impl AsRef<str>) -> Option<FuncId> {
        self.func_id_cache.get(name.as_ref()).copied()
    }

    /// Find a single [`Func`](crate::Func) and [`FuncArgument`](crate::FuncArgument) by providing
    /// the name for each, respectively.
    pub async fn find_func_and_single_argument_by_names(
        &self,
        ctx: &DalContext,
        func_name: &str,
        func_argument_name: &str,
    ) -> BuiltinsResult<(FuncId, FuncArgumentId)> {
        Self::find_func_and_single_argument_by_names_raw(ctx, func_name, func_argument_name).await
    }

    pub async fn find_func_and_single_argument_by_names_raw(
        ctx: &DalContext,
        func_name: &str,
        func_argument_name: &str,
    ) -> BuiltinsResult<(FuncId, FuncArgumentId)> {
        // NOTE(nick): we may eventually want to make "self" mutable and perform auto caching.
        let func_name = func_name.to_string();
        let func = Func::find_by_attr(ctx, "name", &func_name)
            .await?
            .pop()
            .ok_or_else(|| SchemaError::FuncNotFound(func_name.clone()))?;
        let func_id = *func.id();
        let func_argument = FuncArgument::find_by_name_for_func(ctx, func_argument_name, func_id)
            .await?
            .ok_or_else(|| {
                BuiltinsError::BuiltinMissingFuncArgument(func_name, func_argument_name.to_string())
            })?;
        Ok((func_id, *func_argument.id()))
    }
}
