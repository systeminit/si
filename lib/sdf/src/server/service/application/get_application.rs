use axum::{extract::Query, Json};
use dal::node::NodeId;
use dal::{Component, ComponentId, StandardModel, Visibility, WorkspaceId};
use serde::{Deserialize, Serialize};

use super::ApplicationResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use crate::service::application::ApplicationError;

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetApplicationRequest {
    pub application_id: ComponentId,
    pub workspace_id: WorkspaceId,
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetApplicationResponse {
    pub application: Component,
    pub application_node_id: NodeId,
}

//pub type GetApplicationResponse = Option<Component>;

pub async fn get_application(
    HandlerContext(builder, mut txns): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<GetApplicationRequest>,
) -> ApplicationResult<Json<GetApplicationResponse>> {
    let txns = txns.start().await?;
    let ctx = builder.build(request_ctx.build(request.visibility), &txns);

    let application = Component::get_by_id(
        ctx.pg_txn(),
        &ctx.read_tenancy().into(),
        ctx.visibility(),
        &request.application_id,
    )
    .await?
    .ok_or(ApplicationError::NotFound)?;
    let application_node = application
        .node(ctx.pg_txn(), &ctx.read_tenancy().into(), ctx.visibility())
        .await?
        .pop()
        .ok_or(ApplicationError::NotFound)?;

    txns.commit().await?;

    Ok(Json(GetApplicationResponse {
        application,
        application_node_id: *application_node.id(),
    }))
}
