use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::func::FuncError;
use axum::Json;
use dal::{
    generate_name, job::definition::DependentValuesUpdate, prototype_context::HasPrototypeContext,
    qualification_prototype::QualificationPrototypeContext, AttributeValue, AttributeValueId,
    CodeGenerationPrototype, CodeLanguage, ComponentId, ConfirmationPrototype, DalContext, Func,
    FuncBackendKind, FuncBackendResponseType, FuncBindingReturnValue, FuncId,
    QualificationPrototype, SchemaId, SchemaVariantId, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum CreateFuncOptions {
    #[serde(rename_all = "camelCase")]
    AttributeOptions {
        value_id: Option<AttributeValueId>,
        parent_value_id: Option<AttributeValueId>,
        component_id: Option<ComponentId>,
        schema_variant_id: Option<SchemaVariantId>,
        schema_id: Option<SchemaId>,
        current_func_id: Option<FuncId>,
    },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFuncRequest {
    kind: FuncBackendKind,
    options: Option<CreateFuncOptions>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFuncResponse {
    pub id: FuncId,
    pub handler: Option<String>,
    pub kind: FuncBackendKind,
    pub name: String,
    pub code: Option<String>,
    pub schema_variants: Vec<SchemaVariantId>,
}

pub static ATTRIBUTE_CODE_HANDLER_PLACEHOLDER: &str = "HANDLER";
pub static ATTRIBUTE_CODE_DEFAULT_HANDLER: &str = "attribute";
pub static ATTRIBUTE_CODE_RETURN_VALUE_PLACEHOLDER: &str = "FUNCTION_RETURN_VALUE";
pub static DEFAULT_ATTRIBUTE_CODE_TEMPLATE: &str = include_str!("./defaults/attribute_template.ts");
pub static DEFAULT_CODE_GENERATION_HANDLER: &str = "generateCode";
pub static DEFAULT_CODE_GENERATION_FORMAT: CodeLanguage = CodeLanguage::Json;
pub static DEFAULT_CODE_GENERATION_CODE: &str = include_str!("./defaults/code_generation.ts");
pub static DEFAULT_QUALIFICATION_HANDLER: &str = "qualification";
pub static DEFAULT_QUALIFICATION_CODE: &str = include_str!("./defaults/qualification.ts");
pub static DEFAULT_CONFIRMATION_HANDLER: &str = "confirm";
pub static DEFAULT_CONFIRMATION_CODE: &str = include_str!("./defaults/confirmation.ts");
pub static DEFAULT_COMMAND_HANDLER: &str = "command";
pub static DEFAULT_COMMAND_CODE: &str = include_str!("./defaults/command.ts");
pub static DEFAULT_VALIDATION_HANDLER: &str = "validate";
pub static DEFAULT_VALIDATION_CODE: &str = include_str!("./defaults/validation.ts");

async fn create_qualification_func(ctx: &DalContext) -> FuncResult<Func> {
    let mut func = Func::new(
        ctx,
        generate_name(),
        FuncBackendKind::JsQualification,
        FuncBackendResponseType::Qualification,
    )
    .await?;

    func.set_code_plaintext(ctx, Some(DEFAULT_QUALIFICATION_CODE))
        .await?;
    func.set_handler(ctx, Some(DEFAULT_QUALIFICATION_HANDLER))
        .await?;

    let _ =
        QualificationPrototype::new(ctx, *func.id(), QualificationPrototypeContext::new()).await?;

    Ok(func)
}

async fn create_validation_func(ctx: &DalContext) -> FuncResult<Func> {
    let mut func = Func::new(
        ctx,
        generate_name(),
        FuncBackendKind::JsValidation,
        FuncBackendResponseType::Validation,
    )
    .await?;

    func.set_code_plaintext(ctx, Some(DEFAULT_VALIDATION_CODE))
        .await?;
    func.set_handler(ctx, Some(DEFAULT_VALIDATION_HANDLER))
        .await?;

    Ok(func)
}

async fn create_command_func(ctx: &DalContext) -> FuncResult<Func> {
    let mut func = Func::new(
        ctx,
        generate_name(),
        FuncBackendKind::JsCommand,
        FuncBackendResponseType::Command,
    )
    .await?;

    func.set_code_plaintext(ctx, Some(DEFAULT_COMMAND_CODE))
        .await?;
    func.set_handler(ctx, Some(DEFAULT_COMMAND_HANDLER)).await?;

    Ok(func)
}

async fn copy_attribute_func(ctx: &DalContext, func_to_copy: &Func) -> FuncResult<Func> {
    let mut func = Func::new(
        ctx,
        generate_name(),
        FuncBackendKind::JsAttribute,
        *func_to_copy.backend_response_type(),
    )
    .await?;

    func.set_handler(ctx, func_to_copy.handler()).await?;
    func.set_display_name(ctx, func_to_copy.display_name())
        .await?;
    func.set_code_base64(ctx, func_to_copy.code_base64())
        .await?;

    Ok(func)
}

async fn create_confirmation_func(ctx: &DalContext) -> FuncResult<Func> {
    let mut func = Func::new(
        ctx,
        generate_name(),
        FuncBackendKind::JsConfirmation,
        FuncBackendResponseType::Confirmation,
    )
    .await?;

    func.set_code_plaintext(ctx, Some(DEFAULT_CONFIRMATION_CODE))
        .await?;
    func.set_handler(ctx, Some(DEFAULT_CONFIRMATION_HANDLER))
        .await?;

    ConfirmationPrototype::new(
        ctx,
        func.display_name().unwrap_or("unknown"),
        *func.id(),
        ConfirmationPrototype::new_context(),
    )
    .await?;

    Ok(func)
}

async fn create_code_gen_func(ctx: &DalContext) -> FuncResult<Func> {
    let mut func = Func::new(
        ctx,
        generate_name(),
        FuncBackendKind::JsCodeGeneration,
        FuncBackendResponseType::CodeGeneration,
    )
    .await?;

    func.set_code_plaintext(ctx, Some(DEFAULT_CODE_GENERATION_CODE))
        .await?;
    func.set_handler(ctx, Some(DEFAULT_CODE_GENERATION_HANDLER))
        .await?;

    CodeGenerationPrototype::new_temporary(ctx, *func.id(), None).await?;

    Ok(func)
}

async fn create_default_attribute_func(
    ctx: &DalContext,
    value_id: Option<AttributeValueId>,
    current_func: Option<&Func>,
) -> FuncResult<Func> {
    let default_code = DEFAULT_ATTRIBUTE_CODE_TEMPLATE.replace(
        ATTRIBUTE_CODE_HANDLER_PLACEHOLDER,
        ATTRIBUTE_CODE_DEFAULT_HANDLER,
    );

    // if we were given an existing AttributeValue, generate a function with that value as the
    // default return value
    let default_code = if let Some(current_value_id) = value_id {
        let current_value = AttributeValue::get_by_id(ctx, &current_value_id)
            .await?
            .ok_or(FuncError::AttributeValueMissing)?;

        let fbrv_id = current_value.func_binding_return_value_id();
        let fbrv = FuncBindingReturnValue::get_by_id(ctx, &fbrv_id)
            .await?
            .ok_or(FuncError::FuncBindingReturnValueMissing)?;

        let current_value_value = fbrv.unprocessed_value();

        default_code.replace(
            ATTRIBUTE_CODE_RETURN_VALUE_PLACEHOLDER,
            &serde_json::to_string_pretty(&current_value_value)?,
        )
    } else {
        default_code.replace(ATTRIBUTE_CODE_RETURN_VALUE_PLACEHOLDER, "null")
    };

    let backend_response_type = match current_func {
        Some(func) => *func.backend_response_type(),
        None => FuncBackendResponseType::Unset,
    };

    let mut func = Func::new(
        ctx,
        generate_name(),
        FuncBackendKind::JsAttribute,
        backend_response_type,
    )
    .await?;

    func.set_code_plaintext(ctx, Some(&default_code)).await?;
    func.set_handler(ctx, Some(ATTRIBUTE_CODE_DEFAULT_HANDLER.to_owned()))
        .await?;

    Ok(func)
}

async fn create_attribute_func(
    ctx: &DalContext,
    value_id: Option<AttributeValueId>,
    current_func_id: Option<FuncId>,
) -> FuncResult<Func> {
    let current_func = match current_func_id {
        Some(current_func_id) => Some(
            Func::get_by_id(ctx, &current_func_id)
                .await?
                .ok_or(FuncError::FuncNotFound)?,
        ),
        None => None,
    };

    let should_copy_existing = match current_func {
        Some(ref current_func) => {
            if let FuncBackendKind::JsAttribute = current_func.backend_kind() {
                current_func.is_builtin()
            } else {
                false
            }
        }
        None => false,
    };

    let func = if should_copy_existing {
        // expect is safe from panic here, should_copy_existing will only ever be true if current_func is Some()
        copy_attribute_func(ctx, current_func.as_ref().expect("current_func was None")).await?
    } else {
        create_default_attribute_func(ctx, value_id, current_func.as_ref()).await?
    };

    // If we were given a value, update that value with the new function
    if let Some(value_id) = value_id {
        let mut value = AttributeValue::get_by_id(ctx, &value_id)
            .await?
            .ok_or(FuncError::AttributeValueMissing)?;
        value.update_from_prototype_function(ctx).await?;
        ctx.enqueue_job(DependentValuesUpdate::new(ctx, value_id))
            .await;
    }

    Ok(func)
}

pub async fn create_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CreateFuncRequest>,
) -> FuncResult<Json<CreateFuncResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let func = match request.kind {
        FuncBackendKind::JsAttribute => match request.options {
            Some(CreateFuncOptions::AttributeOptions {
                value_id,
                current_func_id,
                ..
            }) => create_attribute_func(&ctx, value_id, current_func_id).await?,
            None => create_attribute_func(&ctx, None, None).await?,
        },
        FuncBackendKind::JsQualification => create_qualification_func(&ctx).await?,
        FuncBackendKind::JsCodeGeneration => create_code_gen_func(&ctx).await?,
        FuncBackendKind::JsConfirmation => create_confirmation_func(&ctx).await?,
        FuncBackendKind::JsCommand => create_command_func(&ctx).await?,
        FuncBackendKind::JsValidation => create_validation_func(&ctx).await?,
        _ => Err(FuncError::FuncNotSupported)?,
    };

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;
    ctx.commit().await?;

    Ok(Json(CreateFuncResponse {
        id: func.id().to_owned(),
        handler: func.handler().map(|h| h.to_owned()),
        kind: func.backend_kind().to_owned(),
        name: func.name().to_owned(),
        code: func.code_plaintext()?,
        schema_variants: vec![],
    }))
}
