use std::collections::HashSet;

use axum::{extract::Path, Json};

use dal::{ChangeSet, ChangeSetId, WorkspacePk};
use si_frontend_types::{index::MvIndex, reference::ReferenceKind};
use telemetry::prelude::*;

use crate::extract::{EddaClient, FriggStore, HandlerContext};

use super::request_rebuild_and_watch;
use super::{AccessBuilder, FrontEndObjectMeta, IndexError, IndexResult};

pub async fn get_change_set_index(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    FriggStore(frigg): FriggStore,
    EddaClient(edda_client): EddaClient,
    Path((workspace_pk, change_set_id)): Path<(WorkspacePk, ChangeSetId)>,
) -> IndexResult<Json<FrontEndObjectMeta>> {
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
            let span = info_span!("sdf.index.get_change_set_index.implemented_kinds");
            let implemented_kinds = span.in_scope(|| {
                let mut implemented_kinds = HashSet::new();
                for index_ref in mv_index.mv_list {
                    let kind = ReferenceKind::try_from(index_ref.kind.as_str())
                        .map_err(IndexError::InvalidStringForReferenceKind)?;
                    if kind.is_revision_sensitive() {
                        implemented_kinds.insert(kind);
                    }
                }
                IndexResult::Ok(implemented_kinds)
            })?;

            if implemented_kinds == ReferenceKind::revision_sensitive() {
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
                    .ok_or(IndexError::IndexNotFound(workspace_pk, change_set_id))?
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
                .ok_or(IndexError::IndexNotFound(workspace_pk, change_set_id))?
        }
    };

    Ok(Json(FrontEndObjectMeta {
        workspace_snapshot_address: change_set.workspace_snapshot_address,
        front_end_object: index,
    }))
}
