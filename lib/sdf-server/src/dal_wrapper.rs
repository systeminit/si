//! This module contains business logic wrapping functionality from the DAL that cannot (and should
//! not) be in the DAL itself.
//!
//! _Warning:_ this module should only be used as a last resort! Business logic should live in
//! other crates by default.

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use std::collections::HashMap;

use dal::{
    change_set::approval::ChangeSetApproval,
    workspace_snapshot::{graph::approval::ApprovalRequirement, EntityKindExt},
    DalContext, HistoryActor, WorkspacePk,
};
use permissions::{Permission, PermissionBuilder};
use si_events::ChangeSetApprovalStatus;
use si_id::{ChangeSetApprovalId, EntityId};
use thiserror::Error;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum DalWrapperError {
    #[error("change set approval error")]
    ChangeSetApproval(#[from] dal::change_set::approval::ChangeSetApprovalError),
    #[error("invalid user found")]
    InvalidUser,
    #[error("missing applicable approval id")]
    MissingApplicableApproval(ChangeSetApprovalId),
    #[error("permissions error: {0}")]
    Permissions(#[from] permissions::Error),
    #[error("spicedb lookup subjects error: {0}")]
    SpiceDBLookupSubjects(#[source] si_data_spicedb::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
}

type Result<T> = std::result::Result<T, DalWrapperError>;

/// Calculator for determining the status of approvals for a given change set.
#[derive(Debug)]
pub struct ChangeSetApprovalCalculator {
    frontend_latest_approvals: HashMap<ChangeSetApprovalId, si_frontend_types::ChangeSetApproval>,
    requirements_to_approvals_cache: HashMap<EntityId, Vec<ChangeSetApprovalId>>,
    requirements: Vec<ApprovalRequirement>,
}

impl ChangeSetApprovalCalculator {
    /// Creates a new calculator using the current change set and a [SpiceDbClient](`si_data_spicedb::Client`).
    pub async fn new(
        ctx: &DalContext,
        spicedb_client: &mut si_data_spicedb::Client,
    ) -> Result<Self> {
        // Gather everything we need upfront.
        let workspace_id = ctx.workspace_pk()?;
        let changes = ctx
            .workspace_snapshot()?
            .detect_changes_from_head(ctx)
            .await?;
        let requirements = ctx
            .workspace_snapshot()?
            .approval_requirements_for_changes(workspace_id, &changes)
            .await?;
        let latest_approvals = ChangeSetApproval::list_latest(ctx).await?;

        // Initialize what we will eventual use to construct ourself.
        let mut frontend_latest_approvals = HashMap::new();
        let mut requirements_to_approvals_cache: HashMap<EntityId, Vec<ChangeSetApprovalId>> =
            HashMap::new();

        // Go through each approval, determine its validity, and populate the cache.
        for approval in latest_approvals {
            let approved_requirement_ids =
                determine_approving_ids_inner(ctx, spicedb_client, workspace_id, &requirements)
                    .await?;
            for approved_requirement_id in &approved_requirement_ids {
                requirements_to_approvals_cache
                    .entry(*approved_requirement_id)
                    .and_modify(|a| a.push(approval.id()))
                    .or_insert_with(|| vec![approval.id()]);
            }

            // Based on the approving IDs, get the checksum.
            let checksum = ctx
                .workspace_snapshot()?
                .calculate_checksum(ctx, approved_requirement_ids)
                .await?
                .to_string();

            // Use the checksum to determine validity in the frontend approval type.
            frontend_latest_approvals.insert(
                approval.id(),
                si_frontend_types::ChangeSetApproval {
                    id: approval.id(),
                    user_id: approval.user_id(),
                    status: approval.status(),
                    is_valid: approval.checksum() == checksum.as_str(),
                },
            );
        }

        Ok(Self {
            frontend_latest_approvals,
            requirements_to_approvals_cache,
            requirements,
        })
    }

    /// Returns the latest approvals for frontend use as an array.
    pub fn frontend_latest_approvals(&self) -> Vec<si_frontend_types::ChangeSetApproval> {
        self.frontend_latest_approvals.values().cloned().collect()
    }

    /// Returns the requirments for frontend use, which includes whether or not they are satisfied.
    pub async fn frontend_requirements(
        &self,
        ctx: &DalContext,
        spicedb_client: &mut si_data_spicedb::Client,
    ) -> Result<Vec<si_frontend_types::ChangeSetApprovalRequirement>> {
        let mut frontend_requirements = Vec::with_capacity(self.requirements.len());
        let mut global_approving_groups_cache: HashMap<String, Vec<String>> = HashMap::new();

        // For each requirement, check if it has been satisfied, what approvals are applicable for
        // it, and what groups and users can approve it.
        for requirement in &self.requirements {
            // First, reset the satisfication and helper variables.
            let required_count = requirement.number;
            let mut satisfying_approval_count = 0;
            let mut is_satisfied = false;

            // If we have applicable approvals, then any that are valid and "approved" will
            // contribute to the count. If we hit the count, break and rejoice.
            let applicable_approval_ids = if let Some(applicable_approval_ids) = self
                .requirements_to_approvals_cache
                .get(&requirement.entity_id)
            {
                for applicable_approval_id in applicable_approval_ids {
                    let applicable_approval = self
                        .frontend_latest_approvals
                        .get(applicable_approval_id)
                        .ok_or(DalWrapperError::MissingApplicableApproval(
                            *applicable_approval_id,
                        ))?;
                    if applicable_approval.is_valid
                        && applicable_approval.status == ChangeSetApprovalStatus::Approved
                    {
                        satisfying_approval_count += 1;
                    }
                    if satisfying_approval_count >= required_count {
                        is_satisfied = true;
                        break;
                    }
                }
                applicable_approval_ids.to_owned()
            } else {
                Vec::new()
            };

            // We know what requirements have been satisfied, but we need to determine what groups
            // can fulfill those requirements as well as who belongs to them.
            let mut approving_groups = HashMap::new();
            for lookup_group in &requirement.lookup_groups {
                let lookup_group_key = format!(
                    "{}#{}#{}",
                    lookup_group.object_type, lookup_group.object_id, lookup_group.permission
                );

                // Check the global cache to reduce calls to SpiceDB.
                let member_ids: Vec<String> =
                    match global_approving_groups_cache.get(&lookup_group_key) {
                        Some(member_ids) => member_ids.to_owned(),
                        None => {
                            // TODO(nick): uh... do what Brit said in her original comment to this.
                            let member_ids = spicedb_client
                                .lookup_subjects(
                                    lookup_group.object_type.to_owned(),
                                    lookup_group.object_id.to_owned(),
                                    lookup_group.permission.to_owned(),
                                    "user".to_owned(),
                                )
                                .await
                                .map_err(DalWrapperError::SpiceDBLookupSubjects)?;
                            global_approving_groups_cache
                                .insert(lookup_group_key.to_owned(), member_ids.to_owned());
                            member_ids
                        }
                    };

                approving_groups.insert(lookup_group_key, member_ids);
            }

            // With both the satisfaction and approving groups information in hand, we can assemble
            // the frontend requirement.
            frontend_requirements.push(si_frontend_types::ChangeSetApprovalRequirement {
                entity_id: requirement.entity_id,
                entity_kind: ctx
                    .workspace_snapshot()?
                    .get_entity_kind_for_id(requirement.entity_id)
                    .await?,
                required_count,
                is_satisfied,
                applicable_approval_ids,
                approving_groups,
            })
        }

        Ok(frontend_requirements)
    }
}

/// Determines which IDs corresponding to nodes on the graph that the user can approve changes for.
pub async fn determine_approving_ids(
    ctx: &DalContext,
    spicedb_client: &mut si_data_spicedb::Client,
) -> Result<Vec<EntityId>> {
    let workspace_id = ctx.workspace_pk()?;
    let changes = ctx
        .workspace_snapshot()?
        .detect_changes_from_head(ctx)
        .await?;
    let requirements = ctx
        .workspace_snapshot()?
        .approval_requirements_for_changes(workspace_id, &changes)
        .await?;
    determine_approving_ids_inner(ctx, spicedb_client, workspace_id, &requirements).await
}

async fn determine_approving_ids_inner(
    ctx: &DalContext,
    spicedb_client: &mut si_data_spicedb::Client,
    workspace_id: WorkspacePk,
    requirements: &[ApprovalRequirement],
) -> Result<Vec<EntityId>> {
    let mut approving_ids = Vec::new();
    let mut cache = HashMap::new();

    for requirement in requirements {
        // For each requirement, we need to see if we have permission to fulfill it.
        for lookup_group in &requirement.lookup_groups {
            let has_permission_for_requirement =
                if let Some(has_permission) = cache.get(&lookup_group) {
                    *has_permission
                } else {
                    // TODO(nick,jacob): use the actual lookup group rather than this hardcoded check.
                    let has_permission = PermissionBuilder::new()
                        .workspace_object(workspace_id)
                        .permission(Permission::Approve)
                        .user_subject(match ctx.history_actor() {
                            HistoryActor::SystemInit => return Err(DalWrapperError::InvalidUser),
                            HistoryActor::User(user_id) => *user_id,
                        })
                        .has_permission(spicedb_client)
                        .await?;
                    cache.insert(lookup_group, has_permission);
                    has_permission
                };

            if has_permission_for_requirement {
                approving_ids.push(requirement.entity_id);

                // If we found that we have permission for the requirement, we do not need to continue
                // going through lookup groups for permissions.
                break;
            }
        }
    }

    Ok(approving_ids)
}
