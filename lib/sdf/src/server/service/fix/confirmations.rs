use super::FixResult;
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{
    ComponentId, ConfirmationResolver, ConfirmationResolverId, FuncBindingReturnValue,
    StandardModel, Visibility,
};
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
    Running,
    Failure,
    Success,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfirmationView {
    id: ConfirmationResolverId,
    title: String,
    component_id: ComponentId,
    description: Option<String>,
    output: Option<Vec<String>>,
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
        let prototype = resolver.confirmation_prototype(&ctx).await?;

        let mut output = Vec::new();
        if let Some(message) = resolver.message() {
            output.push(message.to_owned());
        }
        if let Some(func_binding_return_value) =
            FuncBindingReturnValue::get_by_func_binding_id(&ctx, resolver.func_binding_id()).await?
        {
            if let Some(output_streams) = func_binding_return_value.get_output_stream(&ctx).await? {
                for output_stream in output_streams {
                    output.push(output_stream.message);
                }
            }
        }

        let status = match resolver.success() {
            Some(true) => ConfirmationStatusView::Success,
            Some(false) => ConfirmationStatusView::Failure,
            None => ConfirmationStatusView::Running,
        };
        views.push(ConfirmationView {
            id: *resolver.id(),
            title: prototype.name().to_owned(),
            description: match status {
                ConfirmationStatusView::Success => {
                    prototype.success_description().map(ToOwned::to_owned)
                }
                ConfirmationStatusView::Failure => {
                    prototype.failure_description().map(ToOwned::to_owned)
                }
                ConfirmationStatusView::Running => None,
            },
            component_id: resolver.context().component_id(),
            output: Some(output).filter(|o| !o.is_empty()),
            status,
        });
    }

    Ok(Json(views))
}
