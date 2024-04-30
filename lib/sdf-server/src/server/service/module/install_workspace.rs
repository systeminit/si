use axum::extract::OriginalUri;
use axum::http::Uri;
use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use ulid::Ulid;

use dal::layer_db_types::ContentTypes;
use dal::{ChangeSet, ChangeSetId, ContentHash, DalContext, Workspace, WorkspaceSnapshot, WsEvent};
use module_index_client::types::WorkspaceExportContentV0;
use module_index_client::IndexClient;
use si_layer_cache::db::serialize;

use crate::server::extract::RawAccessToken;
use crate::service::async_route::handle_error;
use crate::{
    server::extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::module::ModuleError,
};

use super::ModuleResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallWorkspaceRequest {
    pub id: Ulid,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallWorkspaceResponse {
    pub id: Ulid,
}

pub async fn install_workspace(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<InstallWorkspaceRequest>,
) -> ModuleResult<impl IntoResponse> {
    let ctx = builder.build_head(request_ctx).await?;

    let id = Ulid::new();
    tokio::task::spawn(async move {
        if let Err(err) = install_module_inner(
            &ctx,
            request,
            &original_uri,
            PosthogClient(posthog_client),
            raw_access_token,
        )
        .await
        {
            handle_error(&ctx, original_uri, id, err).await;
        } else {
            match WsEvent::async_finish_workspace(&ctx, id).await {
                Ok(event) => {
                    if let Err(err) = event.publish_immediately(&ctx).await {
                        handle_error(&ctx, original_uri, id, err).await;
                    }
                }
                Err(err) => {
                    handle_error(&ctx, original_uri, id, err).await;
                }
            }
        }
    });

    Ok(Json(InstallWorkspaceResponse { id }))
}

async fn install_module_inner(
    ctx: &DalContext,
    request: InstallWorkspaceRequest,
    _original_uri: &Uri,
    PosthogClient(_posthog_client): PosthogClient,
    raw_access_token: String,
) -> ModuleResult<()> {
    let WorkspaceExportContentV0 {
        change_sets,
        content_store_values,
        metadata,
    } = {
        let module_index_url = match ctx.module_index_url() {
            Some(url) => url,
            None => return Err(ModuleError::ModuleIndexNotConfigured),
        };
        let module_index_client = IndexClient::new(module_index_url.try_into()?, &raw_access_token);
        module_index_client
            .download_workspace(request.id)
            .await?
            .into_latest()
    };

    // ABANDON PREVIOUS CHANGESETS
    for mut change_set in ChangeSet::list_open(ctx).await? {
        change_set.abandon(ctx).await?;
    }

    let base_changeset_for_default = {
        let Some(workspace_pk) = ctx.tenancy().workspace_pk() else {
            return Err(ModuleError::ExportingImportingWithRootTenancy);
        };

        let workspace = Workspace::get_by_pk(ctx, &workspace_pk)
            .await?
            .ok_or(ModuleError::WorkspaceNotFound(workspace_pk))?;

        let changeset_id = workspace.default_change_set_id();

        let changeset = ChangeSet::find(ctx, changeset_id)
            .await?
            .ok_or(ModuleError::ChangeSetNotFound(changeset_id))?;

        changeset.base_change_set_id
    };

    // Go from head changeset to children, creating new changesets and updating base references
    let mut base_change_set_queue = VecDeque::from([metadata.default_change_set_base]);
    let mut change_set_id_map = HashMap::new();
    while let Some(base_change_set_ulid) = base_change_set_queue.pop_front() {
        let Some(change_sets) = change_sets.get(&base_change_set_ulid) else {
            continue;
        };

        for change_set_data in change_sets {
            let imported_snapshot =
                WorkspaceSnapshot::from_bytes(&change_set_data.workspace_snapshot_serialized_data)
                    .await?;

            // If base_change_set is default_change_set_base, it pointed to the builtin workspace
            // originally, so this change set needs to be the new default for the workspace - HEAD
            let mut is_new_default = false;
            let actual_base_changeset: Option<ChangeSetId> =
                if base_change_set_ulid == metadata.default_change_set_base {
                    is_new_default = true;
                    base_changeset_for_default
                } else {
                    Some(*change_set_id_map.get(&base_change_set_ulid).ok_or(
                        ModuleError::ImportingOrphanChangeset(base_change_set_ulid.into()),
                    )?)
                };

            let mut new_change_set =
                ChangeSet::new(ctx, change_set_data.name.clone(), actual_base_changeset).await?;

            change_set_id_map.insert(change_set_data.id, new_change_set.id);

            let new_snap_address = imported_snapshot
                .write(ctx, new_change_set.vector_clock_id())
                .await?;

            new_change_set.update_pointer(ctx, new_snap_address).await?;

            // Set new default changeset for workspace
            if is_new_default {
                ctx.tenancy().workspace_pk();
                let workspace_pk = ctx
                    .tenancy()
                    .workspace_pk()
                    .ok_or(ModuleError::ExportingImportingWithRootTenancy)?;
                let mut workspace = Workspace::get_by_pk(&ctx, &workspace_pk)
                    .await?
                    .ok_or(ModuleError::WorkspaceNotFound(workspace_pk))?;

                println!("{} is the new head", new_change_set.id);

                workspace
                    .update_default_change_set_id(ctx, new_change_set.id)
                    .await?;
            }

            base_change_set_queue.push_back(change_set_data.id)
        }
    }

    let cas_values: HashMap<ContentHash, (Arc<ContentTypes>, String)> =
        serialize::from_bytes(&content_store_values)?;

    println!("Importing {} cas value(s)", cas_values.len());

    let layer_db = ctx.layer_db();

    // TODO use the serialization format to ensure we're hashing the data correctly, if we change the format
    for (_, (content, _serialization_format)) in cas_values {
        layer_db
            .cas()
            .write(content, None, ctx.events_tenancy(), ctx.events_actor())
            .await?;
    }
    ctx.commit_no_rebase().await?;

    Ok(())
}
