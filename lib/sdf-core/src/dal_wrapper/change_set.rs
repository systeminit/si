//! This module contains DAL-wrapper logic around change set approvals.

use std::{
    collections::HashMap,
    str::FromStr,
};

use dal::{
    ChangeSet,
    DalContext,
    UserPk,
    Workspace,
    WorkspacePk,
    approval_requirement::{
        ApprovalRequirement,
        ApprovalRequirementApprover,
    },
    change_set::{
        approval::ChangeSetApproval,
        calculate_checksum,
    },
};
use permissions::{
    Permission,
    PermissionBuilder,
};
use si_db::{
    HistoryActor,
    User,
};
use si_events::{
    ChangeSetApprovalStatus,
    merkle_tree_hash::MerkleTreeHash,
};
use si_id::{
    ChangeSetApprovalId,
    EntityId,
};

use super::DalWrapperError;

type Result<T> = std::result::Result<T, DalWrapperError>;

/// Returns all unsatisfied approval requirements (minimal information) for the current change set.
pub async fn approval_requirements_are_satisfied_or_error(
    ctx: &DalContext,
    spicedb_client: &mut si_data_spicedb::Client,
) -> Result<()> {
    let (_, requirements) = super::change_set::status(ctx, spicedb_client).await?;

    let mut unsatisfied_requirements = Vec::new();
    for requirement in requirements {
        if !requirement.is_satisfied {
            unsatisfied_requirements.push((requirement.entity_id, requirement.entity_kind));
        }
    }

    if !unsatisfied_requirements.is_empty() {
        return Err(DalWrapperError::ApplyWithUnsatisfiedRequirements(
            unsatisfied_requirements,
        ));
    }
    Ok(())
}

/// Applies the current change set to the base change set, but with protections in place, such as
/// ensuring that the workspace is opt-ed into approvals, requirements are met and that we have
/// committed preparations.
pub async fn protected_apply_to_base_change_set(
    ctx: &mut DalContext,
    spicedb_client: &mut si_data_spicedb::Client,
) -> Result<()> {
    let workspace_pk = ctx.workspace_pk()?;
    let workspace = Workspace::get_by_pk(ctx, workspace_pk).await?;

    // Let's check if the workspace is opt-ed in for approvals
    if workspace.approvals_enabled() {
        let solo_user_in_workspace = match ctx.history_actor() {
            HistoryActor::SystemInit => return Err(DalWrapperError::InvalidUser),
            HistoryActor::User(user_pk) => {
                let workspace_pk = ctx.workspace_pk()?;
                let user_pks =
                    User::list_member_pks_for_workspace(ctx, workspace_pk.to_string()).await?;

                user_pks.len() == 1
                    && user_pks
                        .first()
                        .ok_or(DalWrapperError::NoUsersInWorkspace(workspace_pk))?
                        == user_pk
            }
        };

        // First, check if all requirements have been satisfied. We do not need to check this
        // if we are allowed to skip the approval flow
        if !solo_user_in_workspace {
            approval_requirements_are_satisfied_or_error(ctx, spicedb_client).await?;
        }
    }

    // With the requirement check satisfied, we can finally perform the apply.
    // We skip the "status" check since it is performed above
    ChangeSet::begin_apply_without_status_check(ctx).await?;

    Ok(())
}

/// Gets the current change set approval status, which is a combination of approvals and
/// requirements with relevant metadata.
pub async fn status(
    ctx: &DalContext,
    spicedb_client: &mut si_data_spicedb::Client,
) -> Result<(
    Vec<si_frontend_types::ChangeSetApproval>,
    Vec<si_frontend_types::ChangeSetApprovalRequirement>,
)> {
    let changes = ctx.detect_changes_from_head().await?;
    let (requirements, ids_with_hashes_for_deleted_nodes) =
        ApprovalRequirement::list(ctx, &changes).await?;

    let (frontend_latest_approvals_by_id, requirements_to_approvals_cache) =
        inner_determine_latest_approvals_and_populate_caches(
            ctx,
            spicedb_client,
            &requirements,
            &ids_with_hashes_for_deleted_nodes,
        )
        .await?;

    let frontend_requirements = inner_determine_frontend_requirements(
        spicedb_client,
        &requirements,
        &frontend_latest_approvals_by_id,
        &requirements_to_approvals_cache,
    )
    .await?;
    let frontend_latest_approvals = frontend_latest_approvals_by_id.values().cloned().collect();

    Ok((frontend_latest_approvals, frontend_requirements))
}

/// Provides the approving IDs (with hashes) for new change set approval creation.
pub async fn new_approval_approving_ids_with_hashes(
    ctx: &DalContext,
    spicedb_client: &mut si_data_spicedb::Client,
) -> Result<Vec<(EntityId, MerkleTreeHash)>> {
    let workspace_id = ctx.workspace_pk()?;
    let changes = ctx.detect_changes_from_head().await?;
    let (requirements, ids_with_hashes_for_deleted_nodes) =
        ApprovalRequirement::list(ctx, &changes).await?;
    inner_determine_approving_ids_with_hashes(
        ctx,
        spicedb_client,
        None,
        workspace_id,
        &requirements,
        &ids_with_hashes_for_deleted_nodes,
    )
    .await
}

async fn inner_determine_latest_approvals_and_populate_caches(
    ctx: &DalContext,
    spicedb_client: &mut si_data_spicedb::Client,
    requirements: &[ApprovalRequirement],
    ids_with_hashes_for_deleted_nodes: &HashMap<EntityId, MerkleTreeHash>,
) -> Result<(
    HashMap<ChangeSetApprovalId, si_frontend_types::ChangeSetApproval>,
    HashMap<EntityId, Vec<ChangeSetApprovalId>>,
)> {
    let workspace_id = ctx.workspace_pk()?;

    // Gather everything we need upfront.
    let latest_approvals = ChangeSetApproval::list_latest(ctx).await?;

    // Initialize what we will eventual use to construct ourself.
    let mut frontend_latest_approvals_by_id = HashMap::new();
    let mut requirements_to_approvals_cache: HashMap<EntityId, Vec<ChangeSetApprovalId>> =
        HashMap::new();

    // Go through each approval, determine its validity, and populate the cache.
    for approval in latest_approvals {
        let approving_requirement_ids_with_hashes = inner_determine_approving_ids_with_hashes(
            ctx,
            spicedb_client,
            Some(approval.user_id()),
            workspace_id,
            requirements,
            ids_with_hashes_for_deleted_nodes,
        )
        .await?;

        for (approving_requirement_id, _) in &approving_requirement_ids_with_hashes {
            requirements_to_approvals_cache
                .entry(*approving_requirement_id)
                .and_modify(|a| a.push(approval.id()))
                .or_insert_with(|| vec![approval.id()]);
        }

        // Based on the approving IDs, get the checksum.
        let checksum = calculate_checksum(ctx, approving_requirement_ids_with_hashes.to_owned())
            .await?
            .to_string();

        // Use the checksum to determine validity in the frontend approval type.
        frontend_latest_approvals_by_id.insert(
            approval.id(),
            si_frontend_types::ChangeSetApproval {
                id: approval.id(),
                user_id: approval.user_id(),
                status: approval.status(),
                is_valid: approval.checksum() == checksum.as_str(),
            },
        );
    }

    Ok((
        frontend_latest_approvals_by_id,
        requirements_to_approvals_cache,
    ))
}

async fn inner_determine_frontend_requirements(
    spicedb_client: &mut si_data_spicedb::Client,
    requirements: &[ApprovalRequirement],
    frontend_latest_approvals_by_id: &HashMap<
        ChangeSetApprovalId,
        si_frontend_types::ChangeSetApproval,
    >,
    requirements_to_approvals_cache: &HashMap<EntityId, Vec<ChangeSetApprovalId>>,
) -> Result<Vec<si_frontend_types::ChangeSetApprovalRequirement>> {
    let mut frontend_requirements = Vec::with_capacity(requirements.len());
    let mut global_approving_groups_cache: HashMap<String, Vec<UserPk>> = HashMap::new();

    // For each requirement, check if it has been satisfied, what approvals are applicable for
    // it, and what groups and users can approve it.
    for requirement in requirements {
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
            requirements_to_approvals_cache.get(&rule.entity_id)
        {
            for applicable_approval_id in applicable_approval_ids {
                let applicable_approval = frontend_latest_approvals_by_id
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

async fn inner_determine_approving_ids_with_hashes(
    ctx: &DalContext,
    spicedb_client: &mut si_data_spicedb::Client,
    user_id: Option<UserPk>,
    workspace_id: WorkspacePk,
    requirements: &[ApprovalRequirement],
    ids_with_hashes_for_deleted_nodes: &HashMap<EntityId, MerkleTreeHash>,
) -> Result<Vec<(EntityId, MerkleTreeHash)>> {
    let user_id = match user_id {
        Some(user_id) => user_id,
        None => match ctx.history_actor() {
            HistoryActor::SystemInit => return Err(DalWrapperError::InvalidUser),
            HistoryActor::User(user_id) => *user_id,
        },
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
                    ApprovalRequirementApprover::PermissionLookup(permission_lookup) => {
                        match (
                            permission_lookup.object_type.as_str(),
                            permission_lookup.object_id.as_str(),
                            permission_lookup.permission.as_str(),
                        ) {
                            ("workspace", object_id, "approve") => {
                                let object_workspace_id = WorkspacePk::from_str(object_id)?;
                                if object_workspace_id != workspace_id {
                                    return Err(
                                        DalWrapperError::InvalidWorkspaceForPermissionLookup(
                                            object_workspace_id,
                                            workspace_id,
                                        ),
                                    );
                                }
                                let has_permission = PermissionBuilder::new()
                                    .workspace_object(workspace_id)
                                    .permission(Permission::Approve)
                                    .user_subject(user_id)
                                    .has_permission(spicedb_client)
                                    .await?;
                                cache.insert(approver, has_permission);
                                has_permission
                            }
                            (object_type, object_id, permission) => {
                                return Err(DalWrapperError::UnsupportedPermissionLookup(
                                    object_type.into(),
                                    object_id.into(),
                                    permission.into(),
                                ));
                            }
                        }
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
                        .get_node_weight(rule.entity_id)
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
