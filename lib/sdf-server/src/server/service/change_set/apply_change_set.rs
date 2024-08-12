use axum::extract::{Host, OriginalUri};
use axum::Json;
use dal::change_set::ChangeSet;
use dal::{Func, Schema, SchemaVariant, Visibility};
use serde::{Deserialize, Serialize};

use super::ChangeSetResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyChangeSetRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ApplyChangeSetResponse {
    pub change_set: ChangeSet,
}

pub async fn apply_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    Json(request): Json<ApplyChangeSetRequest>,
) -> ChangeSetResult<Json<ApplyChangeSetResponse>> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    // Lock all unlocked variants
    for schema_id in Schema::list_ids(&ctx).await? {
        let schema = Schema::get_by_id(&ctx, schema_id).await?;
        let Some(variant) = SchemaVariant::get_unlocked_for_schema(&ctx, schema_id).await? else {
            continue;
        };

        let variant_id = variant.id();

        variant.lock(&ctx).await?;
        schema.set_default_schema_variant(&ctx, variant_id).await?;
    }
    // Lock all unlocked functions too
    for func in Func::list_for_default_and_editing(&ctx).await? {
        if !func.is_locked {
            func.lock(&ctx).await?;
        }
    }

    // We need to run a commit before apply so changes get saved
    ctx.commit().await?;

    let change_set = ChangeSet::apply_to_base_change_set(&mut ctx).await?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "apply_change_set",
        serde_json::json!({
            "merged_change_set": request.visibility.change_set_id,
        }),
    );

    // // If anything fails with uploading the workspace backup module, just log it. We shouldn't
    // // have the change set apply itself fail because of this.
    // tokio::task::spawn(
    //     super::upload_workspace_backup_module(ctx, raw_access_token)
    //         .instrument(info_span!("Workspace backup module upload")),
    // );

    ctx.commit().await?;

    Ok(Json(ApplyChangeSetResponse {
        change_set: change_set.to_owned(),
    }))
}
