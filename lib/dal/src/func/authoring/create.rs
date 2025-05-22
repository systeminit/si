use base64::{
    Engine,
    engine::general_purpose,
};
use telemetry::prelude::*;

use super::{
    FuncAuthoringError,
    FuncAuthoringResult,
};
use crate::{
    DalContext,
    Func,
    FuncBackendKind,
    FuncBackendResponseType,
    SchemaVariantId,
    action::prototype::{
        ActionKind,
        ActionPrototype,
    },
    func::binding::{
        AttributeArgumentBinding,
        AttributeFuncDestination,
        EventualParent,
        action::ActionBinding,
        attribute::AttributeBinding,
        authentication::AuthBinding,
        leaf::LeafBinding,
        management::ManagementBinding,
    },
    generate_name,
    schema::variant::leaves::{
        LeafInputLocation,
        LeafKind,
    },
};

static DEFAULT_CODE_HANDLER: &str = "main";
static DEFAULT_ATTRIBUTE_CODE: &str = include_str!("data/defaults/attribute.ts");
static DEFAULT_CODE_GENERATION_CODE: &str = include_str!("data/defaults/code_generation.ts");
static DEFAULT_QUALIFICATION_CODE: &str = include_str!("data/defaults/qualification.ts");
static DEFAULT_ACTION_CODE: &str = include_str!("data/defaults/action.ts");
static DEFAULT_AUTHENTICATION_CODE: &str = include_str!("data/defaults/authentication.ts");
static DEFAULT_MGMT_CODE: &str = include_str!("data/defaults/management.ts");

#[allow(dead_code)]
static DEFAULT_VALIDATION_CODE: &str = include_str!("data/defaults/validation.ts");

#[instrument(
    name = "func.authoring.create_func.create.management",
    level = "debug",
    skip(ctx)
)]
pub(crate) async fn create_management_func(
    ctx: &DalContext,
    name: Option<String>,
    schema_variant_id: SchemaVariantId,
) -> FuncAuthoringResult<Func> {
    let func = create_func_stub(
        ctx,
        name.clone(),
        FuncBackendKind::Management,
        FuncBackendResponseType::Management,
        DEFAULT_MGMT_CODE,
        DEFAULT_CODE_HANDLER,
    )
    .await?;

    ManagementBinding::create_management_binding(ctx, func.id, schema_variant_id).await?;

    Ok(func)
}

#[instrument(
    name = "func.authoring.create_func.create.action",
    level = "debug",
    skip(ctx)
)]
pub(crate) async fn create_action_func(
    ctx: &DalContext,
    name: Option<String>,
    action_kind: ActionKind,
    schema_variant_id: SchemaVariantId,
) -> FuncAuthoringResult<Func> {
    // need to see if there's already an action func of this particular kind for the schema variant
    // before doing anything else

    let exising_actions = ActionPrototype::for_variant(ctx, schema_variant_id).await?;
    for action in exising_actions {
        if action.kind == action_kind && action_kind != ActionKind::Manual {
            return Err(FuncAuthoringError::ActionKindAlreadyExists(
                action_kind,
                schema_variant_id,
            ));
        }
    }

    let func = create_func_stub(
        ctx,
        name.clone(),
        FuncBackendKind::JsAction,
        FuncBackendResponseType::Action,
        DEFAULT_ACTION_CODE,
        DEFAULT_CODE_HANDLER,
    )
    .await?;

    ActionBinding::create_action_binding(ctx, func.id, action_kind, schema_variant_id).await?;

    Ok(func)
}

#[instrument(
    name = "func.authoring.create_func.create.leaf",
    level = "debug",
    skip(ctx)
)]
pub(crate) async fn create_leaf_func(
    ctx: &DalContext,
    name: Option<String>,
    leaf_kind: LeafKind,
    eventual_parent: EventualParent,
    inputs: &[LeafInputLocation],
) -> FuncAuthoringResult<Func> {
    let (code, handler, backend_kind, backend_response_type) = match leaf_kind {
        LeafKind::CodeGeneration => (
            DEFAULT_CODE_GENERATION_CODE,
            DEFAULT_CODE_HANDLER,
            FuncBackendKind::JsAttribute,
            FuncBackendResponseType::CodeGeneration,
        ),
        LeafKind::Qualification => (
            DEFAULT_QUALIFICATION_CODE,
            DEFAULT_CODE_HANDLER,
            FuncBackendKind::JsAttribute,
            FuncBackendResponseType::Qualification,
        ),
    };
    let func = create_func_stub(
        ctx,
        name,
        backend_kind,
        backend_response_type,
        code,
        handler,
    )
    .await?;
    LeafBinding::create_leaf_func_binding(ctx, func.id, eventual_parent, leaf_kind, inputs).await?;
    Ok(func)
}

#[instrument(
    name = "func.authoring.create_func.create.attribute",
    level = "debug",
    skip(ctx)
)]
pub(crate) async fn create_attribute_func(
    ctx: &DalContext,
    name: Option<String>,
    eventual_parent: Option<EventualParent>,
    output_location: Option<AttributeFuncDestination>,
    argument_bindings: Vec<AttributeArgumentBinding>,
) -> FuncAuthoringResult<Func> {
    let (code, handler, backend_kind, backend_response_type) = (
        DEFAULT_ATTRIBUTE_CODE,
        DEFAULT_CODE_HANDLER,
        FuncBackendKind::JsAttribute,
        FuncBackendResponseType::Unset,
    );

    let func = create_func_stub(
        ctx,
        name,
        backend_kind,
        backend_response_type,
        code,
        handler,
    )
    .await?;

    if let Some(output_location) = output_location {
        AttributeBinding::upsert_attribute_binding(
            ctx,
            func.id,
            eventual_parent,
            output_location,
            argument_bindings,
        )
        .await?;
    }

    Ok(func)
}

#[instrument(
    name = "func.authoring.create_func.create.authentication",
    level = "debug",
    skip(ctx)
)]
pub(crate) async fn create_authentication_func(
    ctx: &DalContext,
    name: Option<String>,
    schema_variant_id: SchemaVariantId,
) -> FuncAuthoringResult<Func> {
    let func = create_func_stub(
        ctx,
        name,
        FuncBackendKind::JsAuthentication,
        FuncBackendResponseType::Void,
        DEFAULT_AUTHENTICATION_CODE,
        DEFAULT_CODE_HANDLER,
    )
    .await?;

    AuthBinding::create_auth_binding(ctx, func.id, schema_variant_id).await?;
    Ok(func)
}

async fn create_func_stub(
    ctx: &DalContext,
    name: Option<String>,
    backend_kind: FuncBackendKind,
    backend_response_type: FuncBackendResponseType,
    code: &str,
    handler: &str,
) -> FuncAuthoringResult<Func> {
    let name = name.unwrap_or(generate_name());
    if Func::find_id_by_name(ctx, &name).await?.is_some() {
        return Err(FuncAuthoringError::FuncNameExists(name));
    }

    let code_base64 = general_purpose::STANDARD_NO_PAD.encode(code);

    let func = Func::new(
        ctx,
        name,
        None::<String>,
        None::<String>,
        None::<String>,
        false,
        false,
        backend_kind,
        backend_response_type,
        Some(handler),
        Some(code_base64),
    )
    .await?;

    Ok(func)
}
