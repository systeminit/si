use super::{FuncError, FuncResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{
    job::definition::{DependentValuesUpdate, Qualification},
    AttributePrototype, Component, DalContext, Func, FuncBackendKind, FuncId, PrototypeListForFunc,
    QualificationPrototype, QualificationPrototypeError, StandardModel, SystemId, Visibility,
    WsEvent,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecFuncRequest {
    pub id: FuncId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ExecFuncResponse {
    pub success: bool,
}

async fn update_values_for_func(ctx: &DalContext, func: &Func) -> FuncResult<()> {
    let prototypes = AttributePrototype::find_for_func(ctx, func.id()).await?;
    for proto in prototypes {
        for value in proto.attribute_values(ctx).await?.iter_mut() {
            value.update_from_prototype_function(ctx).await?;
            ctx.enqueue_job(DependentValuesUpdate::new(ctx, *value.id()))
                .await;
        }
    }

    Ok(())
}

async fn run_qualifications(ctx: &DalContext, func: &Func) -> FuncResult<()> {
    for proto in QualificationPrototype::list_for_func(ctx, *func.id()).await? {
        let component_id = proto.component_id();
        let schema_variant_id = proto.schema_variant_id();

        if component_id.is_none() && schema_variant_id.is_none() {
            continue;
        }

        if component_id.is_some() {
            ctx.enqueue_job(
                Qualification::new(ctx, component_id, *proto.id(), SystemId::NONE)
                    .await
                    .map_err(|err| QualificationPrototypeError::Component(err.to_string()))?,
            )
            .await;
        } else if schema_variant_id.is_some() {
            for component in Component::list_for_schema_variant(ctx, schema_variant_id)
                .await
                .map_err(|err| QualificationPrototypeError::Component(err.to_string()))?
            {
                ctx.enqueue_job(
                    Qualification::new(ctx, *component.id(), *proto.id(), SystemId::NONE)
                        .await
                        .map_err(|err| QualificationPrototypeError::Component(err.to_string()))?,
                )
                .await;
            }
        }
    }

    Ok(())
}

pub async fn exec_func(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<ExecFuncRequest>,
) -> FuncResult<Json<ExecFuncResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let func = Func::get_by_id(&ctx, &request.id)
        .await?
        .ok_or(FuncError::FuncNotFound)?;

    match func.backend_kind() {
        FuncBackendKind::JsQualification => {
            run_qualifications(&ctx, &func).await?;
        }
        FuncBackendKind::JsAttribute => {
            update_values_for_func(&ctx, &func).await?;
        }
        _ => {}
    }

    WsEvent::change_set_written(&ctx).publish(&ctx).await?;

    ctx.commit().await?;

    Ok(Json(ExecFuncResponse { success: true }))
}
