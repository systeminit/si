use super::{FixError, FixResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{
    AttributeReadContext, Component, ComponentId, ConfirmationPrototype, ConfirmationResolver,
    ConfirmationResolverId, ConfirmationResolverTree, FixResolver, FixResolverContext,
    StandardModel, SystemId, Visibility,
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
        if resolver.success().is_none() {
            continue;
        }

        let component_id = resolver.context().component_id;
        if component_id.is_none() {
            continue;
        }

        let component = Component::get_by_id(&ctx, &component_id)
            .await?
            .ok_or(FixError::ComponentNotFound(component_id))?;
        let schema = component
            .schema(&ctx)
            .await?
            .ok_or(FixError::NoSchemaForComponent(component_id))?;
        let schema_variant = component
            .schema_variant(&ctx)
            .await?
            .ok_or(FixError::NoSchemaVariantForComponent(component_id))?;

        let context = FixResolverContext {
            component_id,
            schema_id: *schema.id(),
            schema_variant_id: *schema_variant.id(),
            system_id: SystemId::NONE,
        };
        let fix = FixResolver::find_for_confirmation(&ctx, *resolver.id(), context).await?;

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
                status: match fix.as_ref().and_then(FixResolver::success) {
                    Some(true) => FixStatusView::Success,
                    Some(false) => FixStatusView::Failure,
                    None => FixStatusView::Unstarted,
                },
            })
        }
    }

    Ok(Json(views))
}
