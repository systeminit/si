use base64::engine::general_purpose;
use base64::Engine;
use telemetry::prelude::*;

use crate::action::prototype::ActionPrototype;
use crate::func::authoring::{
    AttributeOutputLocation, CreateFuncOptions, FuncAuthoringError, FuncAuthoringResult,
};
use crate::func::FuncKind;
use crate::schema::variant::leaves::{LeafInputLocation, LeafKind};
use crate::{
    generate_name, AttributePrototype, DalContext, Func, FuncBackendKind, FuncBackendResponseType,
    SchemaVariant, SchemaVariantId,
};

static DEFAULT_CODE_HANDLER: &str = "main";
static DEFAULT_ATTRIBUTE_CODE: &str = include_str!("data/defaults/attribute.ts");
static DEFAULT_CODE_GENERATION_CODE: &str = include_str!("data/defaults/code_generation.ts");
static DEFAULT_QUALIFICATION_CODE: &str = include_str!("data/defaults/qualification.ts");
static DEFAULT_ACTION_CODE: &str = include_str!("data/defaults/action.ts");
static DEFAULT_AUTHENTICATION_CODE: &str = include_str!("data/defaults/authentication.ts");

#[allow(dead_code)]
static DEFAULT_VALIDATION_CODE: &str = include_str!("data/defaults/validation.ts");

#[instrument(name = "func.authoring.create_func.create", level = "debug", skip(ctx))]
pub(crate) async fn create(
    ctx: &DalContext,
    kind: FuncKind,
    name: Option<String>,
    options: Option<CreateFuncOptions>,
) -> FuncAuthoringResult<Func> {
    let func = match kind {
        FuncKind::Action => create_action_func(ctx, name, options).await?,
        FuncKind::Attribute => {
            create_attribute_func(ctx, name, FuncKind::Attribute, options).await?
        }
        FuncKind::Authentication => create_authentication_func(ctx, name, options).await?,
        FuncKind::CodeGeneration => {
            create_attribute_func(ctx, name, FuncKind::CodeGeneration, options).await?
        }
        FuncKind::Qualification => {
            create_attribute_func(ctx, name, FuncKind::Qualification, options).await?
        }
        kind => return Err(FuncAuthoringError::InvalidFuncKindForCreation(kind)),
    };

    Ok(func)
}

#[instrument(
    name = "func.authoring.create_func.create.action",
    level = "debug",
    skip(ctx)
)]
async fn create_action_func(
    ctx: &DalContext,
    name: Option<String>,
    options: Option<CreateFuncOptions>,
) -> FuncAuthoringResult<Func> {
    // need to see if there's already an action func of this particular kind for the schema variant
    // before doing anything else
    if let Some(CreateFuncOptions::ActionOptions {
        schema_variant_id,
        action_kind,
    }) = options
    {
        let exising_actions = ActionPrototype::for_variant(ctx, schema_variant_id).await?;
        for action in exising_actions {
            if action.kind == action_kind {
                return Err(FuncAuthoringError::ActionKindAlreadyExists(
                    action_kind,
                    schema_variant_id,
                ));
            }
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

    if let Some(CreateFuncOptions::ActionOptions {
        schema_variant_id,
        action_kind,
    }) = options
    {
        // default to func name if the name is missing for whatever reason...
        ActionPrototype::new(
            ctx,
            action_kind,
            name.clone().unwrap_or(func.name.clone()),
            None,
            schema_variant_id,
            func.id,
        )
        .await?;
    }

    Ok(func)
}

#[instrument(
    name = "func.authoring.create_func.create.attribute",
    level = "debug",
    skip(ctx)
)]
async fn create_attribute_func(
    ctx: &DalContext,
    name: Option<String>,
    kind: FuncKind,
    options: Option<CreateFuncOptions>,
) -> FuncAuthoringResult<Func> {
    let (code, handler, backend_kind, backend_response_type) = match kind {
        FuncKind::Attribute => (
            DEFAULT_ATTRIBUTE_CODE,
            DEFAULT_CODE_HANDLER,
            FuncBackendKind::JsAttribute,
            FuncBackendResponseType::Unset,
        ),
        FuncKind::CodeGeneration => (
            DEFAULT_CODE_GENERATION_CODE,
            DEFAULT_CODE_HANDLER,
            FuncBackendKind::JsAttribute,
            FuncBackendResponseType::CodeGeneration,
        ),
        FuncKind::Qualification => (
            DEFAULT_QUALIFICATION_CODE,
            DEFAULT_CODE_HANDLER,
            FuncBackendKind::JsAttribute,
            FuncBackendResponseType::Qualification,
        ),
        _ => {
            return Err(FuncAuthoringError::UnexpectedFuncKindCreatingAttributeFunc(
                kind,
            ));
        }
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

    if let Some(options) = options {
        match (kind, options) {
            (
                FuncKind::Attribute,
                CreateFuncOptions::AttributeOptions {
                    output_location, ..
                },
            ) => {
                if let Some(ap) = match output_location {
                    AttributeOutputLocation::OutputSocket { output_socket_id } => {
                        AttributePrototype::find_for_output_socket(ctx, output_socket_id).await?
                    }
                    AttributeOutputLocation::Prop { prop_id } => {
                        AttributePrototype::find_for_prop(ctx, prop_id, &None).await?
                    }
                } {
                    // TODO - Paul / Nick - we need to ensure this code is working as expected
                    // right now, we don't allow overiding identity
                    // See create_attribute_with_socket and create_attribute_with_prop tests for
                    // examples of where this breaks
                    let func_id = AttributePrototype::func_id(ctx, ap).await?;
                    if let Some(func) = Func::get_by_id(ctx, func_id).await? {
                        if Func::is_dynamic_for_name_string(func.name.as_str()) {
                            return Err(FuncAuthoringError::AttributePrototypeAlreadySetByFunc(
                                func_id, func.name,
                            ));
                        }
                    }

                    AttributePrototype::update_func_by_id(ctx, ap, func.id).await?;
                }
            }
            (
                FuncKind::CodeGeneration,
                CreateFuncOptions::CodeGenerationOptions { schema_variant_id },
            ) => {
                create_leaf_prototype(ctx, &func, schema_variant_id, kind).await?;
            }
            (
                FuncKind::Qualification,
                CreateFuncOptions::QualificationOptions { schema_variant_id },
            ) => {
                create_leaf_prototype(ctx, &func, schema_variant_id, kind).await?;
            }
            (_, _) => return Err(FuncAuthoringError::FuncOptionsAndVariantMismatch),
        }
    }

    Ok(func)
}

#[instrument(
    name = "func.authoring.create_func.create.authentication",
    level = "debug",
    skip(ctx)
)]
async fn create_authentication_func(
    ctx: &DalContext,
    name: Option<String>,
    options: Option<CreateFuncOptions>,
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

    if let Some(CreateFuncOptions::AuthenticationOptions { schema_variant_id }) = options {
        SchemaVariant::new_authentication_prototype(ctx, func.id, schema_variant_id).await?;
    }

    Ok(func)
}

#[instrument(
    name = "func.authoring.create_func.create.attribute.leaf",
    level = "debug",
    skip(ctx)
)]
async fn create_leaf_prototype(
    ctx: &DalContext,
    func: &Func,
    schema_variant_id: SchemaVariantId,
    kind: FuncKind,
) -> FuncAuthoringResult<()> {
    let leaf_kind = match kind {
        FuncKind::CodeGeneration => LeafKind::CodeGeneration,
        FuncKind::Qualification => LeafKind::Qualification,
        _ => return Err(FuncAuthoringError::FuncOptionsAndVariantMismatch),
    };

    let input_locations = match leaf_kind {
        LeafKind::CodeGeneration => vec![LeafInputLocation::Domain],
        LeafKind::Qualification => vec![LeafInputLocation::Domain, LeafInputLocation::Code],
    };

    SchemaVariant::upsert_leaf_function(
        ctx,
        schema_variant_id,
        None,
        leaf_kind,
        &input_locations,
        func,
    )
    .await?;

    Ok(())
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
    if Func::find_by_name(ctx, &name).await?.is_some() {
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
