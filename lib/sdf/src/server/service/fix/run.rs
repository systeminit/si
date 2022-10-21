use axum::Json;
use serde::{Deserialize, Serialize};

use super::{FixError, FixResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use dal::{
    job::definition::{fix::Fix, Fixes},
    ActionPrototype, Component, ComponentId, ConfirmationResolverId, StandardModel,
    SystemId, Visibility,
};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FixRunRequest {
    pub confirmation_resolver_id: ConfirmationResolverId,
    pub component_id: ComponentId,
    pub action_name: String,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FixesRunRequest {
    pub list: Vec<FixRunRequest>,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct FixesRunResponse {
    success: bool,
}

pub async fn run(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Json(request): Json<FixesRunRequest>,
) -> FixResult<Json<FixesRunResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let mut fixes = Vec::with_capacity(request.list.len());
    for fix in request.list {
        let component = Component::get_by_id(&ctx, &fix.component_id)
            .await?
            .ok_or(FixError::ComponentNotFound(fix.component_id))?;
        let schema = component
            .schema(&ctx)
            .await?
            .ok_or(FixError::NoSchemaForComponent(fix.component_id))?;
        let schema_variant = component
            .schema_variant(&ctx)
            .await?
            .ok_or(FixError::NoSchemaVariantForComponent(fix.component_id))?;

        let action = ActionPrototype::find_by_name(
            &ctx,
            &fix.action_name,
            *schema.id(),
            *schema_variant.id(),
            SystemId::NONE,
        )
        .await?
        .ok_or_else(|| FixError::ActionNotFound(fix.action_name.clone(), fix.component_id))?;

        fixes.push(Fix {
            confirmation_resolver_id: fix.confirmation_resolver_id,
            component_id: fix.component_id,
            workflow_prototype_id: action.workflow_prototype_id(),
        });
    }

    ctx.enqueue_job(Fixes::new(&ctx, fixes)).await;

    ctx.commit().await?;

    Ok(Json(FixesRunResponse { success: true }))
}
