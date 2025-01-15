use std::collections::HashSet;

use axum::{
    extract::{Host, OriginalUri, Path},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use dal::{
    cached_module::CachedModule, workspace::WorkspaceId, ChangeSet, ChangeSetId, DalContext,
    FuncId, SchemaId, SchemaVariant, WsEvent, WsEventError,
};
use hyper::StatusCode;
use si_events::audit_log::AuditLogKind;
use si_frontend_types::fs::{
    ChangeSet as FsChangeSet, Func as FsFunc, ListVariantsResponse, Schema as FsSchema,
};
use thiserror::Error;

use crate::{
    extract::{HandlerContext, PosthogClient},
    service::ApiError,
};

use super::{AccessBuilder, AppState};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FsError {
    #[error("cached module error: {0}")]
    CachedModule(#[from] dal::cached_module::CachedModuleError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("ChangeSet {0}:{1} is inactive")]
    ChangeSetInactive(String, ChangeSetId),
    #[error("func error: {0}")]
    Func(#[from] dal::FuncError),
    #[error("resource not found")]
    ResourceNotFound,
    #[error("schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

pub type FsResult<T> = Result<T, FsError>;

impl IntoResponse for FsError {
    fn into_response(self) -> Response {
        let status_code = match self {
            FsError::ChangeSetInactive(_, _) | FsError::ResourceNotFound => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        ApiError::new(status_code, self.to_string()).into_response()
    }
}

pub async fn create_change_set(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(access_builder): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    Path(_workspace_id): Path<WorkspaceId>,
    Json(request): Json<si_frontend_types::fs::CreateChangeSetRequest>,
) -> FsResult<Json<si_frontend_types::fs::CreateChangeSetResponse>> {
    let ctx = builder.build_head(access_builder).await?;

    let change_set = ChangeSet::fork_head(&ctx, request.name).await?;

    ctx.write_audit_log(AuditLogKind::CreateChangeSet, change_set.name.clone())
        .await?;

    WsEvent::change_set_created(&ctx, change_set.id)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit_no_rebase().await?;

    Ok(Json(FsChangeSet {
        name: change_set.name,
        id: change_set.id,
    }))
}

pub async fn list_change_sets(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    Path(_workspace_id): Path<WorkspaceId>,
) -> FsResult<Json<si_frontend_types::fs::ListChangeSetsResponse>> {
    let ctx = builder.build_head(request_ctx).await?;
    let open_change_sets = ChangeSet::list_active(&ctx).await?;

    Ok(Json(
        open_change_sets
            .into_iter()
            .map(|cs| FsChangeSet {
                name: cs.name,
                id: cs.id,
            })
            .collect(),
    ))
}

fn check_change_set(ctx: &DalContext) -> FsResult<()> {
    let change_set = ctx.change_set()?;
    if change_set.status.is_active() {
        Ok(())
    } else {
        Err(FsError::ChangeSetInactive(
            change_set.name.clone(),
            change_set.id,
        ))
    }
}

pub async fn list_schemas(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    Path((_workspace_id, change_set_id)): Path<(WorkspaceId, ChangeSetId)>,
) -> FsResult<Json<Vec<FsSchema>>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    // TODO: make this middleware
    check_change_set(&ctx)?;

    let mut result = vec![];

    let mut installed_set = HashSet::new();

    for schema in dal::Schema::list(&ctx).await? {
        installed_set.insert(schema.id());
        let default_variant = SchemaVariant::get_default_for_schema(&ctx, schema.id()).await?;
        result.push(FsSchema {
            installed: true,
            category: default_variant.category().to_string(),
            name: schema.name().to_string(),
            id: schema.id(),
        });
    }

    for module in CachedModule::latest_modules(&ctx)
        .await?
        .into_iter()
        .filter(|module| !installed_set.contains(&module.schema_id))
    {
        result.push(FsSchema {
            installed: false,
            category: module.category.unwrap_or_default(),
            name: module.schema_name,
            id: module.schema_id,
        });
    }

    Ok(Json(result))
}

pub async fn list_variants(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    Path((_workspace_id, change_set_id, schema_id)): Path<(WorkspaceId, ChangeSetId, SchemaId)>,
) -> FsResult<Json<ListVariantsResponse>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set(&ctx)?;

    if ctx
        .workspace_snapshot()?
        .get_node_index_by_id_opt(schema_id)
        .await
        .is_none()
        && CachedModule::latest_by_schema_id(&ctx, schema_id)
            .await?
            .is_some()
    {
        return Ok(Json(ListVariantsResponse {
            locked: None,
            unlocked: None,
        }));
    }

    let default_variant = SchemaVariant::get_default_for_schema(&ctx, schema_id).await?;

    Ok(Json(if !default_variant.is_locked() {
        ListVariantsResponse {
            locked: None,
            unlocked: Some(default_variant.id()),
        }
    } else {
        ListVariantsResponse {
            locked: Some(default_variant.id()),
            unlocked: SchemaVariant::get_unlocked_for_schema(&ctx, schema_id)
                .await?
                .map(|var| var.id()),
        }
    }))
}

async fn list_change_set_funcs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    Path((_workspace_id, change_set_id, kind)): Path<(WorkspaceId, ChangeSetId, String)>,
) -> FsResult<Json<Vec<FsFunc>>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set(&ctx)?;

    let Some(kind) = si_frontend_types::fs::kind_from_string(&kind) else {
        return Ok(Json(vec![]));
    };

    Ok(Json(
        dal::Func::list_all(&ctx)
            .await?
            .into_iter()
            .filter(|f| kind == f.kind.into())
            .map(|f| FsFunc {
                id: f.id,
                kind: f.kind.into(),
                is_locked: f.is_locked,
                code_size: f
                    .code_plaintext()
                    .ok()
                    .flatten()
                    .map(|code| code.len())
                    .unwrap_or(0) as u64,
                name: f.name,
            })
            .collect(),
    ))
}

async fn get_func_code(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    Path((_workspace_id, change_set_id, func_id)): Path<(WorkspaceId, ChangeSetId, FuncId)>,
) -> FsResult<String> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set(&ctx)?;

    let func = dal::Func::get_by_id(&ctx, func_id)
        .await?
        .ok_or(FsError::ResourceNotFound)?;

    Ok(func.code_plaintext()?.unwrap_or_default())
}

pub fn fs_routes() -> Router<AppState> {
    Router::new()
        .route("/change-sets", get(list_change_sets))
        .route("/change-sets/create", post(create_change_set))
        .nest(
            "/change-sets/:change_set_id",
            Router::new()
                .route("/funcs/:kind", get(list_change_set_funcs))
                .route("/func-code/:func_id", get(get_func_code))
                .route("/schemas", get(list_schemas))
                .route("/schemas/:schema_id/variants", get(list_variants)),
        )
}
