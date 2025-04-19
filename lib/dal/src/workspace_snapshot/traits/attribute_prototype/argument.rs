use async_trait::async_trait;
use si_id::AttributePrototypeArgumentId;

use crate::{
    attribute::prototype::argument::value_source::ValueSource,
    layer_db_types::{AttributeValueSubscriptionContent, AttributeValueSubscriptionContentV1},
    workspace_snapshot::{
        graph::traits::{
            attribute_prototype::argument::{ArgumentValue, AttributePrototypeArgumentExt as _},
            component::ComponentExt,
        },
        WorkspaceSnapshotResult,
    },
    DalContext, WorkspaceSnapshot, WorkspaceSnapshotError,
};

#[async_trait]
pub trait AttributePrototypeArgumentExt {
    async fn value_source(
        &self,
        ctx: &DalContext,
        apa_id: AttributePrototypeArgumentId,
    ) -> WorkspaceSnapshotResult<Option<ValueSource>>;
    async fn argument_value(
        &self,
        apa_id: AttributePrototypeArgumentId,
    ) -> WorkspaceSnapshotResult<Option<ArgumentValue>>;
}

#[async_trait]
impl AttributePrototypeArgumentExt for WorkspaceSnapshot {
    async fn value_source(
        &self,
        ctx: &DalContext,
        apa_id: AttributePrototypeArgumentId,
    ) -> WorkspaceSnapshotResult<Option<ValueSource>> {
        let Some(argument_value) = self.argument_value(apa_id).await? else {
            return Ok(None);
        };
        Ok(Some(match argument_value {
            ArgumentValue::InputSocket(id) => ValueSource::InputSocket(id),
            ArgumentValue::OutputSocket(id) => ValueSource::OutputSocket(id),
            ArgumentValue::Prop(id) => ValueSource::Prop(id),
            ArgumentValue::Secret(id) => ValueSource::Secret(id),
            ArgumentValue::StaticArgumentValue(id) => ValueSource::StaticArgumentValue(id),
            // Look up the value source by path
            ArgumentValue::AttributeValueSubscription {
                component_id,
                json_pointer,
            } => {
                let AttributeValueSubscriptionContent::V1(AttributeValueSubscriptionContentV1 {
                    json_pointer,
                }) = ctx
                    .layer_db()
                    .cas()
                    .try_read_as(&json_pointer.content_hash())
                    .await?
                    .ok_or(WorkspaceSnapshotError::MissingContentFromStoreForAddress(
                        json_pointer,
                    ))?;

                let Some(av_id) = self
                    .working_copy()
                    .await
                    .resolve_attribute_value(component_id, &json_pointer)?
                else {
                    return Ok(None);
                };
                ValueSource::AttributeValue(av_id)
            }
        }))
    }

    async fn argument_value(
        &self,
        apa_id: AttributePrototypeArgumentId,
    ) -> WorkspaceSnapshotResult<Option<ArgumentValue>> {
        Ok(self.working_copy().await.argument_value(apa_id)?)
    }
}
