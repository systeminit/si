use std::collections::HashSet;

use axum::{
    extract::{Host, OriginalUri, Path},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use dal::{
    cached_module::CachedModule,
    func::authoring::{FuncAuthoringClient, FuncAuthoringError},
    pkg::{import_pkg_from_pkg, ImportOptions, PkgError},
    workspace::WorkspaceId,
    ChangeSet, ChangeSetId, DalContext, FuncId, Schema, SchemaId, SchemaVariant, WsEvent,
    WsEventError,
};
use hyper::StatusCode;
use si_events::audit_log::AuditLogKind;
use si_frontend_types::fs::{
    AssetFuncs, ChangeSet as FsChangeSet, Func as FsFunc, Schema as FsSchema, SetFuncCodeRequest,
};
use thiserror::Error;

use crate::{
    extract::{HandlerContext, PosthogClient},
    service::ApiError,
};

use super::{
    func::{get_code_response, FuncAPIError},
    AccessBuilder, AppState,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FsError {
    #[error("cached module error: {0}")]
    CachedModule(#[from] dal::cached_module::CachedModuleError),
    #[error("cannot write to HEAD")]
    CannotWriteToHead,
    #[error("change set error: {0}")]
    ChangeSet(#[from] dal::ChangeSetError),
    #[error("ChangeSet {0}:{1} is inactive")]
    ChangeSetInactive(String, ChangeSetId),
    #[error("func error: {0}")]
    Func(#[from] dal::FuncError),
    #[error("func api error: {0}")]
    FuncApi(#[from] FuncAPIError),
    #[error("func authoring error: {0}")]
    FuncAuthoring(#[from] FuncAuthoringError),
    #[error("pkg error: {0}")]
    Pkg(#[from] PkgError),
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

async fn check_change_set_and_not_head(ctx: &DalContext) -> FsResult<()> {
    let change_set = ctx.change_set()?;
    if change_set.id == ctx.get_workspace_default_change_set_id().await? {
        return Err(FsError::CannotWriteToHead);
    }

    if change_set.status.is_active() {
        Ok(())
    } else {
        Err(FsError::ChangeSetInactive(
            change_set.name.clone(),
            change_set.id,
        ))
    }
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
            .map(dal_func_to_fs_func)
            .collect(),
    ))
}

async fn list_variant_funcs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    Path((_workspace_id, change_set_id, schema_id, kind)): Path<(
        WorkspaceId,
        ChangeSetId,
        SchemaId,
        String,
    )>,
) -> FsResult<Json<Vec<FsFunc>>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set(&ctx)?;

    let Some(kind) = si_frontend_types::fs::kind_from_string(&kind) else {
        return Ok(Json(vec![]));
    };

    let mut funcs = vec![];

    if let Some(locked_variant) = lookup_variant_for_schema(&ctx, schema_id, false).await? {
        funcs.extend(
            dal::SchemaVariant::all_funcs_without_intrinsics(&ctx, locked_variant.id())
                .await?
                .into_iter()
                .filter(|f| kind == f.kind.into())
                .map(dal_func_to_fs_func),
        );
    }

    if let Some(unlocked_variant) = lookup_variant_for_schema(&ctx, schema_id, true).await? {
        funcs.extend(
            dal::SchemaVariant::all_funcs_without_intrinsics(&ctx, unlocked_variant.id())
                .await?
                .into_iter()
                .filter(|f| kind == f.kind.into())
                .map(dal_func_to_fs_func),
        );
    }

    Ok(Json(funcs))
}

async fn lookup_variant_for_schema(
    ctx: &DalContext,
    schema_id: SchemaId,
    unlocked: bool,
) -> FsResult<Option<SchemaVariant>> {
    if ctx
        .workspace_snapshot()?
        .get_node_index_by_id_opt(schema_id)
        .await
        .is_none()
    {
        if CachedModule::latest_by_schema_id(ctx, schema_id)
            .await?
            .is_some()
        {
            return Ok(None);
        } else {
            return Err(FsError::ResourceNotFound);
        }
    }

    Ok(if unlocked {
        SchemaVariant::get_unlocked_for_schema(ctx, schema_id).await?
    } else {
        Some(SchemaVariant::get_default_for_schema(ctx, schema_id).await?)
    })
}

pub async fn get_asset_funcs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    Path((_workspace_id, change_set_id, schema_id)): Path<(WorkspaceId, ChangeSetId, SchemaId)>,
) -> FsResult<Json<AssetFuncs>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set(&ctx)?;

    let mut result = AssetFuncs {
        locked: None,
        unlocked: None,
    };

    result.locked = match lookup_variant_for_schema(&ctx, schema_id, false).await? {
        Some(variant) => {
            let asset_func = variant.get_asset_func(&ctx).await?;

            Some(dal_func_to_fs_func(asset_func))
        }
        None => None,
    };

    result.unlocked = match lookup_variant_for_schema(&ctx, schema_id, true).await? {
        Some(variant) => {
            let asset_func = variant.get_asset_func(&ctx).await?;

            Some(dal_func_to_fs_func(asset_func))
        }
        None => None,
    };

    if result.locked.is_none() && result.unlocked.is_none() {
        Err(FsError::ResourceNotFound)
    } else {
        Ok(Json(result))
    }
}

fn dal_func_to_fs_func(func: dal::Func) -> FsFunc {
    FsFunc {
        id: func.id,
        kind: func.kind.into(),
        is_locked: func.is_locked,
        code_size: func
            .code_plaintext()
            .ok()
            .flatten()
            .map(|code| code.as_bytes().len())
            .unwrap_or(0) as u64,
        name: func.name,
    }
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

async fn set_func_code(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    Path((_workspace_id, change_set_id, func_id)): Path<(WorkspaceId, ChangeSetId, FuncId)>,
    Json(request): Json<SetFuncCodeRequest>,
) -> FsResult<()> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set_and_not_head(&ctx).await?;

    FuncAuthoringClient::save_code(&ctx, func_id, request.code).await?;
    let func_code = get_code_response(&ctx, func_id).await?;

    WsEvent::func_code_saved(&ctx, func_code, false)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(())
}

async fn install_schema(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    Path((_workspace_id, change_set_id, schema_id)): Path<(WorkspaceId, ChangeSetId, SchemaId)>,
) -> FsResult<()> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set_and_not_head(&ctx).await?;

    if Schema::get_by_id(&ctx, schema_id).await?.is_none() {
        let mut uninstalled_module = CachedModule::latest_by_schema_id(&ctx, schema_id)
            .await?
            .ok_or(FsError::ResourceNotFound)?;

        let si_pkg = uninstalled_module.si_pkg(&ctx).await?;
        import_pkg_from_pkg(
            &ctx,
            &si_pkg,
            Some(ImportOptions {
                schema_id: Some(schema_id.into()),
                ..Default::default()
            }),
        )
        .await?;

        if let Some(default_variant_id) =
            Schema::get_default_schema_variant_by_id(&ctx, schema_id).await?
        {
            let variant = SchemaVariant::get_by_id_or_error(&ctx, default_variant_id).await?;

            let front_end_variant = variant.clone().into_frontend_type(&ctx, schema_id).await?;
            WsEvent::module_imported(&ctx, vec![front_end_variant.clone()])
                .await?
                .publish_on_commit(&ctx)
                .await?;
        }
    }

    ctx.commit().await?;

    Ok(())
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
                .route("/func-code/:func_id", post(set_func_code))
                .route("/schemas", get(list_schemas))
                .route("/schemas/:schema_id/asset_funcs", get(get_asset_funcs))
                .route("/schemas/:schema_id/funcs/:kind", get(list_variant_funcs))
                .route("/schemas/:schema_id/install", post(install_schema)),
        )
}
