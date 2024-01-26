use si_pkg::SiPkg;
use std::collections::HashSet;
use strum::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;

use crate::builtins::schema::test_exclusive_schema_fallout::migrate_test_exclusive_schema_fallout;
use crate::builtins::schema::test_exclusive_schema_starfield::migrate_test_exclusive_schema_starfield;
use crate::installed_pkg::InstalledPkg;
use crate::pkg::{import_pkg_from_pkg, ImportOptions};
use crate::{BuiltinsError, BuiltinsResult, DalContext, SelectedTestBuiltinSchemas};

mod test_exclusive_schema_fallout;
mod test_exclusive_schema_starfield;

/// Migrate [`Schemas`](crate::Schema) for production use.
pub async fn migrate_local_all_schemas(ctx: &DalContext) -> BuiltinsResult<()> {
    info!("migrating schemas");

    migrate_pkg(ctx, super::SI_AWS_PKG, None).await?;
    migrate_pkg(ctx, super::SI_AWS_EC2_PKG, None).await?;
    migrate_pkg(ctx, super::SI_DOCKER_IMAGE_PKG, None).await?;
    migrate_pkg(ctx, super::SI_COREOS_PKG, None).await?;
    migrate_pkg(ctx, super::SI_GENERIC_FRAME_PKG, None).await?;
    migrate_pkg(ctx, super::SI_AWS_IAM_PKG, None).await?;
    migrate_pkg(ctx, super::SI_AWS_ECS_PKG, None).await?;
    migrate_pkg(ctx, super::SI_AWS_CLOUDWATCH_PKG, None).await?;
    migrate_pkg(ctx, super::SI_AWS_LB_TARGET_GROUP_PKG, None).await?;

    Ok(())
}

/// Migrate [`Schemas`](crate::Schema) for use in tests.
pub async fn migrate_local_only_test_schemas(
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

    if migrate_all {
        migrate_pkg(ctx, super::SI_AWS_PKG, None).await?;
        migrate_pkg(ctx, super::SI_AWS_EC2_PKG, None).await?;
        migrate_pkg(ctx, super::SI_COREOS_PKG, None).await?;
        migrate_pkg(ctx, super::SI_DOCKER_IMAGE_PKG, None).await?;
        migrate_pkg(ctx, super::SI_GENERIC_FRAME_PKG, None).await?;
        migrate_pkg(ctx, super::SI_AWS_LB_TARGET_GROUP_PKG, None).await?;

        migrate_pkg_test_exclusive(ctx, TestExclusiveSchema::Fallout).await?;
        migrate_pkg_test_exclusive(ctx, TestExclusiveSchema::Starfield).await?;
    } else if migrate_test_exclusive {
        // We migrate generic frame to get "si:resourceToPayloadValue" cheaply. This function
        // should be converted to an intrinsic (or removed?)
        migrate_pkg(ctx, super::SI_GENERIC_FRAME_PKG, None).await?;

        migrate_pkg_test_exclusive(ctx, TestExclusiveSchema::Fallout).await?;
        migrate_pkg_test_exclusive(ctx, TestExclusiveSchema::Starfield).await?;
    } else {
        let schemas: Vec<String> = specific_builtin_schemas
            .iter()
            .map(|s| s.to_owned())
            .collect();
        migrate_pkg(ctx, super::SI_AWS_PKG, Some(schemas.to_owned())).await?;
        migrate_pkg(ctx, super::SI_AWS_EC2_PKG, Some(schemas.to_owned())).await?;
        migrate_pkg(ctx, super::SI_COREOS_PKG, Some(schemas.to_owned())).await?;
        migrate_pkg(ctx, super::SI_DOCKER_IMAGE_PKG, Some(schemas.to_owned())).await?;
        migrate_pkg(ctx, super::SI_GENERIC_FRAME_PKG, Some(schemas.to_owned())).await?;
        migrate_pkg(
            ctx,
            super::SI_AWS_LB_TARGET_GROUP_PKG,
            Some(schemas.to_owned()),
        )
        .await?;

        for test_schema in [TestExclusiveSchema::Starfield, TestExclusiveSchema::Fallout] {
            if specific_builtin_schemas.contains(test_schema.real_schema_name()) {
                migrate_pkg_test_exclusive(ctx, test_schema).await?;
            }
        }
    }

    Ok(())
}

async fn migrate_pkg(
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
            true,
        )
        .await?;
    }

    Ok(())
}

async fn migrate_pkg_test_exclusive(
    ctx: &DalContext,
    schema: TestExclusiveSchema,
) -> BuiltinsResult<()> {
    match schema {
        TestExclusiveSchema::Fallout => {
            migrate_test_exclusive_schema_fallout(ctx).await?;
        }
        TestExclusiveSchema::Starfield => {
            migrate_test_exclusive_schema_starfield(ctx).await?;
        }
    }
    ctx.blocking_commit().await?;
    Ok(())
}

/// Test exclusive [`Schema`](crate::Schema) are solely used for "dal" integration tests.
#[remain::sorted]
#[derive(Debug, Copy, Clone, AsRefStr, Display, EnumIter, EnumString, Eq, PartialEq)]
enum TestExclusiveSchema {
    Fallout,
    Starfield,
}

impl TestExclusiveSchema {
    pub fn real_schema_name(&self) -> &'static str {
        match self {
            TestExclusiveSchema::Fallout => "fallout",
            TestExclusiveSchema::Starfield => "starfield",
        }
    }
}
