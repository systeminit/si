use serde::Deserialize;
use serde::Serialize;

use crate::action_prototype::ActionKind;
use crate::fix::FixResult;
use crate::{
    AttributeReadContext, Component, ComponentId, ConfirmationResolver, ConfirmationResolverId,
    ConfirmationResolverTree, Fix, FixResolver, FixResolverContext, StandardModel,
};
use crate::{ComponentError, DalContext};

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum RecommendationStatus {
    Success,
    Failure,
    Running,
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
    schema_name: String,
    recommendation: String,
    recommendation_kind: ActionKind,
    status: RecommendationStatus,
    provider: Option<String>,
    output: Option<String>,
}

impl Recommendation {
    pub async fn list(ctx: &DalContext) -> FixResult<Vec<Recommendation>> {
        let resolvers = ConfirmationResolver::list(ctx).await?;

        let running = Fix::find_by_attr_null(ctx, "finished_at").await?;

        // Sorted resolvers
        let resolvers = ConfirmationResolverTree::build(ctx, resolvers)
            .await?
            .into_vec()?;

        let mut views = Vec::with_capacity(resolvers.len());
        for resolver in resolvers {
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
            };
            let fix = FixResolver::find_for_confirmation(ctx, *resolver.id(), context).await?;

            let recommendations = resolver.recommended_actions(ctx).await?;

            for action in recommendations {
                let workflow_prototype = action.workflow_prototype(ctx).await?;
                let prototype = resolver.confirmation_prototype(ctx).await?;

                views.push(Recommendation {
                    id: *resolver.id(),
                    name: workflow_prototype.title().to_owned(),
                    schema_name: schema.name().to_owned(),
                    component_name: Component::name_from_context(
                        ctx,
                        AttributeReadContext {
                            component_id: Some(component_id),
                            ..AttributeReadContext::any()
                        },
                    )
                    .await?,
                    component_id,
                    recommendation: action.name().to_owned(),
                    recommendation_kind: action.kind().to_owned(),
                    status: if resolver.success().is_none()
                        || running.iter().any(|r| {
                            r.confirmation_resolver_id() == *resolver.id()
                                && r.action() == action.name()
                        }) {
                        RecommendationStatus::Running
                    } else {
                        match fix.as_ref().and_then(FixResolver::success) {
                            Some(true) => RecommendationStatus::Success,
                            Some(false) => RecommendationStatus::Failure,
                            None => RecommendationStatus::Unstarted,
                        }
                    },
                    provider: prototype.provider().map(ToOwned::to_owned),
                    output: None,
                })
            }
        }
        Ok(views)
    }
}
