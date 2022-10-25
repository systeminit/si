use super::{FixError, FixResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{
    AttributeReadContext, Component, ComponentId, ConfirmationPrototype, ConfirmationResolver,
    ConfirmationResolverId, ConfirmationResolverTree, StandardModel, SystemId, Visibility,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListFixesRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum FixStatusView {
    Success,
    Failure,
    Unstarted,
}

// TODO: add fields that are optional in the frontend
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListedFixView {
    id: ConfirmationResolverId,
    name: String,
    component_name: String,
    component_id: ComponentId,
    recommendation: String,
    status: FixStatusView,
}

pub type ListFixesResponse = Vec<ListedFixView>;

pub async fn list(
    HandlerContext(builder): HandlerContext,
    AccessBuilder(request_ctx): AccessBuilder,
    Query(request): Query<ListFixesRequest>,
) -> FixResult<Json<ListFixesResponse>> {
    let ctx = builder.build(request_ctx.build(request.visibility)).await?;

    let resolvers = ConfirmationResolver::list(&ctx).await?;

    // Sorted resolvers
    let resolvers = ConfirmationResolverTree::build(&ctx, resolvers)
        .await?
        .into_vec();

    let mut views = Vec::with_capacity(resolvers.len());
    for resolver in resolvers {
        // Resolver is being executed
        if resolver.success().is_none() {
            continue;
        }

        let component_id = resolver.context().component_id;
        if component_id.is_none() {
            continue;
        }

        let recommendations = resolver
            .recommended_actions(&ctx)
            .await?
            .into_iter()
            .map(|action| action.name().to_owned());

        for recommendation in recommendations {
            let prototype =
                ConfirmationPrototype::get_by_id(&ctx, &resolver.confirmation_prototype_id())
                    .await?
                    .ok_or_else(|| {
                        FixError::ConfirmationPrototypeNotFound(
                            resolver.confirmation_prototype_id(),
                        )
                    })?;

            views.push(ListedFixView {
                id: *resolver.id(),
                name: prototype.name().to_owned(),
                component_name: Component::name_from_context(
                    &ctx,
                    AttributeReadContext {
                        component_id: Some(component_id),
                        system_id: Some(SystemId::NONE),
                        ..AttributeReadContext::any()
                    },
                )
                .await?,
                component_id,
                recommendation,
                status: FixStatusView::Unstarted,
            });
        }
    }

    Ok(Json(views))
}
