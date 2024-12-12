use anyhow::Result;
use axum::{
    extract::{Path, State},
    Json,
};
use dal::{ChangeSetId, WorkspacePk};

use crate::{
    dal_wrapper::{self},
    extract::HandlerContext,
    service::v2::AccessBuilder,
    AppState,
};

use super::ChangeSetAPIError;

pub async fn approval_status(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    Path((_workspace_id, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    State(mut state): State<AppState>,
) -> Result<Json<si_frontend_types::ChangeSetApprovals>> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;

    let spicedb_client = state
        .spicedb_client()
        .ok_or(ChangeSetAPIError::SpiceDBClientNotFound)?;

    let (latest_approvals, requirements) =
        dal_wrapper::change_set::status(&ctx, spicedb_client).await?;

    Ok(Json(si_frontend_types::ChangeSetApprovals {
        latest_approvals,
        requirements,
    }))
}
