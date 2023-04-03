use super::{FuncError, FuncResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::Json;
use dal::{
    job::definition::DependentValuesUpdate, AttributePrototype, AttributeValue,
    AttributeValueError, AttributeValueId, Component, DalContext, Func, FuncBackendKind, FuncId,
    PropId, StandardModel, ValidationPrototype, Visibility, WsEvent,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

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
    if prototypes.is_empty() {
        return Err(FuncError::FuncExecutionFailedNoPrototypes);
    }

    for proto in prototypes {
        let mut values = proto.attribute_values(ctx).await?;
        let value_ids = values
            .iter()
            .cloned()
            .map(|av| *av.id())
            .collect::<Vec<AttributeValueId>>();

        for value in values.iter_mut() {
            match value.update_from_prototype_function(ctx).await {
                Ok(_) => {}
                Err(AttributeValueError::FuncBackendResultFailure { message, .. }) => {
                    return Err(FuncError::FuncExecutionFailed(message))
                }
                Err(err) => Err(err)?,
            }
        }

        ctx.enqueue_job(DependentValuesUpdate::new(ctx, value_ids))
            .await;
    }

    Ok(())
}

async fn run_validations(ctx: &DalContext, func: &Func) -> FuncResult<()> {
    for proto in ValidationPrototype::list_for_func(ctx, *func.id()).await? {
        let schema_variant_id = proto.context().schema_variant_id();
        if schema_variant_id.is_none() {
            continue;
        }
        let components = Component::list_for_schema_variant(ctx, schema_variant_id).await?;
        for component in components {
            let mut cache: HashMap<PropId, (Option<Value>, AttributeValue)> = HashMap::new();
            component
                .check_single_validation(ctx, &proto, &mut cache)
                .await?;
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
        FuncBackendKind::JsAttribute => {
            update_values_for_func(&ctx, &func).await?;
        }
        FuncBackendKind::JsValidation => {
            run_validations(&ctx, &func).await?;
        }
        _ => {}
    }

    WsEvent::change_set_written(&ctx)
        .await?
        .publish_on_commit(&ctx)
        .await?;

    ctx.commit().await?;

    Ok(Json(ExecFuncResponse { success: true }))
}
