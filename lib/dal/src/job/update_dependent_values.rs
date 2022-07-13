use crate::{
    attribute::value::dependent_update::AttributeValueDependentUpdateHarness, AttributeValue,
    AttributeValueError, AttributeValueId, BillingAccountId, ComponentId, DalContext, HistoryActor,
    Job, JobFuture, StandardModel, SystemId, WsEvent, WsPayload, JobError,
};
use serde::{Deserialize, Serialize};

#[must_use]
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq, Default)]
pub struct UpdateDependentValuesJob {
    value_id: AttributeValueId,
}

impl UpdateDependentValuesJob {
    pub fn new(value_id: AttributeValueId) -> Self {
        Self { value_id }
    }
}

impl Job for UpdateDependentValuesJob {
    fn prepare<'a, 'b, 'c>(&self, ctx: &'a DalContext<'b, 'c>) -> JobFuture {
        Box::new(async { Ok(()) })
    }

    fn run<'a, 'b, 'c>(&self, ctx: &'a DalContext<'b, 'c>) -> JobFuture {
        Box::new(async move {
            // After we have _completely_ updated ourself, we can update our dependent values.
            AttributeValueDependentUpdateHarness::update_dependent_values(ctx, self.value_id)
                .await
                .map_err(|err| JobError::AttributeValue(err.to_string()))?;

            let attribute_value = AttributeValue::get_by_id(ctx, &self.value_id)
                .await?
                .ok_or(AttributeValueError::NotFound(
                    self.value_id,
                    ctx.visibility().clone(),
                ))
                .map_err(|err| JobError::AttributeValue(err.to_string()))?;

            if attribute_value.context.component_id().is_some() {
                WsEvent::updated_dependent_value(
                    attribute_value.context.component_id(),
                    attribute_value.context.system_id(),
                    ctx.read_tenancy().billing_accounts().into(),
                    ctx.history_actor(),
                )
                .publish(ctx.txns().nats())
                .await?;
            }
            Ok(())
        })
    }

    fn name(&self) -> &'static str {
        "UpdateDependentValuesJob"
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DependentValuesUpdated {
    component_id: ComponentId,
    system_id: SystemId,
}

impl WsEvent {
    pub fn updated_dependent_value(
        component_id: ComponentId,
        system_id: SystemId,
        billing_account_ids: Vec<BillingAccountId>,
        history_actor: &HistoryActor,
    ) -> Self {
        WsEvent::new(
            billing_account_ids,
            history_actor.clone(),
            WsPayload::UpdatedDependentValue(DependentValuesUpdated {
                component_id,
                system_id,
            }),
        )
    }
}
