use axum::Json;
use dal::{
    component::ComponentAsyncTasks, Component, ComponentId, StandardModel, SystemId, Visibility,
};
use serde::{Deserialize, Serialize};

use super::{ComponentError, ComponentResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use telemetry::prelude::*;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CheckQualficationsRequest {
    pub component_id: ComponentId,
    pub system_id: Option<SystemId>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CheckQualficationsResponse {
    pub success: bool,
}

pub async fn check_qualifications(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<CheckQualficationsRequest>,
) -> ComponentResult<Json<CheckQualficationsResponse>> {
    let system_id = request.system_id.unwrap_or_else(|| SystemId::from(-1));

    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.clone().build(request.visibility), &txns);

    let is_component_in_tenancy = Component::is_in_tenancy(&ctx, request.component_id).await?;
    let is_component_in_visibility = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .is_some();
    if is_component_in_tenancy && !is_component_in_visibility {
        return Err(ComponentError::InvalidVisibility);
    }

    let component = Component::get_by_id(&ctx, &request.component_id)
        .await?
        .ok_or(ComponentError::ComponentNotFound)?;
    component
        .prepare_qualifications_check(&ctx, system_id)
        .await?;
    txns.commit().await?;

    let async_tasks = ComponentAsyncTasks::new(component, system_id);
    tokio::task::spawn(async move {
        if let Err(err) = async_tasks
            .run(request_ctx, request.visibility, &builder)
            .await
        {
            error!("Component async qualification check failed: {err}");
        }
    });

    Ok(Json(CheckQualficationsResponse { success: true }))
}
