use axum::extract::OriginalUri;
use axum::{response::IntoResponse, Json};
use base64::engine::general_purpose;
use base64::Engine;
use dal::func::FuncKind;
use dal::{
    generate_name, ActionKind, ChangeSet, DalContext, Func, FuncBackendKind,
    FuncBackendResponseType, FuncId, OutputSocketId, PropId, SchemaVariant, SchemaVariantId,
    Visibility,
};
use serde::{Deserialize, Serialize};

use super::FuncResult;
use crate::server::extract::{AccessBuilder, HandlerContext, PosthogClient};
use crate::server::tracking::track;
use crate::service::func::FuncError;

#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum AttributeOutputLocation {
    #[serde(rename_all = "camelCase")]
    OutputSocket { output_socket_id: OutputSocketId },
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
    AuthenticationOptions { schema_variant_id: SchemaVariantId },
    #[serde(rename_all = "camelCase")]
    CodeGenerationOptions { schema_variant_id: SchemaVariantId },
    #[serde(rename_all = "camelCase")]
    QualificationOptions { schema_variant_id: SchemaVariantId },
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CreateFuncRequest {
    kind: FuncKind,
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
    pub kind: FuncKind,
    pub name: String,
    pub code: Option<String>,
}

pub static DEFAULT_CODE_HANDLER: &str = "main";
pub static DEFAULT_ATTRIBUTE_CODE: &str = include_str!("./defaults/attribute.ts");
pub static DEFAULT_CODE_GENERATION_CODE: &str = include_str!("./defaults/code_generation.ts");
pub static DEFAULT_QUALIFICATION_CODE: &str = include_str!("./defaults/qualification.ts");
pub static DEFAULT_ACTION_CODE: &str = include_str!("./defaults/action.ts");
pub static DEFAULT_VALIDATION_CODE: &str = include_str!("./defaults/validation.ts");
pub static DEFAULT_AUTHENTICATION_CODE: &str = include_str!("./defaults/authentication.ts");

async fn create_func_stub(
    ctx: &DalContext,
    name: Option<String>,
    backend_kind: FuncBackendKind,
    backend_response_type: FuncBackendResponseType,
    code: &str,
    handler: &str,
) -> FuncResult<Func> {
    let name = name.unwrap_or(generate_name());
    if Func::find_by_name(ctx, &name).await?.is_some() {
        return Err(FuncError::FuncNameExists(name));
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
) -> FuncResult<Func> {
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

//async fn create_leaf_prototype(
//    ctx: &DalContext,
//    func: &Func,
//    schema_variant_id: SchemaVariantId,
//    variant: FuncVariant,
//) -> FuncResult<()> {
//    let leaf_kind = match variant {
//        FuncVariant::CodeGeneration => LeafKind::CodeGeneration,
//        FuncVariant::Qualification => LeafKind::Qualification,
//        _ => return Err(FuncError::FuncOptionsAndVariantMismatch),
//    };
//
//    let input_locations = match leaf_kind {
//        LeafKind::CodeGeneration => vec![LeafInputLocation::Domain],
//        LeafKind::Qualification => vec![LeafInputLocation::Domain, LeafInputLocation::Code],
//    };
//
//    SchemaVariant::upsert_leaf_function(
//        ctx,
//        schema_variant_id,
//        None,
//        leaf_kind,
//        &input_locations,
//        func,
//    )
//    .await?;
//
//    Ok(())
//}

async fn create_attribute_func(
    ctx: &DalContext,
    name: Option<String>,
    kind: FuncKind,
    _options: Option<CreateFuncOptions>,
) -> FuncResult<Func> {
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
            return Err(FuncError::UnexpectedFuncKindCreatingAttributeFunc(kind));
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

    // if let Some(options) = options {
    //     match (variant, options) {
    //         (
    //             FuncVariant::Attribute,
    //             CreateFuncOptions::AttributeOptions {
    //                 output_location, ..
    //             },
    //         ) => {
    //             // XXX: we need to search *up* the attribute tree to ensure that
    //             // the parent of this prop is not also set by a function. But we
    //             // should also hide props on the frontend if they are the
    //             // children of a value that is set by a function.
    //             let mut context_builder = AttributeContextBuilder::new();
    //             match output_location {
    //                 AttributeOutputLocation::OutputSocket {
    //                     external_provider_id,
    //                 } => {
    //                     context_builder.set_external_provider_id(external_provider_id);
    //                 }
    //                 AttributeOutputLocation::Prop { prop_id } => {
    //                     context_builder.set_prop_id(prop_id);
    //                 }
    //             }

    //             let context = context_builder.to_context()?;
    //             let mut prototype =
    //                 AttributePrototype::find_for_context_and_key(ctx, context, &None)
    //                     .await?
    //                     .pop()
    //                     .ok_or(FuncError::AttributePrototypeMissing)?;

    //             if let Some(func) = Func::get_by_id(ctx, &prototype.func_id()).await? {
    //                 if !func.is_intrinsic() {
    //                     return Err(FuncError::AttributePrototypeAlreadySetByFunc(
    //                         func.name().into(),
    //                     ));
    //                 }
    //             }

    //             prototype.set_func_id(ctx, *func.id()).await?;
    //         }
    //         (
    //             FuncVariant::CodeGeneration,
    //             CreateFuncOptions::CodeGenerationOptions { schema_variant_id },
    //         ) => {
    //             create_leaf_prototype(ctx, &func, schema_variant_id, variant).await?;
    //         }
    //         (
    //             FuncVariant::Qualification,
    //             CreateFuncOptions::QualificationOptions { schema_variant_id },
    //         ) => {
    //             create_leaf_prototype(ctx, &func, schema_variant_id, variant).await?;
    //         }
    //         (_, _) => return Err(FuncError::FuncOptionsAndVariantMismatch),
    //     }
    // }

    Ok(func)
}

async fn create_authentication_func(
    ctx: &DalContext,
    name: Option<String>,
    options: Option<CreateFuncOptions>,
) -> FuncResult<Func> {
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

pub async fn create_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    PosthogClient(posthog_client): PosthogClient,
    OriginalUri(original_uri): OriginalUri,
    Json(request): Json<CreateFuncRequest>,
) -> FuncResult<impl IntoResponse> {
    let mut ctx = builder.build(request_ctx.build(request.visibility)).await?;

    if let Some(name) = request.name.as_deref() {
        if dal::func::is_intrinsic(name)
            || ["si:resourcePayloadToValue", "si:normalizeToArray"].contains(&name)
        {
            return Err(FuncError::FuncNameReserved(name.into()));
        }
    }

    let force_change_set_id = ChangeSet::force_new(&mut ctx).await?;

    let func = match request.kind {
        FuncKind::Action => create_action_func(&ctx, request.name, request.options).await?,
        FuncKind::Attribute => {
            create_attribute_func(&ctx, request.name, FuncKind::Attribute, request.options).await?
        }
        FuncKind::Authentication => {
            create_authentication_func(&ctx, request.name, request.options).await?
        }
        FuncKind::CodeGeneration => {
            create_attribute_func(
                &ctx,
                request.name,
                FuncKind::CodeGeneration,
                request.options,
            )
            .await?
        }
        FuncKind::Qualification => {
            create_attribute_func(&ctx, request.name, FuncKind::Qualification, request.options)
                .await?
        }
        kind => return Err(FuncError::InvalidFuncKindForCreation(kind)),
    };

    track(
        &posthog_client,
        &ctx,
        &original_uri,
        "created_func",
        serde_json::json!({
                    "func_id": func.id,
                    "func_handler": func.handler.as_ref().map(|h| h.to_owned()),
                    "func_name": func.name.to_owned(),
                    "func_kind": func.kind,
        }),
    );

    ctx.commit().await?;

    let mut response = axum::response::Response::builder();
    response = response.header("Content-Type", "application/json");
    if let Some(force_change_set_id) = force_change_set_id {
        response = response.header("force_change_set_id", force_change_set_id.to_string());
    }
    Ok(response.body(serde_json::to_string(&CreateFuncResponse {
        id: func.id,
        handler: func.handler.as_ref().map(|h| h.to_owned()),
        kind: func.kind,
        name: func.name.to_owned(),
        code: func.code_plaintext()?,
    })?)?)
}
