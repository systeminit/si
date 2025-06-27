use axum::{
    Json,
    extract::Path,
};
use dal::{
    ChangeSet,
    ChangeSetId,
    WorkspacePk,
};
use sdf_extract::{
    PosthogEventTracker,
    change_set::ChangeSetDalContext,
};
use serde::Deserialize;

use super::Result;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RenameChangeSetRequest {
    new_name: String,
}

pub async fn rename(
    ChangeSetDalContext(ref mut ctx): ChangeSetDalContext,
    tracker: PosthogEventTracker,
    Path((_workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
    Json(request): Json<RenameChangeSetRequest>,
) -> Result<()> {
    ChangeSet::get_by_id(ctx, ctx.visibility().change_set_id).await?;

    ChangeSet::rename_change_set(ctx, change_set_id, &request.new_name).await?;

    tracker.track(
        ctx,
        "rename_change_set",
        serde_json::json!({
            "change_set": change_set_id,
            "new_name": request.new_name,
        }),
    );

    ctx.commit().await?;

    Ok(())
}
