use serde::Deserialize;
use serde::Serialize;

use crate::action_prototype::ActionKind;
use crate::component::confirmation::ConfirmationStatusView;
use crate::fix::FixResult;
use crate::DalContext;
use crate::{
    AttributeValueId, Component, ComponentId, Fix, FixResolver, Schema, SchemaError, StandardModel,
};

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
    id: AttributeValueId,
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
        let running = Fix::find_by_attr_null(ctx, "finished_at").await?;
        let confirmations = Component::list_confirmations(ctx).await?;

        let mut views = Vec::with_capacity(confirmations.len());
        for confirmation in confirmations {
            let component_id = confirmation.component_id;

            let schema = Schema::get_by_id(ctx, &confirmation.schema_id)
                .await?
                .ok_or(SchemaError::NotFound(confirmation.schema_id))?;

            let fix = FixResolver::find_for_confirmation_attribute_value(
                ctx,
                confirmation.attribute_value_id,
            )
            .await?;

            let recommendations = confirmation.recommended_actions(ctx).await?;

            for action in recommendations {
                let workflow_prototype = action.workflow_prototype(ctx).await?;

                views.push(Recommendation {
                    id: confirmation.attribute_value_id,
                    name: workflow_prototype.title().to_owned(),
                    schema_name: schema.name().to_owned(),
                    component_name: Component::find_name(ctx, component_id).await?,
                    component_id,
                    recommendation: action.name().to_owned(),
                    recommendation_kind: action.kind().to_owned(),
                    status: if confirmation.status == ConfirmationStatusView::Running
                        || running.iter().any(|r| {
                            r.component_id == confirmation.component_id
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
                    provider: confirmation.provider.clone(),
                    output: None,
                })
            }
        }
        Ok(views)
    }
}
