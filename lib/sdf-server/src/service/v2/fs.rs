use std::{
    collections::{
        BTreeMap,
        HashSet,
    },
    sync::Arc,
};

use axum::{
    Json,
    Router,
    extract::{
        Path,
        Query,
    },
    middleware,
    response::{
        IntoResponse,
        Response,
    },
    routing::{
        get,
        post,
    },
};
use dal::{
    ChangeSet,
    ChangeSetId,
    DalContext,
    FuncId,
    Schema,
    SchemaId,
    SchemaVariant,
    WsEvent,
    WsEventError,
    attribute::prototype::argument::AttributePrototypeArgumentError,
    cached_module::CachedModule,
    func::{
        argument::FuncArgumentError,
        authoring::{
            FuncAuthoringClient,
            FuncAuthoringError,
        },
        binding::{
            AttributeFuncDestination,
            EventualParent,
            FuncBindingError,
        },
    },
    pkg::PkgError,
    prop::PropError,
    schema::variant::{
        authoring::{
            VariantAuthoringClient,
            VariantAuthoringError,
        },
        leaves::{
            LeafInputLocation,
            LeafKind,
        },
    },
    slow_rt::{
        self,
        SlowRuntimeError,
    },
    socket::{
        input::InputSocketError,
        output::OutputSocketError,
    },
    workspace::WorkspaceId,
};
use hyper::StatusCode;
use si_db::Visibility;
use si_events::audit_log::AuditLogKind;
use si_frontend_types::fs::{
    self,
    AssetFuncs,
    Binding,
    CategoryFilter,
    ChangeSet as FsChangeSet,
    CreateSchemaResponse,
    FsApiError,
    Func as FsFunc,
    HydratedChangeSet,
    HydratedSchema,
    Schema as FsSchema,
    SchemaAttributes,
    SetFuncCodeRequest,
    VariantQuery,
};
use si_id::FuncArgumentId;
use thiserror::Error;

use super::{
    AccessBuilder,
    AppState,
    func::{
        FuncAPIError,
        get_code_response,
    },
};
use crate::extract::{
    HandlerContext,
    PosthogEventTracker,
    change_set::TargetChangeSetIdentFromPath,
    workspace::{
        AuthorizedForAutomationRole,
        TargetWorkspaceIdFromPath,
        WorkspaceDalContext,
    },
};

pub mod bindings;

use bindings::{
    get_bindings,
    get_func_bindings,
    get_identity_bindings,
    get_identity_bindings_for_variant,
    output_to_into_func_destination,
    set_func_bindings,
    set_identity_bindings,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FsError {
    #[error("attribute func bound to neither output socket nor prop")]
    AttributeFuncNotBound,
    #[error("attribute input bound to neither input socket nor prop")]
    AttributeInputNotBound,
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] Box<AttributePrototypeArgumentError>),
    #[error("no attribute protototype argument found for func argument {0}")]
    AttributePrototypeArgumentMissingForFuncArg(FuncArgumentId),
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
    #[error("func already unlocked: {0}")]
    FuncAlreadyUnlocked(FuncId),
    #[error("func api error: {0}")]
    FuncApi(#[from] Box<FuncAPIError>),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("func argument not found with name: {0}")]
    FuncArgumentNotFound(String),
    #[error("func authoring error: {0}")]
    FuncAuthoring(#[from] FuncAuthoringError),
    #[error("func binding error: {0}")]
    FuncBinding(#[from] FuncBindingError),
    #[error("binding kind mismatch")]
    FuncBindingKindMismatch,
    #[error("func name reserved")]
    FuncNameReserved,
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("no input socket found named {0}")]
    InputSocketNotFound(String),
    #[error("tokio join error: {0}")]
    Join(#[from] tokio::task::JoinError),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error("no output socket found named {0}")]
    OutputSocketNotFound(String),
    #[error("pkg error: {0}")]
    Pkg(#[from] PkgError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("prop not found at path {0}")]
    PropNotFound(String),
    #[error("resource not found")]
    ResourceNotFound,
    #[error("schema error: {0}")]
    Schema(#[from] dal::SchemaError),
    #[error("cannot unlock schema {0}")]
    SchemaCannotUnlock(SchemaId),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("slow runtime error: {0}")]
    SlowRuntime(#[from] SlowRuntimeError),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("variant authoring error: {0}")]
    VariantAuthoring(#[from] VariantAuthoringError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] dal::WorkspaceSnapshotError),
    #[error("ws event error: {0}")]
    WsEvent(#[from] WsEventError),
}

impl From<FuncAPIError> for FsError {
    fn from(value: FuncAPIError) -> Self {
        Box::new(value).into()
    }
}
impl From<AttributePrototypeArgumentError> for FsError {
    fn from(value: AttributePrototypeArgumentError) -> Self {
        Box::new(value).into()
    }
}

pub type FsResult<T> = Result<T, FsError>;

const ASSET_EDITOR_TYPES: &str = include_str!("../../../editor_typescript.txt");

impl IntoResponse for FsError {
    fn into_response(self) -> Response {
        let (status_code, error) = match self {
            FsError::ChangeSetInactive(_, change_set_id) => (
                StatusCode::NOT_FOUND,
                FsApiError::ChangeSetInactive(change_set_id),
            ),
            FsError::ResourceNotFound => (StatusCode::NOT_FOUND, FsApiError::ResourceNotFound),
            _ => (
                StatusCode::INTERNAL_SERVER_ERROR,
                FsApiError::InternalServerError(self.to_string()),
            ),
        };

        (status_code, Json(error)).into_response()
    }
}

pub async fn create_change_set(
    WorkspaceDalContext(ref ctx): WorkspaceDalContext,
    Json(request): Json<fs::CreateChangeSetRequest>,
) -> FsResult<Json<fs::CreateChangeSetResponse>> {
    let change_set = ChangeSet::fork_head(ctx, request.name).await?;

    ctx.write_audit_log(AuditLogKind::CreateChangeSet, change_set.name.clone())
        .await?;

    WsEvent::change_set_created(ctx, change_set.id, change_set.workspace_snapshot_address)
        .await?
        .publish_on_commit(ctx)
        .await?;

    ctx.commit_no_rebase().await?;

    Ok(Json(FsChangeSet {
        name: change_set.name,
        id: change_set.id,
    }))
}

/// Prefetches all functions for the entire workspace, so that the deno workspace json
/// can be prefilled with each function as deno "workspace"
pub async fn hydrate(
    WorkspaceDalContext(ref ctx): WorkspaceDalContext,
    tracker: PosthogEventTracker,
) -> FsResult<Json<Vec<HydratedChangeSet>>> {
    let open_change_sets = ChangeSet::list_active(ctx).await?;

    tracker.track(ctx, "fs/hydrate", serde_json::json!({}));

    let cached_modules = Arc::new(CachedModule::latest_user_independent_modules(ctx).await?);

    let mut hydrated_change_sets = vec![];
    for change_set in open_change_sets {
        let change_set_name = change_set.name.clone();
        let change_set_ctx = ctx.clone_with_new_visibility(Visibility::new(change_set.id));

        let ctx = change_set_ctx.clone();
        let cached_modules_clone = cached_modules.clone();
        let hydrated_change_set = slow_rt::spawn(async move {
            let ctx = &ctx;
            let all_funcs: BTreeMap<FuncId, dal::Func> = dal::Func::list_all(ctx)
                .await?
                .into_iter()
                .map(|func| (func.id, func))
                .collect();

            let mut change_set_funcs = vec![];
            // prefetch all bindings and types
            for func in all_funcs.values() {
                if !matches!(
                    func.kind,
                    dal::func::FuncKind::Action
                        | dal::func::FuncKind::Attribute
                        | dal::func::FuncKind::Authentication
                        | dal::func::FuncKind::CodeGeneration
                        | dal::func::FuncKind::Qualification
                        | dal::func::FuncKind::Management
                ) {
                    continue;
                }

                change_set_funcs.push(dal_func_to_fs_func(func, 0, 0));
            }

            let mut hydrated_schemas = vec![];
            let schemas = get_schema_list(
                ctx,
                CategoryFilter { category: None },
                Some(cached_modules_clone.as_slice()),
            )
            .await?;
            for schema in schemas {
                let asset_funcs =
                    asset_funcs_for_schema(ctx, schema.id, Some(&cached_modules_clone)).await?;
                let mut schema_funcs = vec![];
                let variants = match (asset_funcs.locked.is_some(), asset_funcs.unlocked.is_some())
                {
                    (true, true) => vec![true, false],
                    (true, false) => vec![false],
                    (false, true) => vec![true],
                    (false, false) => {
                        hydrated_schemas.push(HydratedSchema {
                            schema,
                            asset_funcs,
                            funcs: None,
                        });
                        continue;
                    }
                };

                for unlocked in variants {
                    let Some(variant) = lookup_variant_for_schema_with_prefetched_modules(
                        ctx,
                        schema.id,
                        unlocked,
                        &Some(&cached_modules_clone),
                    )
                    .await?
                    else {
                        continue;
                    };

                    for func_id in SchemaVariant::all_func_ids(ctx, variant.id()).await? {
                        let Some(func) = all_funcs.get(&func_id) else {
                            continue;
                        };

                        if func.is_intrinsic() {
                            continue;
                        }

                        schema_funcs.push(dal_func_to_fs_func(func, 0, 0))
                    }
                }

                hydrated_schemas.push(HydratedSchema {
                    schema,
                    asset_funcs,
                    funcs: Some(schema_funcs),
                });
            }

            Ok::<HydratedChangeSet, FsError>(HydratedChangeSet {
                name: change_set_name,
                id: change_set.id,
                funcs: change_set_funcs,
                schemas: hydrated_schemas,
            })
        })?
        .await??;

        hydrated_change_sets.push(hydrated_change_set);
    }

    Ok(Json(hydrated_change_sets))
}

pub async fn list_change_sets(
    WorkspaceDalContext(ref ctx): WorkspaceDalContext,
    tracker: PosthogEventTracker,
) -> FsResult<Json<fs::ListChangeSetsResponse>> {
    let open_change_sets = ChangeSet::list_active(ctx).await?;

    tracker.track(ctx, "fs/list_change_sets", serde_json::json!({}));

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

pub async fn list_schema_categories(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id)): Path<(WorkspaceId, ChangeSetId)>,
) -> FsResult<Json<Vec<String>>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set(&ctx)?;

    tracker.track(&ctx, "fs/list_categories", serde_json::json!({}));

    let mut categories = HashSet::new();

    for schema in dal::Schema::list(&ctx).await? {
        let default_variant = SchemaVariant::default_for_schema(&ctx, schema.id()).await?;
        categories.insert(default_variant.category().to_string());
    }

    for module in CachedModule::latest_user_independent_modules(&ctx).await? {
        categories.insert(module.category.unwrap_or("".into()));
    }

    let cats = categories.into_iter().collect();

    Ok(Json(cats))
}

pub async fn list_schemas(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id)): Path<(WorkspaceId, ChangeSetId)>,
    Query(cat_filter): Query<CategoryFilter>,
) -> FsResult<Json<Vec<FsSchema>>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set(&ctx)?;

    tracker.track(&ctx, "fs/list_schemas", serde_json::json!({}));

    let result = get_schema_list(&ctx, cat_filter, None).await?;

    Ok(Json(result))
}

async fn get_schema_list(
    ctx: &DalContext,
    cat_filter: CategoryFilter,
    cached_modules: Option<&[CachedModule]>,
) -> Result<Vec<FsSchema>, FsError> {
    let mut result = vec![];
    let mut installed_set = HashSet::new();
    for schema in dal::Schema::list(ctx).await? {
        installed_set.insert(schema.id());
        let default_variant = SchemaVariant::default_for_schema(ctx, schema.id()).await?;

        if cat_filter.should_skip(default_variant.category()) {
            continue;
        }

        result.push(FsSchema {
            installed: true,
            category: default_variant.category().to_string(),
            name: schema.name().to_string(),
            id: schema.id(),
        });
    }

    #[allow(unused)]
    let mut cached_module_list = vec![];
    for module in match cached_modules {
        Some(cms) => cms,
        None => {
            cached_module_list = CachedModule::latest_user_independent_modules(ctx).await?;
            cached_module_list.as_slice()
        }
    }
    .iter()
    .filter(|module| !installed_set.contains(&module.schema_id))
    {
        if cat_filter.should_skip(module.category.as_deref().unwrap_or("")) {
            continue;
        }

        result.push(FsSchema {
            installed: false,
            category: module.category.clone().unwrap_or_default(),
            name: module.schema_name.clone(),
            id: module.schema_id,
        });
    }
    Ok(result)
}

async fn list_change_set_funcs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id, kind)): Path<(WorkspaceId, ChangeSetId, String)>,
) -> FsResult<Json<Vec<FsFunc>>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set(&ctx)?;

    let Some(kind) = fs::kind_from_string(&kind) else {
        return Ok(Json(vec![]));
    };

    tracker.track(
        &ctx,
        "fs/list_change_set_funcs",
        serde_json::json!({ "func_kind": kind }),
    );

    let funcs = dal::Func::list_all(&ctx).await?;
    let mut result = Vec::new();
    for func in funcs {
        if kind == func.kind.into() {
            let func_id = func.id;
            result.push(dal_func_to_fs_func(
                &func,
                0,
                func_types_size(&ctx, func_id).await?,
            ));
        }
    }

    Ok(Json(result))
}

async fn list_variant_funcs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
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

    let Some(kind) = fs::kind_from_string(&kind) else {
        return Ok(Json(vec![]));
    };

    tracker.track(
        &ctx,
        "fs/list_variant_funcs",
        serde_json::json!({
            "schema_id": schema_id,
            "func_kind": kind,
        }),
    );

    let mut funcs = vec![];

    for unlocked in [true, false] {
        if let Some(variant) = lookup_variant_for_schema(&ctx, schema_id, unlocked).await? {
            for func in dal::SchemaVariant::all_funcs_without_intrinsics(&ctx, variant.id())
                .await?
                .into_iter()
                .filter(|f| kind == f.kind.into())
            {
                let bindings_size = get_bindings(&ctx, func.id, schema_id).await?.0.byte_size();
                let types_size = func_types_size(&ctx, func.id).await?;
                funcs.push(dal_func_to_fs_func(&func, bindings_size, types_size))
            }
        }
    }

    Ok(Json(funcs))
}

async fn lookup_variant_for_schema(
    ctx: &DalContext,
    schema_id: SchemaId,
    unlocked: bool,
) -> FsResult<Option<SchemaVariant>> {
    lookup_variant_for_schema_with_prefetched_modules(ctx, schema_id, unlocked, &None).await
}

async fn lookup_variant_for_schema_with_prefetched_modules(
    ctx: &DalContext,
    schema_id: SchemaId,
    unlocked: bool,
    cached_modules: &Option<&[CachedModule]>,
) -> FsResult<Option<SchemaVariant>> {
    if !ctx.workspace_snapshot()?.node_exists(schema_id).await {
        if let Some(cached_modules) = cached_modules {
            if cached_modules.iter().any(|m| m.schema_id == schema_id) {
                return Ok(None);
            } else {
                return Err(FsError::ResourceNotFound);
            }
        } else if CachedModule::find_latest_for_schema_id(ctx, schema_id)
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
        Some(SchemaVariant::default_for_schema(ctx, schema_id).await?)
    })
}

pub async fn get_asset_funcs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id, schema_id)): Path<(WorkspaceId, ChangeSetId, SchemaId)>,
) -> FsResult<Json<AssetFuncs>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set(&ctx)?;

    tracker.track(
        &ctx,
        "fs/get_asset_funcs",
        serde_json::json!({
            "schema_id": schema_id
        }),
    );

    let result = asset_funcs_for_schema(&ctx, schema_id, None).await?;

    if result.locked.is_none() && result.unlocked.is_none() {
        Err(FsError::ResourceNotFound)
    } else {
        Ok(Json(result))
    }
}

async fn asset_funcs_for_schema(
    ctx: &DalContext,
    schema_id: SchemaId,
    cached_modules: Option<&[CachedModule]>,
) -> FsResult<AssetFuncs> {
    let mut result = AssetFuncs {
        locked: None,
        unlocked: None,
        unlocked_attrs_size: 0,
        unlocked_bindings_size: 0,
        locked_attrs_size: 0,
        locked_bindings_size: 0,
        types_size: ASSET_EDITOR_TYPES.len() as u64,
    };

    result.locked = match lookup_variant_for_schema_with_prefetched_modules(
        ctx,
        schema_id,
        false,
        &cached_modules,
    )
    .await?
    {
        Some(variant) => {
            if !variant.is_locked() {
                None
            } else {
                let asset_func = variant.get_asset_func(ctx).await?;

                let attrs = make_schema_attrs(&variant);
                result.locked_attrs_size = attrs.byte_size();

                let bindings = get_identity_bindings_for_variant(ctx, variant.id()).await?;
                result.locked_bindings_size = bindings.byte_size();

                Some(dal_func_to_fs_func(&asset_func, 0, 0))
            }
        }
        None => None,
    };

    result.unlocked = match lookup_variant_for_schema_with_prefetched_modules(
        ctx,
        schema_id,
        true,
        &cached_modules,
    )
    .await?
    {
        Some(variant) => {
            let asset_func = variant.get_asset_func(ctx).await?;

            let attrs = make_schema_attrs(&variant);
            result.unlocked_attrs_size = attrs.byte_size();
            let bindings = get_identity_bindings_for_variant(ctx, variant.id()).await?;
            result.unlocked_bindings_size = bindings.byte_size();

            Some(dal_func_to_fs_func(&asset_func, 0, 0))
        }
        None => None,
    };

    Ok(result)
}

fn dal_func_to_fs_func(func: &dal::Func, bindings_size: u64, types_size: u64) -> FsFunc {
    FsFunc {
        id: func.id,
        kind: func.kind.into(),
        is_locked: func.is_locked,
        code_size: func
            .code_plaintext()
            .ok()
            .flatten()
            .map(|code| code.len())
            .unwrap_or(0) as u64,
        name: func.name.to_string(),
        bindings_size,
        types_size,
    }
}

async fn get_func_code(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id, func_id)): Path<(WorkspaceId, ChangeSetId, FuncId)>,
) -> FsResult<String> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set(&ctx)?;

    tracker.track(
        &ctx,
        "fs/get_func_code",
        serde_json::json!({ "func_id": func_id }),
    );

    let func = dal::Func::get_by_id_opt(&ctx, func_id)
        .await?
        .ok_or(FsError::ResourceNotFound)?;

    Ok(func.code_plaintext()?.unwrap_or_default())
}

async fn set_func_code(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id, func_id)): Path<(WorkspaceId, ChangeSetId, FuncId)>,
    Json(request): Json<SetFuncCodeRequest>,
) -> FsResult<()> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set_and_not_head(&ctx).await?;

    tracker.track(
        &ctx,
        "fs/set_func_code",
        serde_json::json!({ "func_id": func_id }),
    );

    FuncAuthoringClient::save_code(&ctx, func_id, request.code).await?;
    let func_code = get_code_response(&ctx, func_id).await?;

    WsEvent::func_code_saved(&ctx, func_code, false)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(())
}

async fn func_types_size(ctx: &DalContext, func_id: FuncId) -> FsResult<u64> {
    let func = dal::Func::get_by_id(ctx, func_id).await?;
    Ok(func.get_types(ctx).await?.len() as u64)
}

async fn get_func_types(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id, func_id)): Path<(WorkspaceId, ChangeSetId, FuncId)>,
) -> FsResult<String> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set(&ctx)?;

    tracker.track(
        &ctx,
        "fs/get_func_types",
        serde_json::json!({"func_id": func_id}),
    );

    let func = dal::Func::get_by_id_opt(&ctx, func_id)
        .await?
        .ok_or(FsError::ResourceNotFound)?;

    let types = func.get_types(&ctx).await?;

    Ok(types)
}

async fn create_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id, schema_id, kind_string)): Path<(
        WorkspaceId,
        ChangeSetId,
        SchemaId,
        String,
    )>,
    Json(request): Json<fs::CreateFuncRequest>,
) -> FsResult<Json<FsFunc>> {
    let kind = fs::kind_from_string(&kind_string).ok_or(FsError::FuncBindingKindMismatch)?;

    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set_and_not_head(&ctx).await?;

    if !request.binding.kind_matches(kind) {
        return Err(FsError::FuncBindingKindMismatch);
    }

    tracker.track(
        &ctx,
        "fs/create_func",
        serde_json::json!({
            "schema_id": schema_id,
            "func_kind": kind,
            "payload": &request
        }),
    );

    let name = request.name.as_str();

    if dal::func::is_intrinsic(name) {
        return Err(FsError::FuncNameReserved);
    }

    let unlocked_variant = get_or_unlock_schema(&ctx, schema_id).await?;

    let (attach_audit_log, func) = match request.binding {
        Binding::Action { kind } => {
            let func = FuncAuthoringClient::create_new_action_func(
                &ctx,
                Some(request.name),
                kind.into(),
                unlocked_variant.id(),
            )
            .await?;
            (
                AuditLogKind::AttachActionFunc {
                    func_id: func.id,
                    func_display_name: func.display_name.clone(),
                    schema_variant_id: Some(unlocked_variant.id()),
                    component_id: None,
                    action_kind: kind.into(),
                },
                func,
            )
        }
        Binding::Attribute { output_to, .. } => {
            let output_location =
                output_to_into_func_destination(&ctx, &output_to, unlocked_variant.id()).await?;
            let argument_bindings = vec![];

            let func = FuncAuthoringClient::create_new_attribute_func(
                &ctx,
                Some(request.name),
                Some(EventualParent::SchemaVariant(unlocked_variant.id())),
                output_location,
                argument_bindings,
            )
            .await?;

            (
                AuditLogKind::AttachAttributeFunc {
                    func_id: func.id,
                    func_display_name: func.display_name.clone(),
                    schema_variant_id: Some(unlocked_variant.id()),
                    component_id: None,
                    subject_name: unlocked_variant.display_name().to_string(),
                    prop_id: match output_location {
                        AttributeFuncDestination::Prop(prop_id) => Some(prop_id),
                        _ => None,
                    },
                    output_socket_id: match output_location {
                        AttributeFuncDestination::OutputSocket(output_socket_id) => {
                            Some(output_socket_id)
                        }
                        _ => None,
                    },
                    destination_name: output_location.get_name_of_destination(&ctx).await?,
                },
                func,
            )
        }
        Binding::Authentication => {
            let func = FuncAuthoringClient::create_new_auth_func(
                &ctx,
                Some(request.name),
                unlocked_variant.id(),
            )
            .await?;
            (
                AuditLogKind::AttachAuthFunc {
                    func_id: func.id,
                    func_display_name: func.display_name.clone(),
                    schema_variant_id: Some(unlocked_variant.id()),
                },
                func,
            )
        }
        Binding::CodeGeneration { .. } => {
            let func = FuncAuthoringClient::create_new_leaf_func(
                &ctx,
                Some(request.name),
                LeafKind::CodeGeneration,
                EventualParent::SchemaVariant(unlocked_variant.id()),
                &[LeafInputLocation::Domain],
            )
            .await?;

            (
                AuditLogKind::AttachCodeGenFunc {
                    func_id: func.id,
                    func_display_name: func.display_name.clone(),
                    schema_variant_id: Some(unlocked_variant.id()),
                    component_id: None,
                    subject_name: unlocked_variant.display_name().to_owned(),
                },
                func,
            )
        }
        Binding::Management => {
            let func = FuncAuthoringClient::create_new_management_func(
                &ctx,
                Some(request.name),
                unlocked_variant.id(),
            )
            .await?;

            (
                AuditLogKind::AttachManagementFunc {
                    func_id: func.id,
                    func_display_name: func.display_name.clone(),
                    schema_variant_id: Some(unlocked_variant.id()),
                    component_id: None,
                    subject_name: unlocked_variant.display_name().to_string(),
                },
                func,
            )
        }
        Binding::Qualification { .. } => {
            let func = FuncAuthoringClient::create_new_leaf_func(
                &ctx,
                Some(request.name),
                LeafKind::Qualification,
                EventualParent::SchemaVariant(unlocked_variant.id()),
                &[LeafInputLocation::Domain, LeafInputLocation::Code],
            )
            .await?;
            (
                AuditLogKind::AttachQualificationFunc {
                    func_id: func.id,
                    func_display_name: func.display_name.clone(),
                    schema_variant_id: Some(unlocked_variant.id()),
                    component_id: None,
                    subject_name: unlocked_variant.display_name().to_owned(),
                },
                func,
            )
        }
    };

    ctx.write_audit_log(
        AuditLogKind::CreateFunc {
            func_display_name: func.display_name.clone(),
            func_kind: func.kind.into(),
        },
        func.name.clone(),
    )
    .await?;
    ctx.write_audit_log(attach_audit_log, func.name.clone())
        .await?;

    let summary = func.into_frontend_type(&ctx).await?;
    WsEvent::func_created(&ctx, summary)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    let bindings_size = get_bindings(&ctx, func.id, schema_id).await?.0.byte_size();
    let types_size = func_types_size(&ctx, func.id).await?;

    ctx.commit().await?;

    Ok(Json(dal_func_to_fs_func(&func, bindings_size, types_size)))
}

async fn get_asset_func_code(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id, schema_id)): Path<(WorkspaceId, ChangeSetId, SchemaId)>,
    Query(variant_query): Query<VariantQuery>,
) -> FsResult<String> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set(&ctx)?;

    tracker.track(
        &ctx,
        "fs/get_asset_func_code",
        serde_json::json!({
            "schema_id": schema_id
        }),
    );

    let variant = lookup_variant_for_schema(&ctx, schema_id, variant_query.unlocked)
        .await?
        .ok_or(FsError::ResourceNotFound)?;

    let func = variant.get_asset_func(&ctx).await?;

    Ok(func.code_plaintext()?.unwrap_or_default())
}

async fn set_asset_func_code(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id, schema_id)): Path<(WorkspaceId, ChangeSetId, SchemaId)>,
    Json(request): Json<SetFuncCodeRequest>,
) -> FsResult<()> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set_and_not_head(&ctx).await?;

    let unlocked_variant = lookup_variant_for_schema(&ctx, schema_id, true)
        .await?
        .ok_or(FsError::ResourceNotFound)?;

    let current_asset_func = unlocked_variant.get_asset_func(&ctx).await?;
    let func_id = current_asset_func.id;

    FuncAuthoringClient::save_code(&ctx, func_id, request.code).await?;
    let func_code = get_code_response(&ctx, func_id).await?;

    WsEvent::func_code_saved(&ctx, func_code, false)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    let schema_variant_id = unlocked_variant.id;

    let ctx_clone = ctx.clone();
    let unlocked_variant_display_name = unlocked_variant.display_name().to_owned();

    tracker.track(
        &ctx,
        "fs/set_asset_func_code",
        serde_json::json!({
            "schema_id": schema_id
        }),
    );

    // Commit code changes
    ctx.commit().await?;

    // Now regen the variant in a separate transaction/rebase
    let updated_variant_id =
        VariantAuthoringClient::regenerate_variant(&ctx_clone, schema_variant_id).await?;

    ctx_clone
        .write_audit_log(
            AuditLogKind::RegenerateSchemaVariant { schema_variant_id },
            unlocked_variant_display_name,
        )
        .await?;

    let updated_variant = SchemaVariant::get_by_id(&ctx_clone, updated_variant_id).await?;

    if schema_variant_id == updated_variant_id {
        WsEvent::schema_variant_updated(&ctx_clone, schema_id, updated_variant)
            .await?
            .publish_on_commit(&ctx_clone)
            .await?;
    } else {
        WsEvent::schema_variant_replaced(&ctx_clone, schema_id, schema_variant_id, updated_variant)
            .await?
            .publish_on_commit(&ctx_clone)
            .await?;
    }

    ctx_clone.commit().await?;

    Ok(())
}

async fn install_schema(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id, schema_id)): Path<(WorkspaceId, ChangeSetId, SchemaId)>,
) -> FsResult<()> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set_and_not_head(&ctx).await?;

    if !Schema::exists_locally(&ctx, schema_id).await? {
        let default_variant_id = Schema::get_or_install_default_variant(&ctx, schema_id).await?;
        let variant = SchemaVariant::get_by_id(&ctx, default_variant_id).await?;

        let front_end_variant = variant.clone().into_frontend_type(&ctx, schema_id).await?;
        WsEvent::module_imported(&ctx, vec![front_end_variant.clone()])
            .await?
            .publish_on_commit(&ctx)
            .await?;
    }

    tracker.track(
        &ctx,
        "fs/install_schema",
        serde_json::json!({
            "schema_id": schema_id
        }),
    );

    ctx.commit().await?;

    Ok(())
}

fn make_schema_attrs(variant: &SchemaVariant) -> SchemaAttributes {
    SchemaAttributes {
        category: variant.category().to_owned(),
        description: variant.description(),
        display_name: variant.display_name().to_owned(),
        link: variant.link(),
        color: variant.color().to_owned(),
        component_type: variant.component_type().into(),
    }
}

async fn get_schema_attrs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id, schema_id)): Path<(WorkspaceId, ChangeSetId, SchemaId)>,
    Query(request): Query<VariantQuery>,
) -> FsResult<Json<SchemaAttributes>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set(&ctx)?;

    tracker.track(
        &ctx,
        "fs/get_schema_attrs",
        serde_json::json!({
            "schema_id": schema_id
        }),
    );

    let variant = lookup_variant_for_schema(&ctx, schema_id, request.unlocked)
        .await?
        .ok_or(FsError::ResourceNotFound)?;

    Ok(Json(make_schema_attrs(&variant)))
}

async fn set_schema_attrs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id, schema_id)): Path<(WorkspaceId, ChangeSetId, SchemaId)>,
    Json(attrs): Json<SchemaAttributes>,
) -> FsResult<()> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set_and_not_head(&ctx).await?;

    tracker.track(
        &ctx,
        "fs/set_schema_attrs",
        serde_json::json!({
            "schema_id": schema_id, "payload": &attrs
        }),
    );

    let variant = lookup_variant_for_schema(&ctx, schema_id, true)
        .await?
        .ok_or(FsError::ResourceNotFound)?;

    let schema = variant.schema(&ctx).await?;
    let schema_name = schema.name();

    VariantAuthoringClient::save_variant_content(
        &ctx,
        variant.id,
        schema_name,
        &attrs.display_name,
        &attrs.category,
        attrs.description.clone(),
        attrs.link.clone(),
        &attrs.color,
        attrs.component_type.into(),
        None::<String>,
    )
    .await?;

    WsEvent::schema_variant_saved(
        &ctx,
        schema.id(),
        variant.id(),
        schema_name.to_string(),
        attrs.category,
        attrs.color,
        attrs.component_type.into(),
        attrs.link,
        attrs.description,
        attrs.display_name,
    )
    .await?
    .publish_on_commit(&ctx)
    .await?;

    ctx.commit().await?;

    Ok(())
}

async fn get_asset_func_types(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id, schema_id)): Path<(WorkspaceId, ChangeSetId, SchemaId)>,
) -> FsResult<String> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set(&ctx)?;

    tracker.track(
        &ctx,
        "fs/get_asset_func_types",
        serde_json::json!({
            "schema_id": schema_id
        }),
    );

    Ok(ASSET_EDITOR_TYPES.into())
}

async fn get_or_unlock_schema(ctx: &DalContext, schema_id: SchemaId) -> FsResult<SchemaVariant> {
    Ok(
        match lookup_variant_for_schema(ctx, schema_id, true).await? {
            Some(unlocked) => unlocked,
            None => {
                let locked_variant = lookup_variant_for_schema(ctx, schema_id, false)
                    .await?
                    // make a more specific error here?
                    .ok_or(FsError::SchemaCannotUnlock(schema_id))?;

                // The locked is unlocked (could be that the default is unlocked)
                if !locked_variant.is_locked() {
                    return Ok(locked_variant);
                }

                let unlocked_variant =
                    VariantAuthoringClient::create_unlocked_variant_copy(ctx, locked_variant.id())
                        .await?;

                ctx.write_audit_log(
                    AuditLogKind::UnlockSchemaVariant {
                        schema_variant_id: unlocked_variant.id(),
                        schema_variant_display_name: unlocked_variant.display_name().to_owned(),
                    },
                    locked_variant.schema(ctx).await?.name().to_owned(),
                )
                .await?;

                WsEvent::schema_variant_created(ctx, schema_id, unlocked_variant.clone())
                    .await?
                    .publish_on_commit(ctx)
                    .await?;
                unlocked_variant
            }
        },
    )
}

async fn create_schema(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id)): Path<(WorkspaceId, ChangeSetId)>,
    Json(request): Json<fs::CreateSchemaRequest>,
) -> FsResult<Json<CreateSchemaResponse>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set_and_not_head(&ctx).await?;

    let created_schema_variant = VariantAuthoringClient::create_schema_and_variant(
        &ctx,
        request.name.clone(),
        None::<String>,
        None::<String>,
        request.category.unwrap_or("".into()),
        "#00AAFF".to_string(),
    )
    .await?;

    let schema = created_schema_variant.schema(&ctx).await?;

    tracker.track(
        &ctx,
        "fs/create_schema",
        serde_json::json!({
            "schema_id": schema.id(),
            "schema_name": schema.name().to_owned(),
        }),
    );

    WsEvent::schema_variant_created(&ctx, schema.id(), created_schema_variant.clone())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.write_audit_log(
        AuditLogKind::CreateSchemaVariant {
            schema_id: schema.id(),
            schema_variant_id: created_schema_variant.id(),
        },
        created_schema_variant.display_name().to_string(),
    )
    .await?;

    ctx.commit().await?;

    Ok(Json(CreateSchemaResponse {
        schema_id: schema.id(),
        name: schema.name().to_string(),
    }))
}

async fn unlock_schema(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id, schema_id)): Path<(WorkspaceId, ChangeSetId, SchemaId)>,
) -> FsResult<Json<AssetFuncs>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set_and_not_head(&ctx).await?;

    tracker.track(
        &ctx,
        "fs/unlock_schema",
        serde_json::json!({
            "schema_id": schema_id
        }),
    );

    let unlocked_variant = get_or_unlock_schema(&ctx, schema_id).await?;

    let mut result = AssetFuncs {
        locked: None,
        unlocked: None,
        unlocked_attrs_size: 0,
        locked_attrs_size: 0,
        unlocked_bindings_size: 0,
        locked_bindings_size: 0,
        types_size: ASSET_EDITOR_TYPES.len() as u64,
    };

    let asset_func = unlocked_variant.get_asset_func(&ctx).await?;

    let attrs = make_schema_attrs(&unlocked_variant);
    result.unlocked_attrs_size = attrs.byte_size();
    result.unlocked_bindings_size = get_identity_bindings_for_variant(&ctx, unlocked_variant.id())
        .await?
        .byte_size();
    result.unlocked = Some(dal_func_to_fs_func(&asset_func, 0, 0));

    ctx.commit().await?;

    Ok(Json(result))
}

async fn unlock_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    tracker: PosthogEventTracker,
    Path((_workspace_id, change_set_id, schema_id, func_id)): Path<(
        WorkspaceId,
        ChangeSetId,
        SchemaId,
        FuncId,
    )>,
) -> FsResult<Json<FsFunc>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set_and_not_head(&ctx).await?;

    let existing_func = dal::Func::get_by_id_opt(&ctx, func_id)
        .await?
        .ok_or(FsError::ResourceNotFound)?;
    if !existing_func.is_locked {
        return Err(FsError::FuncAlreadyUnlocked(func_id));
    }

    tracker.track(
        &ctx,
        "fs/unlock_func",
        serde_json::json!({
            "func_id": func_id,
            "schema_id": schema_id,
        }),
    );

    let variant = match lookup_variant_for_schema(&ctx, schema_id, true).await? {
        Some(variant) => variant,
        None => SchemaVariant::default_for_schema(&ctx, schema_id).await?,
    };
    let original_variant_id = variant.id();

    let new_func =
        FuncAuthoringClient::create_unlocked_func_copy(&ctx, func_id, Some(variant.id())).await?;

    let unlocked_variant = if variant.is_locked() {
        lookup_variant_for_schema(&ctx, schema_id, true)
            .await?
            .ok_or(FsError::SchemaCannotUnlock(schema_id))?
    } else {
        variant
    };

    let summary = new_func.into_frontend_type(&ctx).await?;
    WsEvent::func_created(&ctx, summary.clone())
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.write_audit_log(
        AuditLogKind::UnlockFunc {
            func_id,
            func_display_name: new_func.display_name.clone(),
            schema_variant_id: Some(original_variant_id),
            component_id: None,
            // XXX: should this be the *schema* name?
            subject_name: Some(unlocked_variant.display_name().to_owned()),
        },
        new_func.name.clone(),
    )
    .await?;

    let bindings_size = get_bindings(&ctx, new_func.id, schema_id)
        .await?
        .0
        .byte_size();
    let types_size = func_types_size(&ctx, new_func.id).await?;

    ctx.commit().await?;

    // put in real attrs size of serialized bindings
    Ok(Json(dal_func_to_fs_func(
        &new_func,
        bindings_size,
        types_size,
    )))
}

pub fn fs_routes(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/hydrate", get(hydrate))
        .route("/change-sets", get(list_change_sets))
        .route("/change-sets/create", post(create_change_set))
        .nest(
            "/change-sets/:change_set_id",
            Router::new()
                .route("/funcs/:func_id/code", get(get_func_code))
                .route("/funcs/:func_id/code", post(set_func_code))
                .route("/funcs/:func_id/types", get(get_func_types))
                .route("/funcs/:func_id/unlock", post(unlock_func))
                .route("/funcs/:kind", get(list_change_set_funcs))
                .route("/schemas", get(list_schemas))
                .route("/schemas/categories", get(list_schema_categories))
                .route("/schemas/create", post(create_schema))
                .route("/schemas/:schema_id/asset_funcs", get(get_asset_funcs))
                .route("/schemas/:schema_id/asset_func", post(set_asset_func_code))
                .route("/schemas/:schema_id/asset_func", get(get_asset_func_code))
                .route(
                    "/schemas/:schema_id/asset_func/types",
                    get(get_asset_func_types),
                )
                .route("/schemas/:schema_id/attrs", get(get_schema_attrs))
                .route("/schemas/:schema_id/unlock", post(unlock_schema))
                .route("/schemas/:schema_id/attrs", post(set_schema_attrs))
                .route("/schemas/:schema_id/bindings", get(get_identity_bindings))
                .route("/schemas/:schema_id/bindings", post(set_identity_bindings))
                .route(
                    "/schemas/:schema_id/funcs/:func_id/unlock",
                    post(unlock_func),
                )
                .route(
                    "/schemas/:schema_id/funcs/:func_id/bindings",
                    get(get_func_bindings),
                )
                .route(
                    "/schemas/:schema_id/funcs/:func_id/bindings",
                    post(set_func_bindings),
                )
                .route("/schemas/:schema_id/funcs/:kind", get(list_variant_funcs))
                .route("/schemas/:schema_id/funcs/:kind/create", post(create_func))
                .route("/schemas/:schema_id/install", post(install_schema))
                .route_layer(middleware::from_extractor::<TargetChangeSetIdentFromPath>()),
        )
        .route_layer(middleware::from_extractor_with_state::<
            AuthorizedForAutomationRole,
            AppState,
        >(state))
        .route_layer(middleware::from_extractor::<TargetWorkspaceIdFromPath>())
}
