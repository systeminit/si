use axum::{extract::Query, Json};
use dal::node::NodeId;
use dal::{Component, ComponentId, StandardModel, Visibility, WorkspaceId, WriteTenancy};
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
pub struct ApplicationView {
    id: ComponentId,
    name: String,
    visibility: Visibility,
    tenancy: WriteTenancy,
}

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GetApplicationResponse {
    pub application: ApplicationView,
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

    let application = Component::get_by_id(&ctx, &request.application_id)
        .await?
        .ok_or(ApplicationError::NotFound)?;

    let application_node = application
        .node(&ctx)
        .await?
        .pop()
        .ok_or(ApplicationError::NotFound)?;

    let name = application
        .find_value_by_json_pointer::<String>(&ctx, "/root/si/name")
        .await?
        .ok_or(ApplicationError::NameNotFound)?;
    let application = ApplicationView {
        id: *application.id(),
        name,
        visibility: *application.visibility(),
        tenancy: application.tenancy().clone(),
    };

    txns.commit().await?;

    Ok(Json(GetApplicationResponse {
        application,
        application_node_id: *application_node.id(),
    }))
}
