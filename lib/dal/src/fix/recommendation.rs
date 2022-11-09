use serde::Deserialize;
use serde::Serialize;

use crate::fix::{FixError, FixResult};
use crate::{
    AttributeReadContext, Component, ComponentId, ConfirmationPrototype, ConfirmationResolver,
    ConfirmationResolverId, ConfirmationResolverTree, FixResolver, FixResolverContext,
    StandardModel, SystemId,
};
use crate::{ComponentError, DalContext};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RecommendationStatus {
    Success,
    Failure,
    Unstarted,
}

/// A non-persistent object that can be used to create and run [`Fixes`](crate::Fix).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Recommendation {
    id: ConfirmationResolverId,
    name: String,
    component_name: String,
    component_id: ComponentId,
    recommendation: String,
    status: RecommendationStatus,
}

impl Recommendation {
    pub async fn list(ctx: &DalContext) -> FixResult<Vec<Recommendation>> {
        let resolvers = ConfirmationResolver::list(ctx).await?;

        // Sorted resolvers
        let resolvers = ConfirmationResolverTree::build(ctx, resolvers)
            .await?
            .into_vec()?;

        let mut views = Vec::with_capacity(resolvers.len());
        for resolver in resolvers {
            if resolver.success().is_none() {
                continue;
            }

            let component_id = resolver.context().component_id;
            if component_id.is_none() {
                continue;
            }

            let component = Component::get_by_id(ctx, &component_id)
                .await?
                .ok_or(ComponentError::NotFound(component_id))?;
            let schema = component
                .schema(ctx)
                .await?
                .ok_or(ComponentError::NoSchema(component_id))?;
            let schema_variant = component
                .schema_variant(ctx)
                .await?
                .ok_or(ComponentError::NoSchemaVariant(component_id))?;

            let context = FixResolverContext {
                component_id,
                schema_id: *schema.id(),
                schema_variant_id: *schema_variant.id(),
                system_id: SystemId::NONE,
            };
            let fix = FixResolver::find_for_confirmation(ctx, *resolver.id(), context).await?;

            let recommendations = resolver
                .recommended_actions(ctx)
                .await?
                .into_iter()
                .map(|action| action.name().to_owned());

            for recommendation in recommendations {
                let prototype =
                    ConfirmationPrototype::get_by_id(ctx, &resolver.confirmation_prototype_id())
                        .await?
                        .ok_or_else(|| {
                            FixError::ConfirmationPrototypeNotFound(
                                resolver.confirmation_prototype_id(),
                            )
                        })?;

                views.push(Recommendation {
                    id: *resolver.id(),
                    name: prototype.name().to_owned(),
                    component_name: Component::name_from_context(
                        ctx,
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
                        Some(true) => RecommendationStatus::Success,
                        Some(false) => RecommendationStatus::Failure,
                        None => RecommendationStatus::Unstarted,
                    },
                })
            }
        }
        Ok(views)
    }
}
