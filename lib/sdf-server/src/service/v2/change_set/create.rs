use std::collections::HashSet;

use axum::{
    Json,
    extract::{
        Host,
        OriginalUri,
        Path,
    },
};
use dal::{
    ChangeSet,
    WorkspacePk,
    WsEvent,
};
use sdf_extract::{
    EddaClient,
    FriggStore,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_events::audit_log::AuditLogKind;
use si_frontend_mv_types::{
    index::MvIndex,
    reference::ReferenceKind,
};
use telemetry::prelude::*;

use super::{
    Error,
    Result,
};
use crate::{
    extract::{
        HandlerContext,
        PosthogClient,
    },
    service::v2::{
        AccessBuilder,
        index::{
            IndexResult,
            request_rebuild_and_watch,
        },
    },
    track,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    pub name: String,
}

#[allow(clippy::too_many_arguments)]
pub async fn create_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Host(host_name): Host,
    FriggStore(frigg): FriggStore,
    EddaClient(edda_client): EddaClient,
    Path(workspace_pk): Path<WorkspacePk>,
    Json(Request { name }): Json<Request>,
) -> Result<Json<si_frontend_types::ChangeSet>> {
    let ctx = builder.build_head(request_ctx).await?;

    let change_set_name = name.to_owned();

    let change_set = ChangeSet::fork_head(&ctx, change_set_name.clone()).await?;
    let change_set_id = change_set.id;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        &host_name,
        "create_change_set",
        serde_json::json!({
                    "change_set_name": change_set_name.clone(),
        }),
    );

    ctx.write_audit_log(AuditLogKind::CreateChangeSet, change_set_name.to_string())
        .await?;

    WsEvent::change_set_created(&ctx, change_set.id, change_set.workspace_snapshot_address)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    let change_set = change_set.into_frontend_type(&ctx).await?;
    ctx.commit_no_rebase().await?;

    let _index = match frigg.get_index(workspace_pk, change_set_id).await? {
        Some((index, _kv_revision)) => {
            let mv_index: MvIndex = serde_json::from_value(index.data.to_owned())
                .map_err(Error::DeserializingMvIndexData)?;

            // NOTE(nick,jacob): this may or may not be better suited for "edda". Let's trace this
            // to ensure that this stopgap solution does not bog the system down.
            let span = info_span!("sdf.index.get_change_set_index.existing_index_is_valid");
            let existing_index_is_valid = span.in_scope(|| {
                let mut revision_sensitive_reference_kinds_in_existing_index = HashSet::new();
                let mut invalid_reference_kinds = HashSet::new();

                for index_ref in mv_index.mv_list {
                    match ReferenceKind::try_from(index_ref.kind.as_str()) {
                        Ok(kind) => {
                            if kind.is_revision_sensitive() {
                                revision_sensitive_reference_kinds_in_existing_index.insert(kind);
                            }
                        }
                        Err(err) => {
                            trace!(reference_kind = %index_ref.kind, si.error.message = ?err, "could not convert string to ReferenceKind");

                            // Collect all of the invalid reference kinds rather than just bailing out early.
                            invalid_reference_kinds.insert(index_ref.kind);
                        }
                    }
                }

                // If we found at lease one invalid reference kind, the existing index is not
                // valid. Otherwise, let's check that all revision-sensitive kinds are the same as
                // those available today.
                if invalid_reference_kinds.is_empty() {
                    IndexResult::Ok(revision_sensitive_reference_kinds_in_existing_index == ReferenceKind::revision_sensitive())
                } else {
                    warn!(
                        ?invalid_reference_kinds,
                        "found invalid reference kind(s)"
                    );
                    IndexResult::Ok(false)
                }
            })?;

            if existing_index_is_valid {
                index
            } else {
                info!(
                    "Index out of date for change_set {}; attempting full build",
                    change_set_id,
                );
                request_rebuild_and_watch(&frigg, &edda_client, workspace_pk, change_set_id)
                    .await?;
                frigg
                    .get_index(workspace_pk, change_set_id)
                    .await?
                    .map(|i| i.0)
                    .ok_or(Error::IndexNotFoundAfterRebuild(
                        workspace_pk,
                        change_set_id,
                    ))?
            }
        }
        None => {
            info!(
                "Index not found for change_set {}; attempting full build",
                change_set_id,
            );
            request_rebuild_and_watch(&frigg, &edda_client, workspace_pk, change_set_id).await?;
            frigg
                .get_index(workspace_pk, change_set_id)
                .await?
                .map(|i| i.0)
                .ok_or(Error::IndexNotFoundAfterFreshBuild(
                    workspace_pk,
                    change_set_id,
                ))?
        }
    };

    Ok(Json(change_set))
}
