use super::FixResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{ConfirmationResolver, ConfirmationResolverId, StandardModel, Visibility};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationsRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ConfirmationStatusView {
    Finished,
    Running,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationView {
    id: ConfirmationResolverId,
    status: ConfirmationStatusView,
}

pub type ConfirmationsResponse = Vec<ConfirmationView>;

pub async fn confirmations(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ConfirmationsRequest>,
) -> FixResult<Json<ConfirmationsResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let resolvers = ConfirmationResolver::list(&ctx).await?;
    let mut views = Vec::with_capacity(resolvers.len());

    for resolver in resolvers {
        views.push(ConfirmationView {
            id: *resolver.id(),
            status: if resolver.success().is_some() {
                ConfirmationStatusView::Finished
            } else {
                ConfirmationStatusView::Running
            },
        });
    }

    Ok(Json(views))
}
