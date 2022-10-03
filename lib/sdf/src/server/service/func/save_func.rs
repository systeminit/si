use super::{
    AttributePrototypeArgumentView, AttributePrototypeView, FuncArgumentView, FuncAssociations,
    FuncError, FuncResult,
};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{
    func::argument::FuncArgument, qualification_prototype::QualificationPrototypeContextField,
    AttributePrototype, AttributePrototypeArgument, DalContext, Func, FuncBackendKind, FuncId,
    QualificationPrototype, StandardModel, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveFuncRequest {
    pub id: FuncId,
    pub handler: Option<String>,
    pub name: String,
    pub description: Option<String>,
    pub code: Option<String>,
    pub associations: Option<FuncAssociations>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SaveFuncResponse {
    pub associations: Option<FuncAssociations>,
    pub success: bool,
    pub is_revertible: bool,
}

async fn save_attr_func_proto_arguments(
    ctx: &DalContext,
    proto: &AttributePrototype,
    arguments: Vec<AttributePrototypeArgumentView>,
    create_all: bool,
) -> FuncResult<()> {
    for arg in &arguments {
        if let Some(arg_id) = arg.id {
            let proto_arg = if arg_id.is_none() || create_all {
                match arg.internal_provider_id {
                    Some(internal_provider_id) => Some(
                        AttributePrototypeArgument::new_for_intra_component(
                            ctx,
                            *proto.id(),
                            arg.func_argument_id,
                            internal_provider_id,
                        )
                        .await?,
                    ),
                    None => None,
                }
            } else {
                Some(
                    AttributePrototypeArgument::get_by_id(ctx, &arg_id)
                        .await?
                        .ok_or_else(|| {
                            FuncError::AttributePrototypeMissingArgument(*proto.id(), arg_id)
                        })?,
                )
            };

            if let Some(mut proto_arg) = proto_arg {
                if proto_arg.attribute_prototype_id() != *proto.id() {
                    proto_arg
                        .set_attribute_prototype_id(ctx, *proto.id())
                        .await?;
                }

                if let Some(internal_provider_id) = arg.internal_provider_id {
                    if internal_provider_id != proto_arg.internal_provider_id() {
                        proto_arg
                            .set_internal_provider_id_safe(ctx, internal_provider_id)
                            .await?;
                    }
                }
            }
        } else if let Some(internal_provider_id) = arg.internal_provider_id {
            AttributePrototypeArgument::new_for_intra_component(
                ctx,
                *proto.id(),
                arg.func_argument_id,
                internal_provider_id,
            )
            .await?;
        } // else condition should be error here? (saving an arg that has no internal provider id)
    }

    Ok(())
}

async fn save_attr_func_prototypes(
    ctx: &DalContext,
    func: &Func,
    prototypes: Vec<AttributePrototypeView>,
) -> FuncResult<()> {
    let mut id_set = HashSet::new();
    for proto_view in prototypes {
        let context = proto_view.into_context(ctx).await?;

        let create_all: bool;
        let proto = if proto_view.id.is_none() {
            create_all = false;
            AttributePrototype::new_with_context_only(ctx, *func.id(), context, None).await?
        } else {
            let proto = AttributePrototype::get_by_id(ctx, &proto_view.id)
                .await?
                .ok_or(FuncError::AttributePrototypeMissing)?;

            if proto.context != context {
                create_all = true;
                AttributePrototype::remove(ctx, proto.id()).await?;
                AttributePrototype::new_with_context_only(ctx, *func.id(), context, None).await?
            } else {
                create_all = false;
                proto
            }
        };

        id_set.insert(*proto.id());

        save_attr_func_proto_arguments(ctx, &proto, proto_view.prototype_arguments, create_all)
            .await?;
    }

    // TODO: should use a custom query to fetch for *not in* id_set only
    for proto in AttributePrototype::find_for_func(ctx, func.id()).await? {
        if !id_set.contains(proto.id()) {
            AttributePrototype::remove(ctx, proto.id()).await?;
        }
    }

    Ok(())
}

async fn save_attr_func_arguments(
    ctx: &DalContext,
    func: &Func,
    arguments: Vec<FuncArgumentView>,
) -> FuncResult<()> {
    let mut id_set = HashSet::new();
    for arg in &arguments {
        let arg_id = if arg.id.is_some() {
            id_set.insert(arg.id);
            let mut existing = FuncArgument::get_by_id(ctx, &arg.id)
                .await?
                .ok_or(FuncError::FuncArgNotFound)?;
            existing.set_name(ctx, &arg.name).await?;
            existing.set_kind(ctx, arg.kind).await?;
            existing.set_element_kind(ctx, arg.element_kind).await?;

            *existing.id()
        } else {
            let new_arg =
                FuncArgument::new(ctx, &arg.name, arg.kind, arg.element_kind, *func.id()).await?;
            *new_arg.id()
        };

        id_set.insert(arg_id);
    }

    for func_arg in FuncArgument::list_for_func(ctx, *func.id()).await? {
        if !id_set.contains(func_arg.id()) {
            FuncArgument::remove(ctx, func_arg.id()).await?;
        }
    }

    Ok(())
}

pub async fn save_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<SaveFuncRequest>,
) -> FuncResult<Json<SaveFuncResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut func = Func::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(FuncError::FuncNotFound)?;

    // Don't modify builtins, or for other tenancies
    if !ctx.check_tenancy(&func).await? {
        return Err(FuncError::NotWritable);
    }

    func.set_display_name(&ctx, Some(request.name)).await?;
    func.set_description(&ctx, request.description).await?;
    func.set_handler(&ctx, request.handler).await?;
    func.set_code_plaintext(&ctx, request.code.as_deref())
        .await?;

    match func.backend_kind() {
        FuncBackendKind::JsQualification => {
            let mut associations: Vec<QualificationPrototypeContextField> = vec![];
            if let Some(FuncAssociations::Qualification {
                schema_variant_ids,
                component_ids,
            }) = request.associations
            {
                associations.append(&mut schema_variant_ids.iter().map(|f| (*f).into()).collect());
                associations.append(&mut component_ids.iter().map(|f| (*f).into()).collect());
            };

            let _ = QualificationPrototype::associate_prototypes_with_func_and_objects(
                &ctx,
                func.id(),
                &associations,
            )
            .await?;
        }
        FuncBackendKind::JsAttribute => {
            if let Some(FuncAssociations::Attribute {
                prototypes,
                arguments,
            }) = request.associations
            {
                save_attr_func_prototypes(&ctx, &func, prototypes).await?;
                save_attr_func_arguments(&ctx, &func, arguments).await?;
            }
        }
        _ => {}
    }

    let is_revertible = super::is_func_revertible(&ctx, &func).await?;
    let associations = super::get_func_view(&ctx, &func).await?.associations;

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;
    ctx.commit().await?;

    Ok(Json(SaveFuncResponse {
        associations,
        success: true,
        is_revertible,
    }))
}
