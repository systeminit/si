use std::collections::{BTreeMap, HashMap, HashSet};

use axum::{
    extract::{Host, OriginalUri, Path, Query},
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use dal::{
    attribute::prototype::argument::{AttributePrototypeArgument, AttributePrototypeArgumentError},
    cached_module::CachedModule,
    func::{
        argument::{FuncArgument, FuncArgumentError},
        authoring::{FuncAuthoringClient, FuncAuthoringError},
        binding::{
            action::ActionBinding, attribute::AttributeBinding, authentication::AuthBinding,
            leaf::LeafBinding, management::ManagementBinding, EventualParent, FuncBindingError,
        },
        FuncKind,
    },
    pkg::PkgError,
    prop::{PropError, PropPath},
    schema::variant::{
        authoring::{VariantAuthoringClient, VariantAuthoringError},
        leaves::{LeafInputLocation, LeafKind},
    },
    socket::{input::InputSocketError, output::OutputSocketError},
    workspace::WorkspaceId,
    ChangeSet, ChangeSetId, DalContext, Func, FuncId, InputSocket, OutputSocket, Prop, Schema,
    SchemaId, SchemaVariant, SchemaVariantId, WsEvent, WsEventError,
};
use hyper::StatusCode;
use si_events::{audit_log::AuditLogKind, ActionKind};
use si_frontend_types::{
    fs::{
        self, AssetFuncs, AttributeInputFrom, AttributeOutputTo, Binding, ChangeSet as FsChangeSet,
        Func as FsFunc, Schema as FsSchema, SchemaAttributes, SetFuncCodeRequest, VariantQuery,
    },
    AttributeArgumentBinding, FuncBinding,
};
use thiserror::Error;

use crate::{
    extract::{HandlerContext, PosthogClient},
    service::ApiError,
};

use super::{
    func::{
        binding::update_binding::{
            update_action_func_bindings, update_attribute_func_bindings, update_leaf_func_bindings,
            update_mangement_func_bindings,
        },
        get_code_response, FuncAPIError,
    },
    AccessBuilder, AppState,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum FsError {
    #[error("attribute func bound to neither output socket nor prop")]
    AttributeFuncNotBound,
    #[error("attribute input bound to neither input socket nor prop")]
    AttributeInputNotBound,
    #[error("attribute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
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
    FuncApi(#[from] FuncAPIError),
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
    #[error("Schema named {0} could not be found")]
    SchemaNotFoundWithName(String),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] dal::SchemaVariantError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] dal::TransactionsError),
    #[error("variant authoring error: {0}")]
    VariantAuthoring(#[from] VariantAuthoringError),
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
    Json(request): Json<fs::CreateChangeSetRequest>,
) -> FsResult<Json<fs::CreateChangeSetResponse>> {
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
) -> FsResult<Json<fs::ListChangeSetsResponse>> {
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

    let Some(kind) = fs::kind_from_string(&kind) else {
        return Ok(Json(vec![]));
    };

    Ok(Json(
        dal::Func::list_all(&ctx)
            .await?
            .into_iter()
            .filter(|f| kind == f.kind.into())
            // We do not render bindings for the "change-set/functions" folder yet
            .map(|f| dal_func_to_fs_func(f, 0))
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

    let Some(kind) = fs::kind_from_string(&kind) else {
        return Ok(Json(vec![]));
    };

    let mut funcs = vec![];

    for unlocked in [true, false] {
        if let Some(variant) = lookup_variant_for_schema(&ctx, schema_id, unlocked).await? {
            for func in dal::SchemaVariant::all_funcs_without_intrinsics(&ctx, variant.id())
                .await?
                .into_iter()
                .filter(|f| kind == f.kind.into())
            {
                let bindings = get_bindings(&ctx, func.id, variant.id).await?;
                let bindings_size = serde_json::to_vec_pretty(&bindings)?.len() as u64;

                funcs.push(dal_func_to_fs_func(func, bindings_size))
            }
        }
    }

    Ok(Json(funcs))
}

async fn get_bindings(
    ctx: &DalContext,
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
) -> FsResult<fs::Bindings> {
    let bindings =
        get_bindings_for_func_and_schema_variant(ctx, func_id, schema_variant_id).await?;

    let mut display_bindings = vec![];
    for binding in bindings {
        display_bindings.push(func_binding_to_fs_binding(ctx, binding).await?);
    }

    Ok(fs::Bindings {
        bindings: display_bindings,
    })
}

async fn func_binding_to_fs_binding(
    ctx: &DalContext,
    binding: FuncBinding,
) -> FsResult<fs::Binding> {
    Ok(match binding {
        FuncBinding::Action { kind, .. } => Binding::Action {
            kind: kind.unwrap_or(ActionKind::Manual),
        },
        FuncBinding::Attribute {
            prop_id,
            output_socket_id,
            argument_bindings,
            ..
        } => {
            attribute_binding_to_fs_attribute_binding(
                ctx,
                prop_id,
                output_socket_id,
                argument_bindings,
            )
            .await?
        }
        FuncBinding::Authentication { .. } => Binding::Authentication,
        FuncBinding::CodeGeneration { inputs, .. } => Binding::CodeGeneration { inputs },
        FuncBinding::Management {
            managed_schemas, ..
        } => management_binding_to_fs_management_binding(ctx, managed_schemas).await?,
        FuncBinding::Qualification { inputs, .. } => Binding::Qualification { inputs },
    })
}

async fn management_binding_to_fs_management_binding(
    ctx: &DalContext,
    managed_schemas: Option<Vec<SchemaId>>,
) -> FsResult<Binding> {
    Ok(if let Some(schemas) = managed_schemas {
        let mut managed_names = vec![];
        for managed_schema_id in schemas {
            let schema_name =
                match CachedModule::latest_by_schema_id(ctx, managed_schema_id).await? {
                    Some(cached_module) => cached_module.schema_name,
                    None => {
                        Schema::get_by_id_or_error(ctx, managed_schema_id)
                            .await?
                            .name
                    }
                };
            managed_names.push(schema_name);
        }

        Binding::Management {
            managed_schemas: Some(managed_names),
        }
    } else {
        Binding::Management {
            managed_schemas: None,
        }
    })
}

async fn attribute_binding_to_fs_attribute_binding(
    ctx: &DalContext,
    prop_id: Option<dal::PropId>,
    output_socket_id: Option<dal::OutputSocketId>,
    argument_bindings: Vec<si_frontend_types::AttributeArgumentBinding>,
) -> FsResult<Binding> {
    let output_to = if let Some(prop_id) = prop_id {
        let path = Prop::get_by_id(ctx, prop_id)
            .await?
            .path(ctx)
            .await?
            .with_replaced_sep("/");

        AttributeOutputTo::Prop(path)
    } else if let Some(output_socket_id) = output_socket_id {
        let name = OutputSocket::get_by_id(ctx, output_socket_id)
            .await?
            .name()
            .to_string();
        AttributeOutputTo::OutputSocket(name)
    } else {
        return Err(FsError::AttributeFuncNotBound);
    };
    let mut inputs = BTreeMap::new();
    for arg_binding in argument_bindings {
        let func_arg_name = FuncArgument::get_by_id_or_error(ctx, arg_binding.func_argument_id)
            .await?
            .name;
        let input_from = if let Some(input_prop_id) = arg_binding.prop_id {
            let path = Prop::get_by_id(ctx, input_prop_id)
                .await?
                .path(ctx)
                .await?
                .to_string();
            AttributeInputFrom::Prop(path)
        } else if let Some(input_socket_id) = arg_binding.input_socket_id {
            let name = InputSocket::get_by_id(ctx, input_socket_id)
                .await?
                .name()
                .to_string();
            AttributeInputFrom::InputSocket(name)
        } else {
            return Err(FsError::AttributeInputNotBound);
        };

        inputs.insert(func_arg_name, input_from);
    }
    Ok(Binding::Attribute { output_to, inputs })
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
        unlocked_attrs_size: 0,
        locked_attrs_size: 0,
    };

    result.locked = match lookup_variant_for_schema(&ctx, schema_id, false).await? {
        Some(variant) => {
            let asset_func = variant.get_asset_func(&ctx).await?;

            let attrs = make_schema_attrs(&variant);
            result.locked_attrs_size = attrs.byte_size();

            // Asset funcs do not have bindings (yet)
            Some(dal_func_to_fs_func(asset_func, 0))
        }
        None => None,
    };

    result.unlocked = match lookup_variant_for_schema(&ctx, schema_id, true).await? {
        Some(variant) => {
            let asset_func = variant.get_asset_func(&ctx).await?;

            let attrs = make_schema_attrs(&variant);
            result.unlocked_attrs_size = attrs.byte_size();

            Some(dal_func_to_fs_func(asset_func, 0))
        }
        None => None,
    };

    if result.locked.is_none() && result.unlocked.is_none() {
        Err(FsError::ResourceNotFound)
    } else {
        Ok(Json(result))
    }
}

async fn get_bindings_for_func_and_schema_variant(
    ctx: &DalContext,
    func_id: FuncId,
    schema_variant_id: SchemaVariantId,
) -> FsResult<Vec<FuncBinding>> {
    let func = dal::Func::get_by_id_or_error(ctx, func_id).await?;

    Ok(match func.kind {
        dal::func::FuncKind::Action => {
            ActionBinding::assemble_action_bindings(ctx, func_id).await?
        }
        dal::func::FuncKind::Attribute => {
            AttributeBinding::assemble_attribute_bindings(ctx, func_id).await?
        }
        dal::func::FuncKind::Authentication => {
            AuthBinding::assemble_auth_bindings(ctx, func_id).await?
        }
        dal::func::FuncKind::CodeGeneration => {
            LeafBinding::assemble_leaf_func_bindings(ctx, func_id, LeafKind::CodeGeneration).await?
        }
        dal::func::FuncKind::Management => {
            ManagementBinding::assemble_management_bindings(ctx, func_id).await?
        }
        dal::func::FuncKind::Qualification => {
            LeafBinding::assemble_leaf_func_bindings(ctx, func_id, LeafKind::Qualification).await?
        }
        dal::func::FuncKind::Unknown
        | dal::func::FuncKind::Intrinsic
        | dal::func::FuncKind::SchemaVariantDefinition => vec![],
    }
    .into_iter()
    .filter(|binding| binding.get_schema_variant() == Some(schema_variant_id))
    .map(Into::into)
    .collect())
}

fn dal_func_to_fs_func(func: dal::Func, bindings_size: u64) -> FsFunc {
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
        name: func.name,
        bindings_size,
    }
}

async fn get_func_bindings(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    Path((_workspace_id, change_set_id, schema_id, func_id)): Path<(
        WorkspaceId,
        ChangeSetId,
        SchemaId,
        FuncId,
    )>,
    Query(variant_query): Query<VariantQuery>,
) -> FsResult<Json<fs::Bindings>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set(&ctx)?;

    let variant = lookup_variant_for_schema(&ctx, schema_id, variant_query.unlocked)
        .await?
        .ok_or(FsError::ResourceNotFound)?;

    let display_bindings = get_bindings(&ctx, func_id, variant.id()).await?;

    Ok(Json(display_bindings))
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

async fn create_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
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

    let name = request.name.as_str();

    if dal::func::is_intrinsic(name)
        || ["si:resourcePayloadToValue", "si:normalizeToArray"].contains(&name)
    {
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
        Binding::Attribute {
            output_to: _,
            inputs: _,
        } => todo!(),
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
        Binding::Management { .. } => {
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

    let bindings = get_bindings(&ctx, func.id, unlocked_variant.id).await?;
    let bindings_size = serde_json::to_vec_pretty(&bindings)?.len() as u64;

    ctx.commit().await?;

    Ok(Json(dal_func_to_fs_func(func, bindings_size)))
}

async fn set_asset_func_code(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    Path((_workspace_id, change_set_id, schema_id, func_id)): Path<(
        WorkspaceId,
        ChangeSetId,
        SchemaId,
        FuncId,
    )>,
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
    if current_asset_func.id != func_id {
        // different error?
        return Err(FsError::ResourceNotFound);
    }

    FuncAuthoringClient::save_code(&ctx, func_id, request.code).await?;
    let func_code = get_code_response(&ctx, func_id).await?;

    WsEvent::func_code_saved(&ctx, func_code, false)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    let schema_variant_id = unlocked_variant.id;

    let updated_variant_id =
        VariantAuthoringClient::regenerate_variant(&ctx, unlocked_variant.id).await?;

    ctx.write_audit_log(
        AuditLogKind::RegenerateSchemaVariant { schema_variant_id },
        unlocked_variant.display_name().to_string(),
    )
    .await?;

    let updated_variant = SchemaVariant::get_by_id_or_error(&ctx, updated_variant_id).await?;

    if schema_variant_id == updated_variant_id {
        WsEvent::schema_variant_updated(&ctx, schema_id, updated_variant)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    } else {
        WsEvent::schema_variant_replaced(&ctx, schema_id, schema_variant_id, updated_variant)
            .await?
            .publish_on_commit(&ctx)
            .await?;
    }

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
        let default_variant_id = Schema::get_or_install_default_variant(&ctx, schema_id).await?;
        let variant = SchemaVariant::get_by_id_or_error(&ctx, default_variant_id).await?;

        let front_end_variant = variant.clone().into_frontend_type(&ctx, schema_id).await?;
        WsEvent::module_imported(&ctx, vec![front_end_variant.clone()])
            .await?
            .publish_on_commit(&ctx)
            .await?;
    }

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
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    Path((_workspace_id, change_set_id, schema_id)): Path<(WorkspaceId, ChangeSetId, SchemaId)>,
    Query(request): Query<VariantQuery>,
) -> FsResult<Json<SchemaAttributes>> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set(&ctx)?;

    let variant = lookup_variant_for_schema(&ctx, schema_id, request.unlocked)
        .await?
        .ok_or(FsError::ResourceNotFound)?;

    Ok(Json(make_schema_attrs(&variant)))
}

async fn set_schema_attrs(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    Path((_workspace_id, change_set_id, schema_id)): Path<(WorkspaceId, ChangeSetId, SchemaId)>,
    Json(attrs): Json<SchemaAttributes>,
) -> FsResult<()> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set_and_not_head(&ctx).await?;

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

async fn unlock_schema(
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

    check_change_set_and_not_head(&ctx).await?;

    let unlocked_variant = get_or_unlock_schema(&ctx, schema_id).await?;

    let mut result = AssetFuncs {
        locked: None,
        unlocked: None,
        unlocked_attrs_size: 0,
        locked_attrs_size: 0,
    };

    let asset_func = unlocked_variant.get_asset_func(&ctx).await?;

    let attrs = make_schema_attrs(&unlocked_variant);
    result.unlocked_attrs_size = attrs.byte_size();
    result.unlocked = Some(dal_func_to_fs_func(asset_func, 0));

    ctx.commit().await?;

    Ok(Json(result))
}

async fn unlock_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
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
    let existing_func = dal::Func::get_by_id(&ctx, func_id)
        .await?
        .ok_or(FsError::ResourceNotFound)?;
    if !existing_func.is_locked {
        return Err(FsError::FuncAlreadyUnlocked(func_id));
    }

    let variant = match lookup_variant_for_schema(&ctx, schema_id, true).await? {
        Some(variant) => variant,
        None => SchemaVariant::get_default_for_schema(&ctx, schema_id).await?,
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

    let bindings_size = get_bindings(&ctx, new_func.id, unlocked_variant.id())
        .await?
        .byte_size();

    ctx.commit().await?;

    // put in real attrs size of serialized bindings
    Ok(Json(dal_func_to_fs_func(new_func, bindings_size)))
}

async fn set_func_bindings(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(_posthog_client): PosthogClient,
    OriginalUri(_original_uri): OriginalUri,
    Host(_host_name): Host,
    Path((_workspace_id, change_set_id, schema_id, func_id)): Path<(
        WorkspaceId,
        ChangeSetId,
        SchemaId,
        FuncId,
    )>,
    Json(request): Json<fs::Bindings>,
) -> FsResult<()> {
    let ctx = builder
        .build(request_ctx.build(change_set_id.into()))
        .await?;

    check_change_set_and_not_head(&ctx).await?;

    let variant = lookup_variant_for_schema(&ctx, schema_id, true)
        .await?
        .ok_or(FsError::ResourceNotFound)?;

    let func = Func::get_by_id(&ctx, func_id)
        .await?
        .ok_or(FsError::ResourceNotFound)?;

    if matches!(
        func.kind,
        FuncKind::Authentication | FuncKind::Intrinsic | FuncKind::Unknown
    ) {
        return Err(FsError::FuncBindingKindMismatch);
    }

    let current_bindings =
        get_bindings_for_func_and_schema_variant(&ctx, func_id, variant.id()).await?;

    let updated_bindings = request.bindings;

    let mut delete_bindings = vec![];
    let mut final_bindings = vec![];
    for (idx, func_binding) in current_bindings.into_iter().enumerate() {
        match updated_bindings.get(idx) {
            Some(binding_update) => match binding_update {
                Binding::Action { kind: update_kind } => {
                    parse_action_bindings(&mut final_bindings, &func_binding, *update_kind)?;
                }
                Binding::Attribute { output_to, inputs } => {
                    parse_attr_bindings(&ctx, &mut final_bindings, func_binding, output_to, inputs)
                        .await?;
                }
                Binding::Authentication => {}
                Binding::CodeGeneration {
                    inputs: update_inputs,
                } => {
                    parse_code_gen_bindings(&mut final_bindings, func_binding, update_inputs)?;
                }
                Binding::Management {
                    managed_schemas: updated_schemas,
                } => {
                    parse_mgmt_bindings(&ctx, &mut final_bindings, func_binding, updated_schemas)
                        .await?;
                }
                Binding::Qualification {
                    inputs: update_inputs,
                } => {
                    parse_qualification_bindings(&mut final_bindings, func_binding, update_inputs)?;
                }
            },
            None => {
                delete_bindings.push(func_binding);
            }
        }
    }

    match func.kind {
        FuncKind::Attribute => {
            let cycle_check_guard = ctx.workspace_snapshot()?.enable_cycle_check().await;
            update_attribute_func_bindings(&ctx, final_bindings).await?;
            drop(cycle_check_guard);
        }
        FuncKind::Action => {
            update_action_func_bindings(&ctx, final_bindings).await?;
        }
        FuncKind::CodeGeneration | FuncKind::Qualification => {
            let cycle_check_guard = ctx.workspace_snapshot()?.enable_cycle_check().await;
            update_leaf_func_bindings(&ctx, final_bindings).await?;
            drop(cycle_check_guard);
        }
        FuncKind::Management => {
            update_mangement_func_bindings(&ctx, final_bindings).await?;
        }
        _ => return Err(FsError::FuncBindingKindMismatch),
    }

    // delete bindings that were not in the payload, create net new bindings

    let func_summary = Func::get_by_id_or_error(&ctx, func_id)
        .await?
        .into_frontend_type(&ctx)
        .await?;
    WsEvent::func_updated(&ctx, func_summary.clone(), None)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(())
}

fn parse_qualification_bindings(
    final_bindings: &mut Vec<FuncBinding>,
    func_binding: FuncBinding,
    update_inputs: &[si_frontend_types::LeafInputLocation],
) -> FsResult<()> {
    let FuncBinding::Qualification {
        schema_variant_id,
        component_id,
        func_id,
        attribute_prototype_id,
        ..
    } = func_binding
    else {
        return Err(FsError::FuncBindingKindMismatch);
    };

    final_bindings.push(FuncBinding::Qualification {
        schema_variant_id,
        component_id,
        func_id,
        attribute_prototype_id,
        inputs: update_inputs.to_owned(),
    });

    Ok(())
}

async fn parse_mgmt_bindings(
    ctx: &DalContext,
    final_bindings: &mut Vec<FuncBinding>,
    func_binding: FuncBinding,
    updated_schemas: &Option<Vec<String>>,
) -> FsResult<()> {
    let FuncBinding::Management {
        schema_variant_id,
        management_prototype_id,
        func_id,
        ..
    } = func_binding
    else {
        return Err(FsError::FuncBindingKindMismatch);
    };

    let latest_modules: HashMap<String, _> = CachedModule::latest_modules(ctx)
        .await?
        .into_iter()
        .map(|module| (module.schema_name, module.schema_id))
        .collect();
    let mut managed_schemas = vec![];
    if let Some(updated_schemas) = updated_schemas {
        for updated_schema in updated_schemas {
            let schema_id = match latest_modules.get(updated_schema) {
                Some(schema_id) => *schema_id,
                None => {
                    let Some(schema) = Schema::find_by_name(ctx, updated_schema).await? else {
                        return Err(FsError::SchemaNotFoundWithName(updated_schema.to_owned()));
                    };

                    schema.id()
                }
            };

            managed_schemas.push(schema_id);
        }
    }

    final_bindings.push(FuncBinding::Management {
        schema_variant_id,
        management_prototype_id,
        func_id,
        managed_schemas: if managed_schemas.is_empty() {
            None
        } else {
            Some(managed_schemas)
        },
    });

    Ok(())
}

fn parse_code_gen_bindings(
    final_bindings: &mut Vec<FuncBinding>,
    func_binding: FuncBinding,
    update_inputs: &[si_frontend_types::LeafInputLocation],
) -> FsResult<()> {
    let FuncBinding::CodeGeneration {
        schema_variant_id,
        component_id,
        func_id,
        attribute_prototype_id,
        ..
    } = func_binding
    else {
        return Err(FsError::FuncBindingKindMismatch);
    };

    final_bindings.push(FuncBinding::CodeGeneration {
        schema_variant_id,
        component_id,
        func_id,
        attribute_prototype_id,
        inputs: update_inputs.to_owned(),
    });

    Ok(())
}

async fn parse_attr_bindings(
    ctx: &DalContext,
    final_bindings: &mut Vec<FuncBinding>,
    func_binding: FuncBinding,
    output_to: &AttributeOutputTo,
    inputs: &BTreeMap<String, AttributeInputFrom>,
) -> FsResult<()> {
    let FuncBinding::Attribute {
        func_id,
        attribute_prototype_id,
        schema_variant_id,
        ..
    } = func_binding
    else {
        return Err(FsError::FuncBindingKindMismatch);
    };

    // todo: make special errors for all 3
    let schema_variant_id = schema_variant_id.ok_or(FsError::FuncBindingKindMismatch)?;
    let func_id = func_id.ok_or(FsError::FuncBindingKindMismatch)?;
    let proto_id = attribute_prototype_id.ok_or(FsError::FuncBindingKindMismatch)?;

    let (prop_id, output_socket_id) = match output_to {
        AttributeOutputTo::OutputSocket(name) => {
            let socket = OutputSocket::find_with_name(ctx, name, schema_variant_id)
                .await?
                .ok_or(FsError::OutputSocketNotFound(name.to_owned()))?;

            (None, Some(socket.id()))
        }
        AttributeOutputTo::Prop(prop_path_string) => {
            let prop_path = PropPath::new(prop_path_string.split("/"));
            let prop_id = Prop::find_prop_id_by_path_opt(ctx, schema_variant_id, &prop_path)
                .await?
                .ok_or(FsError::PropNotFound(prop_path_string.to_owned()))?;

            (Some(prop_id), None)
        }
    };

    let mut argument_bindings = vec![];
    for (arg_name, input_from) in inputs {
        let func_arg = FuncArgument::find_by_name_for_func(ctx, arg_name, func_id)
            .await?
            .ok_or(FsError::FuncArgumentNotFound(arg_name.to_owned()))?;

        let apa_id =
            AttributePrototypeArgument::find_by_func_argument_id_and_attribute_prototype_id(
                ctx,
                func_arg.id,
                proto_id,
            )
            .await?
            .ok_or(FsError::FuncBindingKindMismatch)?;

        let (prop_id, input_socket_id) = match input_from {
            AttributeInputFrom::InputSocket(input_socket_name) => {
                let socket = InputSocket::find_with_name(ctx, input_socket_name, schema_variant_id)
                    .await?
                    .ok_or(FsError::InputSocketNotFound(input_socket_name.to_owned()))?;

                (None, Some(socket.id()))
            }
            AttributeInputFrom::Prop(prop_path_string) => {
                let prop_path = PropPath::new(prop_path_string.split("/"));
                let prop_id = Prop::find_prop_id_by_path_opt(ctx, schema_variant_id, &prop_path)
                    .await?
                    .ok_or(FsError::PropNotFound(prop_path_string.to_owned()))?;

                (Some(prop_id), None)
            }
        };

        argument_bindings.push(AttributeArgumentBinding {
            func_argument_id: func_arg.id,
            attribute_prototype_argument_id: Some(apa_id),
            prop_id,
            input_socket_id,
            static_value: None,
        });
    }

    final_bindings.push(FuncBinding::Attribute {
        func_id: Some(func_id),
        attribute_prototype_id: Some(proto_id),
        component_id: None,
        schema_variant_id: Some(schema_variant_id),
        prop_id,
        output_socket_id,
        argument_bindings,
    });

    Ok(())
}

fn parse_action_bindings(
    final_bindings: &mut Vec<FuncBinding>,
    func_binding: &FuncBinding,
    update_kind: ActionKind,
) -> FsResult<()> {
    let FuncBinding::Action {
        schema_variant_id,
        action_prototype_id,
        func_id,
        ..
    } = *func_binding
    else {
        return Err(FsError::FuncBindingKindMismatch);
    };

    final_bindings.push(FuncBinding::Action {
        schema_variant_id,
        action_prototype_id,
        func_id,
        kind: Some(update_kind),
    });

    Ok(())
}

pub fn fs_routes() -> Router<AppState> {
    Router::new()
        .route("/change-sets", get(list_change_sets))
        .route("/change-sets/create", post(create_change_set))
        .nest(
            "/change-sets/:change_set_id",
            Router::new()
                .route("/funcs/:func_id/code", get(get_func_code))
                .route("/funcs/:func_id/code", post(set_func_code))
                .route("/funcs/:func_id/unlock", post(unlock_func))
                .route("/funcs/:kind", get(list_change_set_funcs))
                .route("/schemas", get(list_schemas))
                .route("/schemas/:schema_id/asset_funcs", get(get_asset_funcs))
                .route(
                    "/schemas/:schema_id/asset_func/:func_id",
                    post(set_asset_func_code),
                )
                .route("/schemas/:schema_id/attrs", get(get_schema_attrs))
                .route("/schemas/:schema_id/unlock", post(unlock_schema))
                .route("/schemas/:schema_id/attrs", post(set_schema_attrs))
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
                .route("/schemas/:schema_id/install", post(install_schema)),
        )
}
