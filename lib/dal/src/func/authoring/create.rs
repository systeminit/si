//! This module contains [`CreatedFunc`], which provides the ability to create [`Funcs`](Func) in
//! the [`Func`] authoring experience.

use base64::engine::general_purpose;
use base64::Engine;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::func::authoring::{FuncAuthoringError, FuncAuthoringResult};
use crate::func::FuncKind;
use crate::schema::variant::leaves::{LeafInputLocation, LeafKind};
use crate::{
    generate_name, ActionKind, AttributePrototype, DalContext, Func, FuncBackendKind,
    FuncBackendResponseType, FuncId, OutputSocketId, PropId, SchemaVariant, SchemaVariantId,
};

static DEFAULT_CODE_HANDLER: &str = "main";
static DEFAULT_ATTRIBUTE_CODE: &str = include_str!("data/defaults/attribute.ts");
static DEFAULT_CODE_GENERATION_CODE: &str = include_str!("data/defaults/code_generation.ts");
static DEFAULT_QUALIFICATION_CODE: &str = include_str!("data/defaults/qualification.ts");
static DEFAULT_ACTION_CODE: &str = include_str!("data/defaults/action.ts");
static DEFAULT_AUTHENTICATION_CODE: &str = include_str!("data/defaults/authentication.ts");

#[allow(dead_code)]
static DEFAULT_VALIDATION_CODE: &str = include_str!("data/defaults/validation.ts");

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AttributeOutputLocation {
    #[serde(rename_all = "camelCase")]
    OutputSocket { output_socket_id: OutputSocketId },
    #[serde(rename_all = "camelCase")]
    Prop { prop_id: PropId },
}

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CreateFuncOptions {
    #[serde(rename_all = "camelCase")]
    ActionOptions {
        schema_variant_id: SchemaVariantId,
        action_kind: ActionKind,
    },
    #[serde(rename_all = "camelCase")]
    AttributeOptions {
        schema_variant_id: SchemaVariantId,
        output_location: AttributeOutputLocation,
    },
    #[serde(rename_all = "camelCase")]
    AuthenticationOptions { schema_variant_id: SchemaVariantId },
    #[serde(rename_all = "camelCase")]
    CodeGenerationOptions { schema_variant_id: SchemaVariantId },
    #[serde(rename_all = "camelCase")]
    QualificationOptions { schema_variant_id: SchemaVariantId },
}

/// The result of creating a [`Func`] via [`CreatedFunc::run`].
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreatedFunc {
    /// The id of the created [`Func`].
    pub id: FuncId,
    /// The handler of the created [`Func`].
    pub handler: Option<String>,
    /// The [kind](FuncKind) of the created [`Func`].
    pub kind: FuncKind,
    /// The name of the created [`Func`].
    pub name: String,
    /// The code for the created [`Func`].
    pub code: Option<String>,
}

/// Creates a [`Func`] and returns the [result](CreatedFunc).
#[instrument(name = "func.authoring.create_func", level = "info", skip_all)]
pub async fn create_func(
    ctx: &DalContext,
    kind: FuncKind,
    name: Option<String>,
    options: Option<CreateFuncOptions>,
) -> FuncAuthoringResult<CreatedFunc> {
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

    Ok(CreatedFunc {
        id: func.id,
        handler: func.handler.as_ref().map(|h| h.to_owned()),
        kind: func.kind,
        name: func.name.to_owned(),
        code: func.code_plaintext()?,
    })
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

async fn create_action_func(
    ctx: &DalContext,
    name: Option<String>,
    _options: Option<CreateFuncOptions>,
) -> FuncAuthoringResult<Func> {
    let func = create_func_stub(
        ctx,
        name,
        FuncBackendKind::JsAction,
        FuncBackendResponseType::Action,
        DEFAULT_ACTION_CODE,
        DEFAULT_CODE_HANDLER,
    )
    .await?;

    //    if let Some(CreateFuncOptions::ActionOptions {
    //        schema_variant_id,
    //        action_kind,
    //    }) = options
    //    {
    //        ActionPrototype::new(
    //            ctx,
    //            *func.id(),
    //            action_kind,
    //            ActionPrototypeContext { schema_variant_id },
    //        )
    //        .await?;
    //    }

    Ok(func)
}

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
                    let func_id = AttributePrototype::func_id(ctx, ap).await?;
                    if let Some(func) = Func::get_by_id(ctx, func_id).await? {
                        if !Func::is_dynamic_for_name_string(func.name.as_str()) {
                            return Err(FuncAuthoringError::AttributePrototypeAlreadySetByFunc(
                                kind.to_string(),
                            ));
                        }
                    }

                    AttributePrototype::update_func_by_id(ctx, ap, func_id).await?;
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
