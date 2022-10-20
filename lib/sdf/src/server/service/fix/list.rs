use super::{FixError, FixResult};
use crate::server::extract::{AccessBuilder, HandlerContext};
use axum::{extract::Query, Json};
use dal::{
    Component, ComponentId, ConfirmationPrototype, ConfirmationResolver, ConfirmationResolverId,
    ConfirmationResolverTree, StandardModel, Visibility,
};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct ListFixesRequest {
    #[serde(flatten)]
    pub visibility: Visibility,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListedFixView {
    name: String,
    confirmation_resolver_id: ConfirmationResolverId,
    success: bool,
    message: Option<String>,
    recommended_actions: Vec<String>,
    component_id: ComponentId,
    schema_name: String,
    schema_variant_name: String,
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
        let prototype =
            ConfirmationPrototype::get_by_id(&ctx, &resolver.confirmation_prototype_id())
                .await?
                .ok_or_else(|| {
                    FixError::ConfirmationPrototypeNotFound(resolver.confirmation_prototype_id())
                })?;

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

        views.push(ListedFixView {
            name: prototype.name().to_owned(),
            confirmation_resolver_id: *resolver.id(),
            success: resolver.success(),
            message: resolver.message().map(ToOwned::to_owned),
            recommended_actions: resolver
                .recommended_actions(&ctx)
                .await?
                .into_iter()
                .map(|action| action.name().to_owned())
                .collect(),
            component_id,
            schema_name: schema.name().to_owned(),
            schema_variant_name: schema_variant.name().to_owned(),
        });
    }

    Ok(Json(views))
}
