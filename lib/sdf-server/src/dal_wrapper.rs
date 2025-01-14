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

use std::{collections::HashMap, str::FromStr};

use dal::{
    approval_requirement::{ApprovalRequirement, ApprovalRequirementApprover},
    change_set::approval::ChangeSetApproval,
    DalContext, HistoryActor, UserPk, WorkspacePk,
};
use permissions::{Permission, PermissionBuilder};
use si_events::{merkle_tree_hash::MerkleTreeHash, ChangeSetApprovalStatus};
use si_id::{ChangeSetApprovalId, EntityId};
use thiserror::Error;

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum DalWrapperError {
    #[error("approval requirement error: {0}")]
    ApprovalRequirement(#[from] dal::approval_requirement::ApprovalRequirementError),
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
    #[error("ulid decode error: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
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
        let (requirements, ids_with_hashes_for_deleted_nodes) =
            ApprovalRequirement::list(ctx, &changes).await?;
        let approving_requirement_ids_with_hashes = determine_approving_ids_with_hashes_inner(
            ctx,
            spicedb_client,
            workspace_id,
            &requirements,
            &ids_with_hashes_for_deleted_nodes,
        )
        .await?;
        let latest_approvals = ChangeSetApproval::list_latest(ctx).await?;

        // Initialize what we will eventual use to construct ourself.
        let mut frontend_latest_approvals = HashMap::new();
        let mut requirements_to_approvals_cache: HashMap<EntityId, Vec<ChangeSetApprovalId>> =
            HashMap::new();

        // Go through each approval, determine its validity, and populate the cache.
        for approval in latest_approvals {
            for (approving_requirement_id, _) in &approving_requirement_ids_with_hashes {
                requirements_to_approvals_cache
                    .entry(*approving_requirement_id)
                    .and_modify(|a| a.push(approval.id()))
                    .or_insert_with(|| vec![approval.id()]);
            }

            // Based on the approving IDs, get the checksum.
            let checksum = ctx
                .workspace_snapshot()?
                .calculate_checksum(ctx, approving_requirement_ids_with_hashes.to_owned())
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
        spicedb_client: &mut si_data_spicedb::Client,
    ) -> Result<Vec<si_frontend_types::ChangeSetApprovalRequirement>> {
        let mut frontend_requirements = Vec::with_capacity(self.requirements.len());
        let mut global_approving_groups_cache: HashMap<String, Vec<UserPk>> = HashMap::new();

        // For each requirement, check if it has been satisfied, what approvals are applicable for
        // it, and what groups and users can approve it.
        for requirement in &self.requirements {
            let rule = match requirement {
                ApprovalRequirement::Explicit(inner) => &inner.rule,
                ApprovalRequirement::Virtual(inner) => inner,
            };

            // First, reset the satisfication and helper variables.
            let required_count = rule.minimum;
            let mut satisfying_approval_count = 0;
            let mut is_satisfied = false;

            // If we have applicable approvals, then any that are valid and "approved" will
            // contribute to the count. If we hit the count, break and rejoice.
            let applicable_approval_ids = if let Some(applicable_approval_ids) =
                self.requirements_to_approvals_cache.get(&rule.entity_id)
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
            // and/or individuals can fulfill those requirements.
            let mut approver_groups = HashMap::new();
            let mut approver_individuals = Vec::new();
            for approver in &rule.approvers {
                let permission_lookup = match approver {
                    ApprovalRequirementApprover::User(user_id) => {
                        approver_individuals.push(*user_id);
                        continue;
                    }
                    ApprovalRequirementApprover::PermissionLookup(permission_lookup) => {
                        permission_lookup
                    }
                };

                let permisssion_lookup_key = format!(
                    "{}#{}#{}",
                    permission_lookup.object_type,
                    permission_lookup.object_id,
                    permission_lookup.permission
                );

                // Check the global cache to reduce calls to SpiceDB.
                let member_ids: Vec<UserPk> =
                    match global_approving_groups_cache.get(&permisssion_lookup_key) {
                        Some(member_ids) => member_ids.to_owned(),
                        None => {
                            // TODO(nick): uh... do what Brit said in her original comment to this.
                            let raw_member_ids = spicedb_client
                                .lookup_subjects(
                                    permission_lookup.object_type.to_owned(),
                                    permission_lookup.object_id.to_owned(),
                                    permission_lookup.permission.to_owned(),
                                    "user".to_owned(),
                                )
                                .await
                                .map_err(DalWrapperError::SpiceDBLookupSubjects)?;
                            let mut member_ids = Vec::with_capacity(raw_member_ids.len());
                            for raw_member_id in raw_member_ids {
                                member_ids.push(UserPk::from_str(raw_member_id.as_str())?);
                            }
                            global_approving_groups_cache
                                .insert(permisssion_lookup_key.to_owned(), member_ids.to_owned());
                            member_ids
                        }
                    };

                approver_groups.insert(permisssion_lookup_key, member_ids);
            }

            // With both the satisfaction and approvers information in hand, we can assemble the
            // frontend requirement.
            frontend_requirements.push(si_frontend_types::ChangeSetApprovalRequirement {
                entity_id: rule.entity_id,
                entity_kind: rule.entity_kind,
                required_count,
                is_satisfied,
                applicable_approval_ids,
                approver_groups,
                approver_individuals,
            })
        }

        Ok(frontend_requirements)
    }
}

/// Determines which IDs (with hashes) correspond to nodes on the graph that the user can approve
/// changes for.
pub async fn determine_approving_ids_with_hashes(
    ctx: &DalContext,
    spicedb_client: &mut si_data_spicedb::Client,
) -> Result<Vec<(EntityId, MerkleTreeHash)>> {
    let workspace_id = ctx.workspace_pk()?;
    let changes = ctx
        .workspace_snapshot()?
        .detect_changes_from_head(ctx)
        .await?;
    let (requirements, ids_with_hashes_for_deleted_nodes) =
        ApprovalRequirement::list(ctx, &changes).await?;
    determine_approving_ids_with_hashes_inner(
        ctx,
        spicedb_client,
        workspace_id,
        &requirements,
        &ids_with_hashes_for_deleted_nodes,
    )
    .await
}

async fn determine_approving_ids_with_hashes_inner(
    ctx: &DalContext,
    spicedb_client: &mut si_data_spicedb::Client,
    workspace_id: WorkspacePk,
    requirements: &[ApprovalRequirement],
    ids_with_hashes_for_deleted_nodes: &HashMap<EntityId, MerkleTreeHash>,
) -> Result<Vec<(EntityId, MerkleTreeHash)>> {
    let user_id = match ctx.history_actor() {
        HistoryActor::SystemInit => return Err(DalWrapperError::InvalidUser),
        HistoryActor::User(user_id) => *user_id,
    };

    let mut approving_ids_with_hashes = Vec::new();
    let mut cache = HashMap::new();

    for requirement in requirements {
        let rule = match requirement {
            ApprovalRequirement::Explicit(inner) => &inner.rule,
            ApprovalRequirement::Virtual(inner) => inner,
        };

        // For each requirement, we need to see if we have permission to fulfill it.
        for approver in &rule.approvers {
            let has_permission_for_requirement = if let Some(has_permission) = cache.get(&approver)
            {
                *has_permission
            } else {
                // If the permission is not in our cache, we need to find it and cache it for later.
                match approver {
                    ApprovalRequirementApprover::User(approver_user_id) => {
                        let has_permission = *approver_user_id == user_id;
                        cache.insert(approver, has_permission);
                        has_permission
                    }
                    ApprovalRequirementApprover::PermissionLookup(_) => {
                        // TODO(nick): use the actual lookup group rather than this hardcoded check.
                        let has_permission = PermissionBuilder::new()
                            .workspace_object(workspace_id)
                            .permission(Permission::Approve)
                            .user_subject(match ctx.history_actor() {
                                HistoryActor::SystemInit => {
                                    return Err(DalWrapperError::InvalidUser)
                                }
                                HistoryActor::User(user_id) => *user_id,
                            })
                            .has_permission(spicedb_client)
                            .await?;
                        cache.insert(approver, has_permission);
                        has_permission
                    }
                }
            };

            if has_permission_for_requirement {
                // NOTE(nick): okay, this is where things get weird. What if a virtual requirement
                // was generated for a deleted node? Well then ya can't get the merkle tree hash
                // ya know? Therefore, we must first consult the map pertaining to deleted nodes
                // to see if our hash is in there first. This algorithm assumes that the map is
                // assembled correctly, so it's best to be sure it's correct or you will perish.
                let merkle_tree_hash = if let Some(merkle_tree_hash_for_deleted_node) =
                    ids_with_hashes_for_deleted_nodes.get(&rule.entity_id)
                {
                    *merkle_tree_hash_for_deleted_node
                } else {
                    ctx.workspace_snapshot()?
                        .get_node_weight_by_id(rule.entity_id)
                        .await?
                        .merkle_tree_hash()
                };

                // Now that we have the hash, we can push!
                approving_ids_with_hashes.push((rule.entity_id, merkle_tree_hash));

                // If we found that we have permission for the requirement, we do not need to continue
                // going through lookup groups for permissions.
                break;
            }
        }
    }

    Ok(approving_ids_with_hashes)
}
