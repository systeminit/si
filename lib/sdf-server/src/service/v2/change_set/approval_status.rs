use std::{collections::HashSet, str::FromStr};

use axum::{
    extract::{Path, State},
    Json,
};
use dal::{
    change_set::approval::ChangeSetApproval, workspace_snapshot::graph::detector::ChangeKind,
    ChangeSetId, NodeWeightDiscriminants, UserPk, WorkspacePk,
};
use permissions::{ObjectType, Relation, RelationBuilder};
use si_events::{ChangeSetApprovalKind, ChangeSetApprovalStatus};
use si_frontend_types::ChangeSetRequiredApproval;

use crate::{
    extract::{AccessBuilder, HandlerContext},
    AppState,
};

use super::{ChangeSetAPIError, Result};

pub async fn approval_status(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((workspace_id, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    State(mut state): State<AppState>,
) -> Result<Json<si_frontend_types::ChangeSetApprovals>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let (current_checksum, changes) = ChangeSetApproval::calculate_checksum(&ctx).await?;
    let current_checksum = current_checksum.to_string();

    let approvals = ChangeSetApproval::list(&ctx).await?;
    let mut current = Vec::with_capacity(approvals.len());
    for approval in approvals {
        // TODO(nick): we need to decide how we are handling the table. What if someone approves, disapproves,
        // re-approves, does it all multiple times, and then the older ones are no longer valid? I feel as though
        // either the PG table should have a uniqueness constraint on "user_id" and "change_set_id" or that we add a
        // new row for every approval and only look at the latest for each "user_id" and "change_set_id".
        current.push(si_frontend_types::ChangeSetApproval {
            user_id: approval.user_id(),
            status: approval.status(),
            is_valid: approval.checksum() == current_checksum.as_str(),
        })
    }

    // For now, the only eligible approvers are the workspace owners.
    let eligible_approver_user_ids = {
        let client = state
            .spicedb_client()
            .ok_or(ChangeSetAPIError::SpiceDBClientNotFound)?;
        let eligible_approvers = RelationBuilder::new()
            .object(ObjectType::Workspace, workspace_id.clone())
            .relation(Relation::Owner)
            .read(client)
            .await?;
        let mut eligible_approver_user_ids = HashSet::with_capacity(eligible_approvers.len());
        for existing_approver in eligible_approvers {
            eligible_approver_user_ids.insert(UserPk::from_str(existing_approver.subject().id())?);
        }
        eligible_approver_user_ids
    };

    let mut required = Vec::new();
    for change in changes {
        // TODO(nick): add requirements for more than just schema variants.
        if let ChangeKind::Node(NodeWeightDiscriminants::SchemaVariant) = change.kind {
            // TODO(nick): replace the hard-coded number of approvers required.
            let number = 1;

            let mut is_satisfied = false;
            let mut valid_approvals_count = 0;
            for current in &current {
                // TODO(nick): see the approvals table comment above. This check assumes that the very first approval
                // that meets this condition is valid. It does not account for a scenario where a newer "disapproval"
                // comes into play. In addition, it also does not account for processing the same user multiple times.
                if eligible_approver_user_ids.contains(&current.user_id)
                    && current.is_valid
                    && current.status == ChangeSetApprovalStatus::Approved
                {
                    valid_approvals_count += 1;
                }
                if valid_approvals_count >= number {
                    is_satisfied = true;
                    break;
                }
            }

            required.push(ChangeSetRequiredApproval {
                kind: ChangeSetApprovalKind::SchemaVariant,
                id: change.id,
                number,
                is_satisfied,
                eligible_approver_user_ids: eligible_approver_user_ids
                    .clone()
                    .into_iter()
                    .collect(),
            })
        }
    }

    Ok(Json(si_frontend_types::ChangeSetApprovals {
        required,
        current,
    }))
}
