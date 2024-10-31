use axum::{
    extract::{Host, OriginalUri, Path, State},
    Json,
};
use dal::{ChangeSet, ChangeSetId, WorkspacePk};
use permissions::{ObjectType, Relation, RelationBuilder};

use super::{AppState, Error, Result};
use crate::{
    extract::{AccessBuilder, HandlerContext, PosthogClient},
    track,
};

pub async fn list_actionable(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    State(mut state): State<AppState>,
    Path(workspace_pk): Path<WorkspacePk>,
) -> Result<Json<si_frontend_types::WorkspaceMetadata>> {
    let ctx = builder.build_head(request_ctx).await?;

    // List all actionable change sets and assemble them into individual views.
    let open_change_sets = ChangeSet::list_active(&ctx).await?;
    let mut views = Vec::with_capacity(open_change_sets.len());
    for change_set in open_change_sets {
        views.push(change_set.into_frontend_type(&ctx).await?);
    }
    let client = state.spicedb_client().ok_or(Error::SpiceDBNotFound)?;
    let existing_approvers = RelationBuilder::new()
        .object(ObjectType::Workspace, workspace_pk)
        .relation(Relation::Approver)
        .read(client)
        .await?;

    let existing_approver_ids: Vec<String> = existing_approvers
        .into_iter()
        .map(|w| w.subject().id().to_string())
        .collect();

    // Ensure that we find exactly one change set view that matches the open change sets found.
    let head_change_set_id = ctx.get_workspace_default_change_set_id().await?;
    let maybe_head_change_set_id: Vec<ChangeSetId> = views
        .iter()
        .filter_map(|v| {
            if v.id == head_change_set_id.into() {
                Some(head_change_set_id)
            } else {
                None
            }
        })
        .collect();
    if maybe_head_change_set_id.len() != 1 {
        return Err(
            Error::UnexpectedNumberOfOpenChangeSetsMatchingDefaultChangeSet(
                maybe_head_change_set_id,
            ),
        );
    }
    let workspace = &ctx.get_workspace().await?;
    let workspace_view = si_frontend_types::WorkspaceMetadata {
        name: workspace.name().to_string(),
        id: workspace.pk().to_string(),
        default_change_set_id: head_change_set_id.into(),
        change_sets: views,
        approvers: existing_approver_ids,
    };
    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "list",
        serde_json::json!({
            "workspace_id": workspace_pk,
        }),
    );

    Ok(Json(workspace_view))
}
