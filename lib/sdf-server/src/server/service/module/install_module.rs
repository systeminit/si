use axum::extract::OriginalUri;
use axum::http::Uri;
use axum::{response::IntoResponse, Json};
use serde::{Deserialize, Serialize};
use ulid::Ulid;

use dal::{pkg::import_pkg_from_pkg, ChangeSet, DalContext, Visibility, WsEvent};
use module_index_client::IndexClient;
use si_pkg::{SiPkg, SiPkgKind};

use crate::server::extract::RawAccessToken;
use crate::service::async_route::handle_error;
use crate::{
    server::extract::{AccessBuilder, HandlerContext, PosthogClient},
    service::module::ModuleError,
};

use super::ModuleResult;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallModuleRequest {
    pub id: Ulid,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct InstallModuleResponse {
    pub id: Ulid,
}

pub async fn install_module(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    RawAccessToken(raw_access_token): RawAccessToken,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<InstallModuleRequest>,
) -> ModuleResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

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
            match WsEvent::async_finish(&ctx, id).await {
                Ok(event) => match event.publish_on_commit(&ctx).await {
                    Ok(()) => {
                        if let Err(err) = ctx.commit().await {
                            handle_error(&ctx, original_uri, id, err).await;
                        }
                    }
                    Err(err) => {
                        handle_error(&ctx, original_uri, id, err).await;
                    }
                },
                Err(err) => {
                    handle_error(&ctx, original_uri, id, err).await;
                }
            }
        }
    });

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(serde_json::to_string(&InstallModuleResponse { id })?)?)
}

async fn install_module_inner(
    ctx: &DalContext,
    request: InstallModuleRequest,
    _original_uri: &Uri,
    PosthogClient(_posthog_client): PosthogClient,
    raw_access_token: String,
) -> ModuleResult<()> {
    let module_index_url = match ctx.module_index_url() {
        Some(url) => url,
        None => return Err(ModuleError::ModuleIndexNotConfigured),
    };

    let module_index_client = IndexClient::new(module_index_url.try_into()?, &raw_access_token);
    let pkg_data = module_index_client.download_module(request.id).await?;

    let pkg = SiPkg::load_from_bytes(pkg_data)?;
    let metadata = pkg.metadata()?;
    let (_, svs, _import_skips) = import_pkg_from_pkg(ctx, &pkg, None).await?;

    // track(
    //     &posthog_client,
    //     ctx,
    //     original_uri,
    //     "install_pkg",
    //     serde_json::json!({
    //                 "pkg_name": metadata.name().to_owned(),
    //     }),
    // );
    //
    // let user_pk = match ctx.history_actor() {
    //     HistoryActor::User(user_pk) => {
    //         let user = User::get_by_pk(ctx, *user_pk)
    //             .await?
    //             .ok_or(PkgError::InvalidUser(*user_pk))?;
    //
    //         Some(user.pk())
    //     }
    //
    //     HistoryActor::SystemInit => None,
    // };

    if metadata.kind() == SiPkgKind::Module {
        WsEvent::module_imported(ctx, svs)
            .await?
            .publish_on_commit(ctx)
            .await?;
    }

    // match metadata.kind() {
    //     SiPkgKind::Module => {
    //         WsEvent::module_imported(ctx, svs)
    //             .await?
    //             .publish_on_commit(ctx)
    //             .await?;
    //     }
    // SiPkgKind::WorkspaceBackup => {
    //     let workspace_pk = match metadata.workspace_pk() {
    //         Some(workspace_pk) => Some(WorkspacePk::from_str(workspace_pk)?),
    //         None => None,
    //     };
    //
    //     WsEvent::workspace_imported(ctx, workspace_pk, user_pk)
    //         .await?
    //         .publish_on_commit(ctx)
    //         .await?
    // }
    // _ => {}
    // }

    ctx.commit().await?;

    Ok(())
}
