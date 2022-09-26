use super::{
    AttributePrototypeArgumentView, AttributePrototypeView, FuncAssociations, FuncError, FuncResult,
};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{
    AttributePrototype, AttributePrototypeArgument, DalContext, Func, FuncBackendKind, FuncId,
    Prop, QualificationPrototype, StandardModel, Visibility,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetFuncRequest {
    pub id: FuncId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetFuncResponse {
    pub id: FuncId,
    pub handler: Option<String>,
    pub kind: FuncBackendKind,
    pub name: String,
    pub description: Option<String>,
    pub code: Option<String>,
    pub is_builtin: bool,
    pub is_revertable: bool,
    pub associations: Option<FuncAssociations>,
}

async fn prototype_view_for_prototype(
    ctx: &DalContext,
    proto: &AttributePrototype,
) -> FuncResult<AttributePrototypeView> {
    let prop_id = if proto.context.prop_id().is_some() {
        proto.context.prop_id()
    } else {
        return Err(FuncError::AttributePrototypeMissingPropId(*proto.id()));
    };

    let component_id = if proto.context.component_id().is_some() {
        Some(proto.context.component_id())
    } else {
        None
    };

    let schema_variant_id = if proto.context.schema_variant_id().is_none() {
        let prop = Prop::get_by_id(ctx, &prop_id)
            .await?
            .ok_or_else(|| FuncError::AttributePrototypeMissingProp(*proto.id(), prop_id))?;

        match prop.schema_variants(ctx).await?.pop() {
            Some(schema_variant) => *schema_variant.id(),
            None => {
                return Err(FuncError::AttributePrototypeMissingSchemaVariant(
                    *proto.id(),
                ))
            }
        }
    } else {
        proto.context.schema_variant_id()
    };

    let prototype_arguments =
        AttributePrototypeArgument::list_for_attribute_prototype(ctx, *proto.id())
            .await?
            .iter()
            .map(|arg| AttributePrototypeArgumentView {
                id: *arg.id(),
                name: arg.name().to_string(),
                internal_provider_id: arg.internal_provider_id(),
            })
            .collect();

    Ok(AttributePrototypeView {
        prop_id,
        component_id,
        schema_variant_id,
        prototype_arguments,
    })
}

pub async fn get_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetFuncRequest>,
) -> FuncResult<Json<GetFuncResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let func = Func::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(FuncError::FuncNotFound)?;

    let associations = match func.backend_kind() {
        FuncBackendKind::JsQualification => {
            let protos = QualificationPrototype::find_for_func(&ctx, func.id()).await?;

            let mut schema_variant_ids = vec![];
            let mut component_ids = vec![];

            for proto in protos {
                if proto.context().schema_variant_id().is_some() {
                    schema_variant_ids.push(proto.context().schema_variant_id());
                } else if proto.context().component_id().is_some() {
                    component_ids.push(proto.context().component_id());
                }
            }

            Some(FuncAssociations::Qualification {
                schema_variant_ids,
                component_ids,
            })
        }
        FuncBackendKind::JsAttribute => {
            let protos = AttributePrototype::find_for_func(&ctx, func.id()).await?;
            let mut prototype_views = vec![];

            for proto in &protos {
                prototype_views.push(prototype_view_for_prototype(&ctx, proto).await?);
            }

            Some(FuncAssociations::Attribute {
                prototypes: prototype_views,
            })
        }
        _ => None,
    };

    let is_revertable = super::is_func_revertable(&ctx, &func).await?;

    Ok(Json(GetFuncResponse {
        id: func.id().to_owned(),
        handler: func.handler().map(|h| h.to_owned()),
        kind: func.backend_kind().to_owned(),
        name: func
            .display_name()
            .unwrap_or_else(|| func.name())
            .to_owned(),
        description: func.description().map(|d| d.to_owned()),
        code: func.code_plaintext()?,
        is_builtin: func.is_builtin(),
        is_revertable,
        associations,
    }))
}
