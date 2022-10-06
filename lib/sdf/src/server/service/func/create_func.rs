use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::func::FuncError;
use axum::Json;
use dal::func::backend::js_code_generation::FuncBackendJsCodeGenerationArgs;
use dal::{
    attribute::context::AttributeContextBuilder, generate_name,
    prototype_context::HasPrototypeContext, qualification_prototype::QualificationPrototypeContext,
    AttributeValue, AttributeValueId, CodeGenerationPrototype, CodeLanguage, ComponentId,
    DalContext, Func, FuncBackendKind, FuncBackendResponseType, FuncBindingReturnValue, FuncId,
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

static ATTRIBUTE_CODE_HANDLER_PLACEHOLDER: &str = "HANDLER";
static ATTRIBUTE_CODE_DEFAULT_HANDLER: &str = "attribute";
static ATTRIBUTE_CODE_RETURN_VALUE_PLACEHOLDER: &str = "FUNCTION_RETURN_VALUE";

const DEFAULT_CODE_GENERATION_FORMAT: CodeLanguage = CodeLanguage::Json;
static DEFAULT_CODE_GENERATION_CODE: &str = "
function generateCode(component) {
    return {
        format: \"json\",
        code: JSON.stringify(component.properties),
    };
}
";

static DEFAULT_ATTRIBUTE_CODE_TEMPLATE: &str = "/*
*/
function HANDLER(component) {
    return FUNCTION_RETURN_VALUE;
}
";

pub static DEFAULT_QUALIFICATION_CODE: &str = "/*
* Your qualification function
* The signature should never be changed
*
* The input type is `Component`
* The return type is `Qualification`
*
* interface System {
*   name: string;
* }
*
* // The properties are derived from the fields in the Attributes panel
* interface Properties {
*   si: unknown;
*   domain: unknown
* }
*
* enum Kind {
*   Standard,
*   Credential
* }
*
* interface Data {
*   system: System | null;
*   kind: Kind;
*   properties: Properties;
* }
*
* interface Code {
*   format: string;
*   code: string | null;
* }
*
* interface Component {
*   data: Data;
*   parents: Component[]; // The parent's parents won't be available
*   codes: Code[];
* }
*
* interface Qualification {
*   qualified: boolean;
*   message: string;
* }
*/
async function qualification(component) {
  return {
    qualified: true,
    message: 'Component qualified'
  };
}";

async fn create_qualification_func(ctx: &DalContext) -> FuncResult<Func> {
    let mut func = Func::new(
        ctx,
        generate_name(None),
        FuncBackendKind::JsQualification,
        FuncBackendResponseType::Qualification,
    )
    .await?;

    func.set_code_plaintext(ctx, Some(DEFAULT_QUALIFICATION_CODE))
        .await?;
    func.set_handler(ctx, Some("qualification".to_owned()))
        .await?;

    let _ =
        QualificationPrototype::new(ctx, *func.id(), QualificationPrototypeContext::new()).await?;

    Ok(func)
}

async fn copy_attribute_func(ctx: &DalContext, func_to_copy: &Func) -> FuncResult<Func> {
    let mut func = Func::new(
        ctx,
        generate_name(None),
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

async fn create_code_gen_func(ctx: &DalContext) -> FuncResult<Func> {
    let mut func = Func::new(
        ctx,
        generate_name(None),
        FuncBackendKind::JsCodeGeneration,
        FuncBackendResponseType::CodeGeneration,
    )
    .await?;

    func.set_code_plaintext(ctx, Some(DEFAULT_CODE_GENERATION_CODE))
        .await?;
    func.set_handler(ctx, Some("generateCode".to_owned()))
        .await?;

    let code_gen_args = FuncBackendJsCodeGenerationArgs::default();
    let code_gen_args_json = serde_json::to_value(&code_gen_args)?;

    CodeGenerationPrototype::new(
        ctx,
        *func.id(),
        code_gen_args_json,
        DEFAULT_CODE_GENERATION_FORMAT,
        CodeGenerationPrototype::new_context(),
    )
    .await?;

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
        generate_name(None),
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
    parent_value_id: Option<AttributeValueId>,
    component_id: Option<ComponentId>,
    schema_variant_id: Option<SchemaVariantId>,
    schema_id: Option<SchemaId>,
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
        let prop = AttributeValue::find_prop_for_value(ctx, value_id).await?;
        let context = AttributeContextBuilder::new()
            .set_prop_id(*prop.id())
            .set_component_id(component_id.into())
            .set_schema_variant_id(schema_variant_id.into())
            .set_schema_id(schema_id.into())
            .to_context()?;

        super::update_attribute_value_by_func_for_context(
            ctx,
            value_id,
            parent_value_id,
            &func,
            context,
            true,
        )
        .await?;
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
                parent_value_id,
                component_id,
                schema_variant_id,
                schema_id,
                current_func_id,
            }) => {
                create_attribute_func(
                    &ctx,
                    value_id,
                    parent_value_id,
                    component_id,
                    schema_variant_id,
                    schema_id,
                    current_func_id,
                )
                .await?
            }
            None => create_attribute_func(&ctx, None, None, None, None, None, None).await?,
        },
        FuncBackendKind::JsQualification => create_qualification_func(&ctx).await?,
        FuncBackendKind::JsCodeGeneration => create_code_gen_func(&ctx).await?,
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
