use super::{FuncResult, FuncVariant};
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::func::FuncError;
use axum::extract::OriginalUri;
use axum::Json;
use dal::{
    generate_name, validation::prototype::context::ValidationPrototypeContext, ActionKind,
    ActionPrototype, ActionPrototypeContext, AttributeContextBuilder, AttributePrototype,
    DalContext, ExternalProviderId, Func, FuncBackendResponseType, FuncId, LeafInputLocation,
    LeafKind, PropId, SchemaVariant, SchemaVariantId, StandardModel, ValidationPrototype,
    Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};

#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AttributeOutputLocation {
    #[serde(rename_all = "camelCase")]
    OutputSocket {
        external_provider_id: ExternalProviderId,
    },
    #[serde(rename_all = "camelCase")]
    Prop { prop_id: PropId },
}

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
    CodeGenerationOptions { schema_variant_id: SchemaVariantId },
    #[serde(rename_all = "camelCase")]
    QualificationOptions { schema_variant_id: SchemaVariantId },
    #[serde(rename_all = "camelCase")]
    ValidationOptions {
        schema_variant_id: SchemaVariantId,
        prop_to_validate: PropId,
    },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFuncRequest {
    variant: FuncVariant,
    name: Option<String>,
    options: Option<CreateFuncOptions>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFuncResponse {
    pub id: FuncId,
    pub handler: Option<String>,
    pub variant: FuncVariant,
    pub name: String,
    pub code: Option<String>,
}

pub static DEFAULT_ATTRIBUTE_CODE_HANDLER: &str = "setAttribute";
pub static DEFAULT_ATTRIBUTE_CODE: &str = include_str!("./defaults/attribute.ts");
pub static DEFAULT_CODE_GENERATION_HANDLER: &str = "generateCode";
pub static DEFAULT_CODE_GENERATION_CODE: &str = include_str!("./defaults/code_generation.ts");
pub static DEFAULT_QUALIFICATION_HANDLER: &str = "qualification";
pub static DEFAULT_QUALIFICATION_CODE: &str = include_str!("./defaults/qualification.ts");
pub static DEFAULT_ACTION_HANDLER: &str = "action";
pub static DEFAULT_ACTION_CODE: &str = include_str!("./defaults/action.ts");
pub static DEFAULT_VALIDATION_HANDLER: &str = "validate";
pub static DEFAULT_VALIDATION_CODE: &str = include_str!("./defaults/validation.ts");

async fn create_func_stub(
    ctx: &DalContext,
    name: Option<String>,
    variant: FuncVariant,
    response_type: FuncBackendResponseType,
    code: &str,
    handler: &str,
) -> FuncResult<Func> {
    let name = name.unwrap_or(generate_name());
    if Func::find_by_name(ctx, &name).await?.is_some() {
        return Err(FuncError::FuncNameExists(name));
    }

    let mut func = Func::new(ctx, name, variant.into(), response_type).await?;

    func.set_code_plaintext(ctx, Some(code)).await?;
    func.set_handler(ctx, Some(handler)).await?;

    Ok(func)
}

async fn create_validation_func(
    ctx: &DalContext,
    name: Option<String>,
    options: Option<CreateFuncOptions>,
) -> FuncResult<Func> {
    let func = create_func_stub(
        ctx,
        name,
        FuncVariant::Validation,
        FuncBackendResponseType::Validation,
        DEFAULT_VALIDATION_CODE,
        DEFAULT_VALIDATION_HANDLER,
    )
    .await?;

    if let Some(CreateFuncOptions::ValidationOptions {
        schema_variant_id,
        prop_to_validate,
    }) = options
    {
        let mut context = ValidationPrototypeContext::builder();
        let schema_id = *SchemaVariant::get_by_id(ctx, &schema_variant_id)
            .await?
            .ok_or(FuncError::ValidationPrototypeMissingSchemaVariant(
                schema_variant_id,
            ))?
            .schema(ctx)
            .await?
            .ok_or(FuncError::ValidationPrototypeMissingSchema)?
            .id();

        let context = context
            .set_prop_id(prop_to_validate)
            .set_schema_variant_id(schema_variant_id)
            .set_schema_id(schema_id)
            .to_context(ctx)
            .await?;

        // Can we have more than one validation per prop?
        if !ValidationPrototype::find_for_context(ctx, context.to_owned())
            .await?
            .is_empty()
        {
            return Err(FuncError::ValidationAlreadyExists);
        }

        ValidationPrototype::new(ctx, *func.id(), serde_json::json!(null), context).await?;
    }

    Ok(func)
}

async fn create_action_func(
    ctx: &DalContext,
    name: Option<String>,
    options: Option<CreateFuncOptions>,
) -> FuncResult<Func> {
    let func = create_func_stub(
        ctx,
        name,
        FuncVariant::Action,
        FuncBackendResponseType::Action,
        DEFAULT_ACTION_CODE,
        DEFAULT_ACTION_HANDLER,
    )
    .await?;

    if let Some(CreateFuncOptions::ActionOptions {
        schema_variant_id,
        action_kind,
    }) = options
    {
        ActionPrototype::new(
            ctx,
            *func.id(),
            action_kind,
            ActionPrototypeContext { schema_variant_id },
        )
        .await?;
    }

    Ok(func)
}

async fn create_leaf_prototype(
    ctx: &DalContext,
    func: &Func,
    schema_variant_id: SchemaVariantId,
    variant: FuncVariant,
) -> FuncResult<()> {
    let leaf_kind = match variant {
        FuncVariant::CodeGeneration => LeafKind::CodeGeneration,
        FuncVariant::Qualification => LeafKind::Qualification,
        _ => return Err(FuncError::FuncOptionsAndVariantMismatch),
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
    variant: FuncVariant,
    options: Option<CreateFuncOptions>,
) -> FuncResult<Func> {
    let (code, handler, response_type) = match variant {
        FuncVariant::Attribute => (
            DEFAULT_ATTRIBUTE_CODE,
            DEFAULT_ATTRIBUTE_CODE_HANDLER,
            FuncBackendResponseType::Unset,
        ),
        FuncVariant::CodeGeneration => (
            DEFAULT_CODE_GENERATION_CODE,
            DEFAULT_CODE_GENERATION_HANDLER,
            FuncBackendResponseType::CodeGeneration,
        ),
        FuncVariant::Qualification => (
            DEFAULT_QUALIFICATION_CODE,
            DEFAULT_QUALIFICATION_HANDLER,
            FuncBackendResponseType::Qualification,
        ),
        _ => {
            return Err(FuncError::UnexpectedFuncVariantCreatingAttributeFunc(
                variant.to_owned(),
            ));
        }
    };

    let func = create_func_stub(ctx, name, variant, response_type, code, handler).await?;

    if let Some(options) = options {
        match (variant, options) {
            (
                FuncVariant::Attribute,
                CreateFuncOptions::AttributeOptions {
                    output_location, ..
                },
            ) => {
                // XXX: we need to search *up* the attribute tree to ensure that
                // the parent of this prop is not also set by a function. But we
                // should also hide props on the frontend if they are the
                // children of a value that is set by a function.
                let mut context_builder = AttributeContextBuilder::new();
                match output_location {
                    AttributeOutputLocation::OutputSocket {
                        external_provider_id,
                    } => {
                        context_builder.set_external_provider_id(external_provider_id);
                    }
                    AttributeOutputLocation::Prop { prop_id } => {
                        context_builder.set_prop_id(prop_id);
                    }
                }

                let context = context_builder.to_context()?;
                let mut prototype =
                    AttributePrototype::find_for_context_and_key(ctx, context, &None)
                        .await?
                        .pop()
                        .ok_or(FuncError::AttributePrototypeMissing)?;

                if let Some(func) = Func::get_by_id(ctx, &prototype.func_id()).await? {
                    if !func.is_intrinsic() {
                        return Err(FuncError::AttributePrototypeAlreadySetByFunc(
                            func.name().into(),
                        ));
                    }
                }

                prototype.set_func_id(ctx, *func.id()).await?;
            }
            (
                FuncVariant::CodeGeneration,
                CreateFuncOptions::CodeGenerationOptions { schema_variant_id },
            ) => {
                create_leaf_prototype(ctx, &func, schema_variant_id, variant).await?;
            }
            (
                FuncVariant::Qualification,
                CreateFuncOptions::QualificationOptions { schema_variant_id },
            ) => {
                create_leaf_prototype(ctx, &func, schema_variant_id, variant).await?;
            }
            (_, _) => return Err(FuncError::FuncOptionsAndVariantMismatch),
        }
    }

    Ok(func)
}

pub async fn create_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<CreateFuncRequest>,
) -> FuncResult<Json<CreateFuncResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let func = match request.variant {
        FuncVariant::Attribute => {
            create_attribute_func(&ctx, request.name, FuncVariant::Attribute, request.options)
                .await?
        }
        FuncVariant::CodeGeneration => {
            create_attribute_func(
                &ctx,
                request.name,
                FuncVariant::CodeGeneration,
                request.options,
            )
            .await?
        }
        FuncVariant::Action => create_action_func(&ctx, request.name, request.options).await?,
        FuncVariant::Validation => {
            create_validation_func(&ctx, request.name, request.options).await?
        }
        FuncVariant::Qualification => {
            create_attribute_func(
                &ctx,
                request.name,
                FuncVariant::Qualification,
                request.options,
            )
            .await?
        }
        _ => unimplemented!(),
    };

    let func_variant = (&func).try_into()?;

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "created_func",
        serde_json::json!({
                    "func_id": func.id().to_owned(),
                    "func_handler": func.handler().map(|h| h.to_owned()),
                    "func_name": func.name().to_owned(),
                    "func_variant": func_variant,
        }),
    );

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;
    ctx.commit().await?;

    Ok(Json(CreateFuncResponse {
        id: func.id().to_owned(),
        handler: func.handler().map(|h| h.to_owned()),
        variant: func_variant,
        name: func.name().to_owned(),
        code: func.code_plaintext()?,
    }))
}
