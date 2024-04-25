use axum::extract::OriginalUri;
use axum::http::Uri;
use axum::Json;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use ulid::Ulid;

use dal::workspace_snapshot::graph::Direction;
use dal::{
    ChangeSet, DalContext, HistoryActor, User, Visibility, Workspace, WorkspacePk,
    WorkspaceSnapshot, WsEvent,
};
use module_index_client::types::{
    WorkspaceExport, WorkspaceExportContentV0, WorkspaceExportMetadataV0,
};
use si_layer_cache::db::serialize;
use telemetry::prelude::*;

use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient, RawAccessToken};
use crate::service::async_route::handle_error;
use crate::service::module::{ModuleError, ModuleResult};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportWorkspaceRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExportWorkspaceResponse {
    pub id: Ulid,
}

pub async fn export_workspace(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<ExportWorkspaceRequest>,
) -> ModuleResult<Json<ExportWorkspaceResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let task_id = Ulid::new();

    let workspace_pk = ctx.tenancy().workspace_pk().unwrap_or(WorkspacePk::NONE);
    let workspace = Workspace::get_by_pk(&ctx, &workspace_pk)
        .await?
        .ok_or(ModuleError::WorkspaceNotFound(workspace_pk))?;

    tokio::task::spawn(async move {
        if let Err(err) = export_workspace_inner(
            &ctx,
            workspace,
            &original_uri,
            PosthogClient(posthog_client),
            RawAccessToken(raw_access_token),
        )
        .await
        {
            return handle_error(&ctx, original_uri, task_id, err).await;
        }

        let event = match WsEvent::async_finish(&ctx, task_id).await {
            Ok(event) => event,
            Err(err) => {
                return handle_error(&ctx, original_uri, task_id, err).await;
            }
        };

        if let Err(err) = event.publish_on_commit(&ctx).await {
            return handle_error(&ctx, original_uri, task_id, err).await;
        };

        if let Err(err) = ctx.commit().await {
            handle_error(&ctx, original_uri, task_id, err).await;
        }
    });

    Ok(Json(ExportWorkspaceResponse { id: task_id }))
}

// This is all very experimental, so the code is all in place
// For import, we should isolate the structs to make the data formats clearer
// In the future, this should move to a replacement of si-pkg, probably
pub async fn export_workspace_inner(
    ctx: &DalContext,
    workspace: Workspace,
    _original_uri: &Uri,
    PosthogClient(_posthog_client): PosthogClient,
    RawAccessToken(raw_access_token): RawAccessToken,
) -> ModuleResult<()> {
    info!("Exporting workspace backup");

    let version = Utc::now().format("%Y-%m-%d_%H:%M:%S").to_string();

    let index_client = {
        let module_index_url = match ctx.module_index_url() {
            Some(url) => url,
            None => return Err(ModuleError::ModuleIndexNotConfigured),
        };

        module_index_client::IndexClient::new(module_index_url.try_into()?, &raw_access_token)
    };

    let mut content_hashes = vec![];
    let mut workspace_snapshots_for_changeset_id = HashMap::new();
    for change_set in ChangeSet::list_open(ctx).await? {
        let snap = WorkspaceSnapshot::find_for_change_set(ctx, change_set.id).await?;

        // From root, get every value from every node, store with hash
        let mut queue = VecDeque::from([snap.root().await?]);

        while let Some(this_node_idx) = queue.pop_front() {
            // Queue contents
            content_hashes.push(snap.get_node_weight(this_node_idx).await?.content_hash());

            let children = snap
                .edges_directed_by_index(this_node_idx, Direction::Outgoing)
                .await?
                .into_iter()
                .map(|(_, _, target)| target)
                .collect::<VecDeque<_>>();

            queue.extend(children)
        }

        // compress and store snap
        workspace_snapshots_for_changeset_id
            .insert(change_set.id.into_inner(), snap.serialized().await?);
    }

    let store_values_map = ctx
        .layer_db()
        .cas()
        .read_many(content_hashes.as_ref())
        .await?;
    let content_store_values = serialize::to_vec(&store_values_map)?;

    let HistoryActor::User(user_pk) = ctx.history_actor() else {
        return Err(ModuleError::ExportingFromSystemActor);
    };

    let user = User::get_by_pk(ctx, *user_pk)
        .await?
        .ok_or(ModuleError::InvalidUser(*user_pk))?;

    let metadata = WorkspaceExportMetadataV0 {
        name: workspace.name().clone(),
        version: version.clone(),
        description: "Workspace Backup".to_string(), // TODO Get this from the user
        created_at: Default::default(),
        created_by: user.email().clone(),
        default_change_set: Some(workspace.default_change_set_id().to_string()),
        workspace_pk: Some(workspace.pk().to_string()),
        workspace_name: Some(workspace.name().clone()),
    };

    let workspace_payload = {
        WorkspaceExport::new(WorkspaceExportContentV0 {
            workspace_snapshots_for_changeset_id,
            content_store_values,
            metadata,
        })
    };

    index_client
        .upload_workspace(workspace.name().as_str(), &version, workspace_payload)
        .await?;

    // track(
    //     &posthog_client,
    //     ctx,
    //     original_uri,
    //     "export_workspace",
    //     serde_json::json!({
    //                 "pkg_name": workspace.name().to_owned(),
    //                 "pkg_version": version,
    //                 "pkg_created_by_name": created_by_name,
    //                 "pkg_created_by_email": created_by_email,
    //                 "pkg_hash": response.latest_hash,
    //     }),
    // );

    ctx.commit().await?;

    Ok(())
}
