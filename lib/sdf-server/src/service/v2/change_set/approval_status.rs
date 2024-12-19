use axum::{extract::Path, Json};
use dal::{change_set::approval::ChangeSetApproval, ChangeSetId, WorkspacePk};

use crate::extract::{AccessBuilder, HandlerContext};

use super::Result;

pub async fn approval_status(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> Result<Json<si_frontend_types::ChangeSetApprovals>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let current_checksum = ChangeSetApproval::calculate_checksum(&ctx)
        .await?
        .to_string();

    let approvals = ChangeSetApproval::list(&ctx).await?;
    let mut current = Vec::with_capacity(approvals.len());
    for approval in approvals {
        current.push(si_frontend_types::ChangeSetApproval {
            user_id: approval.user_id(),
            status: approval.status(),
            is_valid: approval.checksum() == current_checksum.as_str(),
        })
    }

    Ok(Json(si_frontend_types::ChangeSetApprovals {
        // FIXME(nick): get requirements.
        required: Vec::new(),
        current,
    }))
}
