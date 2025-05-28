use std::collections::HashSet;

use axum::{
    Json,
    extract::Path,
    http::StatusCode,
    response::IntoResponse,
};
use dal::{
    ChangeSet,
    ChangeSetId,
    WorkspacePk,
};
use si_frontend_mv_types::{
    index::MvIndex,
    reference::ReferenceKind,
};
use telemetry::prelude::*;

use super::{
    AccessBuilder,
    FrontEndObjectMeta,
    IndexError,
    IndexResult,
    request_rebuild_and_watch,
};
use crate::extract::{
    EddaClient,
    FriggStore,
    HandlerContext,
};

pub async fn get_change_set_index(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    FriggStore(frigg): FriggStore,
    EddaClient(edda_client): EddaClient,
    Path((workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> IndexResult<impl IntoResponse> {
    let ctx = builder
        .build(access_builder.build(change_set_id.into()))
        .await?;
    let change_set = ChangeSet::get_by_id(&ctx, change_set_id).await?;

    let index = match frigg.get_index(workspace_pk, change_set_id).await? {
        Some((index, _kv_revision)) => {
            let mv_index: MvIndex = serde_json::from_value(index.data.to_owned())
                .map_err(IndexError::DeserializingMvIndexData)?;

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
                if !request_rebuild_and_watch(&frigg, &edda_client, workspace_pk, change_set_id)
                    .await?
                {
                    // Return 202 Accepted with the same response body if the build didn't succeed in time
                    // to let the caller know the create succeeded, we're just waiting on downstream work
                    return Ok((StatusCode::ACCEPTED, Json(None)));
                }
                frigg
                    .get_index(workspace_pk, change_set_id)
                    .await?
                    .map(|i| i.0)
                    .ok_or(IndexError::IndexNotFoundAfterRebuild(
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
            if !request_rebuild_and_watch(&frigg, &edda_client, workspace_pk, change_set_id).await?
            {
                // Return 202 Accepted with the same response body if the build didn't succeed in time
                // to let the caller know the create succeeded, we're just waiting on downstream work
                return Ok((StatusCode::ACCEPTED, Json(None)));
            }
            frigg
                .get_index(workspace_pk, change_set_id)
                .await?
                .map(|i| i.0)
                .ok_or(IndexError::IndexNotFoundAfterFreshBuild(
                    workspace_pk,
                    change_set_id,
                ))?
        }
    };

    Ok((
        StatusCode::OK,
        Json(Some(FrontEndObjectMeta {
            workspace_snapshot_address: change_set.workspace_snapshot_address,
            front_end_object: index,
        })),
    ))
}
