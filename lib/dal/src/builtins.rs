//! This module contains "builtin" objects that are included with System Initiative.
//! All submodules are private since the only entrypoint to this module should be the
//! [migrate()](crate::builtins::migrate_local()) function. However, they may have some functionality
//! exposed for "dev mode" use cases.

use std::collections::HashSet;
use telemetry::prelude::*;
use thiserror::Error;

use si_pkg::{SiPkgError, SpecError};

use crate::func::FuncError;
use crate::installed_pkg::InstalledPkgError;
use crate::pkg::PkgError;
// use crate::schema::variant::definition::SchemaVariantDefinitionError;
use crate::{
    AttributeValueId, DalContext, ExternalProviderId, InternalProviderId, PropId, SchemaVariantId,
    StandardModelError, TransactionsError,
};

// Private builtins modules.
pub mod func;
pub mod schema;

pub const SI_AWS_PKG: &str = "si-aws-2023-09-13.sipkg";
pub const SI_AWS_EC2_PKG: &str = "si-aws-ec2-2023-09-26.sipkg";
pub const SI_DOCKER_IMAGE_PKG: &str = "si-docker-image-2023-09-13.sipkg";
pub const SI_COREOS_PKG: &str = "si-coreos-2023-09-13.sipkg";
pub const SI_GENERIC_FRAME_PKG: &str = "si-generic-frame-2023-09-13.sipkg";
pub const SI_AWS_IAM_PKG: &str = "si-aws-iam-2023-09-13.sipkg";
pub const SI_AWS_ECS_PKG: &str = "si-aws-ecs-2023-09-21.sipkg";
pub const SI_AWS_CLOUDWATCH_PKG: &str = "si-aws-cloudwatch-2023-09-26.sipkg";
pub const SI_AWS_LB_TARGET_GROUP_PKG: &str = "si-aws-lb-target-group-2023-12-05.sipkg";

#[remain::sorted]
#[derive(Error, Debug)]
pub enum BuiltinsError {
    #[error("attribute value not found by id: {0}")]
    AttributeValueNotFound(AttributeValueId),
    #[error("builtin {0} missing func argument {1}")]
    BuiltinMissingFuncArgument(String, String),
    #[error("explicit internal provider not found by name: {0}")]
    ExplicitInternalProviderNotFound(String),
    #[error("external provider not found by name: {0}")]
    ExternalProviderNotFound(String),
    #[error("Filesystem IO error: {0}")]
    FilesystemIO(#[from] std::io::Error),
    #[error(transparent)]
    Func(#[from] FuncError),
    #[error("json error {1} at file {0}")]
    FuncJson(String, serde_json::Error),
    #[error("Func Metadata error: {0}")]
    FuncMetadata(String),
    #[error("func not found in migration cache {0}")]
    FuncNotFoundInMigrationCache(&'static str),
    #[error("implicit internal provider not found for prop: {0}")]
    ImplicitInternalProviderNotFoundForProp(PropId),
    #[error(transparent)]
    InstalledPkg(#[from] InstalledPkgError),
    #[error("missing attribute prototype for attribute value")]
    MissingAttributePrototypeForAttributeValue,
    #[error("missing attribute prototype for explicit internal provider: {0}")]
    MissingAttributePrototypeForExplicitInternalProvider(InternalProviderId),
    #[error("missing attribute prototype for external provider: {0}")]
    MissingAttributePrototypeForExternalProvider(ExternalProviderId),
    #[error("no packages path configured")]
    MissingPkgsPath,
    #[error(transparent)]
    Pkg(#[from] PkgError),
    #[error("prop cache not found: {0}")]
    PropCacheNotFound(SchemaVariantId),
    #[error("prop not bound by id: {0}")]
    PropNotFound(PropId),
    #[error("Regex parsing error: {0}")]
    Regex(#[from] regex::Error),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("encountered serde json error for func ({0}): {1}")]
    SerdeJsonErrorForFunc(String, serde_json::Error),
    #[error(transparent)]
    SiPkg(#[from] SiPkgError),
    #[error(transparent)]
    Spec(#[from] SpecError),
    #[error("standard model error: {0}")]
    StandardModel(#[from] StandardModelError),
    #[error("error creating new transactions")]
    Transactions(#[from] TransactionsError),
}

pub type BuiltinsResult<T> = Result<T, BuiltinsError>;

/// This enum drives what builtin [`Schemas`](crate::Schema) to migrate for tests.
///
/// This enum _should not_ be used outside of tests!
#[remain::sorted]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SelectedTestBuiltinSchemas {
    /// Migrate everything (default behavior).
    All,
    /// Migrate nothing.
    None,
    /// Migrate _some_ [`Schema(s)`](crate::Schema) based on user input.
    Some(HashSet<String>),
    /// Migrate _only_ test-exclusive [`Schemas`](crate::Schema).
    Test,
}

/// Migrate all local "builtins" in a definitive order.
pub async fn migrate_local(
    ctx: &DalContext,
    _selected_test_builtin_schemas: Option<SelectedTestBuiltinSchemas>,
) -> BuiltinsResult<()> {
    info!("migrating intrinsic functions");
    func::migrate_intrinsics(ctx).await?;
    info!("intrinsics migrated");
    // info!("migrating builtin functions");
    // func::migrate(ctx).await?;

    // FIXME(nick): restore builtin migration functionality for all variants.
    info!("migrate minimal number of schemas for testing the new engine");

    schema::migrate_pkg(ctx, SI_DOCKER_IMAGE_PKG, None).await?;
    schema::migrate_pkg(ctx, SI_COREOS_PKG, None).await?;
    schema::migrate_pkg(ctx, SI_AWS_EC2_PKG, None).await?;
    schema::migrate_pkg(ctx, SI_AWS_PKG, None).await?;
    schema::migrate_test_exclusive_schema_starfield(ctx).await?;
    schema::migrate_test_exclusive_schema_fallout(ctx).await?;
    schema::migrate_test_exclusive_schema_bethesda_secret(ctx).await?;

    // match selected_test_builtin_schemas {
    //     Some(found_selected_test_builtin_schemas) => {
    //         schema::migrate_local_only_test_schemas(ctx, found_selected_test_builtin_schemas)
    //            .await?;
    //     }
    //     None => {
    //         schema::migrate_local_all_schemas(ctx).await?;
    //     }
    // }

    // info!("completed migrating functions, workflows and schemas");
    Ok(())
}
