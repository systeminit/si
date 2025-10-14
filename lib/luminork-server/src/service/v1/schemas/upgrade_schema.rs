use std::collections::HashMap;

use axum::{
    Json,
    extract::Path,
};
use dal::{
    Prop,
    Schema,
    SchemaVariant,
    cached_module::CachedModule,
    pkg::{
        ImportOptions,
        import_pkg_from_pkg,
    },
    schema::variant::root_prop::RootPropChild,
    workspace_snapshot::traits::prop::PropExt,
};
use sdf_extract::FriggStore;
use serde::{
    Deserialize,
    Serialize,
};
use serde_json::json;
use si_frontend_mv_types::{
    cached_default_variant::CachedDefaultVariant,
    prop_schema::PropSchemaV1 as CachedPropSchemaV1,
    reference::ReferenceKind,
};
use utoipa::ToSchema;

use super::{
    SchemaError,
    SchemaResult,
    SchemaV1RequestPath,
    check_schema_upgrade_available,
};
use crate::extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeSchemaRequest {
    /// If true, only check for breaking changes without performing the upgrade
    #[serde(default)]
    #[schema(value_type = bool, example = false)]
    pub dry_run: bool,
}

/// Details about a breaking change detected during schema upgrade comparison.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct BreakingChange {
    /// The path to the prop that changed (e.g., "/domain/ImageId")
    pub path: String,
    /// The type of breaking change
    pub change_type: BreakingChangeType,
    /// The old prop type (for type changes)
    pub old_type: Option<String>,
    /// The new prop type (for type changes)
    pub new_type: Option<String>,
}

/// Type of breaking change detected.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum BreakingChangeType {
    /// A prop was removed
    PropRemoved,
    /// A prop's type changed
    TypeChanged,
}

#[derive(Deserialize, Serialize, Debug, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UpgradeSchemaResponse {
    pub success: bool,
    /// List of breaking changes detected (empty if no breaking changes, null if couldn't be determined)
    pub breaking_changes: Option<Vec<BreakingChange>>,
    /// Whether this was a dry run (no actual upgrade performed)
    pub dry_run: bool,
}

#[utoipa::path(
      post,
      path = "/v1/w/{workspace_id}/change-sets/{change_set_id}/schemas/{schema_id}/upgrade",
      params(
          ("workspace_id" = String, Path, description = "Workspace identifier"),
          ("change_set_id" = String, Path, description = "Change Set identifier"),
          ("schema_id" = String, Path, description = "Schema identifier"),
      ),
      request_body = UpgradeSchemaRequest,
      tag = "schemas",
      summary = "Upgrade a schema to the latest available version",
      responses(
          (status = 200, description = "Schema upgraded successfully or dry run completed", body = UpgradeSchemaResponse),
          (status = 401, description = "Unauthorized - Invalid or missing token"),
          (status = 404, description = "Schema not found"),
          (status = 404, description = "No cached module found for schema"),
          (status = 412, description = "Precondition failed - schema has unlocked variants or not installed"),
          (status = 500, description = "Internal server error", body = crate::service::v1::common::ApiError)
      )
  )]
pub async fn upgrade_schema(
    ChangeSetDalContext(ref ctx): ChangeSetDalContext,
    FriggStore(ref frigg): FriggStore,
    tracker: PosthogEventTracker,
    Path(SchemaV1RequestPath { schema_id }): Path<SchemaV1RequestPath>,
    payload: Result<Json<UpgradeSchemaRequest>, axum::extract::rejection::JsonRejection>,
) -> SchemaResult<Json<UpgradeSchemaResponse>> {
    let Json(payload) = payload?;

    let schema_exists_locally = Schema::exists_locally(ctx, schema_id).await?;
    if !schema_exists_locally {
        return Err(SchemaError::SchemaNotFound(schema_id));
    }

    let breaking_changes = check_schema_upgrade_breaking_changes(ctx, frigg, schema_id).await?;

    // If dry run, return early with breaking change information
    if payload.dry_run {
        return Ok(Json(UpgradeSchemaResponse {
            success: false,
            breaking_changes,
            dry_run: true,
        }));
    }

    if SchemaVariant::get_unlocked_for_schema(ctx, schema_id)
        .await?
        .is_some()
    {
        return Err(SchemaError::UnlockedVariantFoundForSchema(schema_id));
    }

    let Some(mut cached_module) = CachedModule::find_latest_for_schema_id(ctx, schema_id).await?
    else {
        return Err(SchemaError::UpgradableModuleNotFound(schema_id));
    };

    let si_pkg = cached_module.si_pkg(ctx).await?;
    let metadata = si_pkg.metadata()?;

    let (_, schema_variant_ids, _) = import_pkg_from_pkg(
        ctx,
        &si_pkg,
        Some(ImportOptions {
            schema_id: Some(schema_id.into()),
            ..Default::default()
        }),
    )
    .await?;

    if schema_variant_ids.is_empty() {
        return Err(SchemaError::UpgradeFailed(schema_id));
    }

    tracker.track(
        ctx,
        "api_upgrade_schema",
        json!({
            "schema_id": schema_id,
            "pkg_name": metadata.name(),
            "has_breaking_changes": breaking_changes.as_ref().map(|c| !c.is_empty()).unwrap_or(false),
            "breaking_change_count": breaking_changes.as_ref().map(|c| c.len()),
        }),
    );

    ctx.commit().await?;

    Ok(Json(UpgradeSchemaResponse {
        success: true,
        breaking_changes,
        dry_run: false,
    }))
}

/// Check if upgrading a schema would cause breaking changes by comparing domain props.
///
/// Returns a list of breaking changes found, or None if comparison not possible.
async fn check_schema_upgrade_breaking_changes(
    ctx: &dal::DalContext,
    frigg: &frigg::FriggStore,
    schema_id: dal::SchemaId,
) -> SchemaResult<Option<Vec<BreakingChange>>> {
    // Check if upgrade is actually available first
    let upgrade_available = check_schema_upgrade_available(ctx, schema_id).await?;
    if !matches!(upgrade_available, Some(true)) {
        return Ok(None);
    }

    // Get default variant ID for installed schema
    let default_variant_id = dal::Schema::default_variant_id(ctx, schema_id).await?;

    // Get current domain props from installed variant
    let domain_prop =
        Prop::find_prop_by_path(ctx, default_variant_id, &RootPropChild::Domain.prop_path())
            .await?;

    let current_domain_props = ctx
        .workspace_snapshot()?
        .build_prop_schema_tree(ctx, domain_prop.id)
        .await?;

    // Get upgraded domain props from Frigg
    let upgraded_domain_props = match frigg
        .get_current_deployment_object(ReferenceKind::CachedDefaultVariant, &schema_id.to_string())
        .await?
    {
        Some(obj) => {
            let cached_variant: CachedDefaultVariant = serde_json::from_value(obj.data)?;
            cached_variant.domain_props
        }
        None => return Ok(None),
    };

    let Some(upgraded_props) = upgraded_domain_props else {
        return Ok(None);
    };

    // Compare the two prop trees and return detailed changes
    Ok(Some(find_breaking_changes(
        &current_domain_props,
        &upgraded_props,
        "",
    )))
}

/// Recursively find all breaking changes between two PropSchemaV1 trees.
fn find_breaking_changes(
    current: &CachedPropSchemaV1,
    upgraded: &CachedPropSchemaV1,
    path_prefix: &str,
) -> Vec<BreakingChange> {
    let mut changes = Vec::new();
    let current_path = if path_prefix.is_empty() {
        format!("/{}", current.name)
    } else {
        format!("{}/{}", path_prefix, current.name)
    };

    // Type changed - this is a breaking change
    if current.prop_type != upgraded.prop_type {
        return vec![BreakingChange {
            path: current_path,
            change_type: BreakingChangeType::TypeChanged,
            old_type: Some(current.prop_type.clone()),
            new_type: Some(upgraded.prop_type.clone()),
        }];
    }

    // No children means no further breaking changes at this level
    let (Some(current_children), Some(upgraded_children)) = (&current.children, &upgraded.children)
    else {
        return changes;
    };

    // Build lookup map for upgraded children
    let upgraded_child_map: HashMap<&str, &CachedPropSchemaV1> = upgraded_children
        .iter()
        .map(|child| (child.name.as_str(), child))
        .collect();

    // Check each current child for removal or nested changes
    for current_child in current_children {
        match upgraded_child_map.get(current_child.name.as_str()) {
            Some(upgraded_child) => {
                changes.extend(find_breaking_changes(
                    current_child,
                    upgraded_child,
                    &current_path,
                ));
            }
            None => {
                changes.push(BreakingChange {
                    path: format!("{}/{}", current_path, current_child.name),
                    change_type: BreakingChangeType::PropRemoved,
                    old_type: Some(current_child.prop_type.clone()),
                    new_type: None,
                });
            }
        }
    }

    changes
}
