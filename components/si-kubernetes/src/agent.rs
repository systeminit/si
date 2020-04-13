use crate::model::entity::EntityEvent;
use si_cea::agent::dispatch::Dispatch;
use si_cea::{
    gen_dispatch, gen_dispatch_keys, gen_dispatch_setup, gen_dispatcher, CeaResult,
    MqttAsyncClientInternal,
};
use si_data::Db;

gen_dispatcher!(self_ident: self);

#[async_trait::async_trait]
impl Dispatch<EntityEvent> for Dispatcher {
    gen_dispatch_keys!(self);

    async fn setup(&mut self, db: &Db) -> CeaResult<()> {
        gen_dispatch_setup!(self, db, {
            integration_name: "aws",
            integration_service_name: "eks_kubernetes",
            dispatch[
                ("create", aws::create),
                ("sync", aws::sync),
                ("edit", aws::edit)
            ]
        });
        Ok(())
    }

    async fn dispatch(
        &self,
        mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut EntityEvent,
        integration_service_id: String,
        action_name: String,
    ) -> CeaResult<()> {
        gen_dispatch!(
            self,
            mqtt_client,
            entity_event,
            integration_service_id,
            action_name,
            dispatch[
                aws::create,
                aws::sync,
                aws::edit
            ]
        );
        Ok(())
    }
}

mod aws {
    use crate::model::entity::{Entity, EntityEvent};
    use si_cea::{CeaError, CeaResult, EntityEvent as _, MqttAsyncClientInternal};
    use tracing::debug_span;
    use tracing_futures::Instrument as _;

    pub async fn create(
        _mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut EntityEvent,
    ) -> CeaResult<()> {
        async {
            entity_event.log("Kubernetes like a motherfucker\n");
            entity_event.log(format!("{:?}", entity_event.input_entity()));
            entity_event.init_output();
            Ok(())
        }
        .instrument(debug_span!("kubernetes_deployment_create"))
        .await
    }

    pub async fn sync(
        _mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut EntityEvent,
    ) -> CeaResult<()> {
        async {
            entity_event.log("Synchronizing Kubernetes like we just dont care\n");
            entity_event.init_output();
            Ok(())
        }
        .instrument(debug_span!("kubernetes_deployment_sync"))
        .await
    }

    pub async fn edit(
        _mqtt_client: &MqttAsyncClientInternal,
        entity_event: &mut EntityEvent,
    ) -> CeaResult<()> {
        async {
            entity_event.log("Editing Kubernetes like we just dont care\n");
            entity_event.init_output();
            Ok(())
        }
        .instrument(debug_span!("kubernetes_deployment_sync"))
        .await
    }

    impl EntityEvent {
        fn init_output(&mut self) {
            self.output_entity = self.input_entity.clone();
        }

        fn output(&self) -> CeaResult<&Entity> {
            self.output_entity
                .as_ref()
                .ok_or(CeaError::MissingOutputEntity)
        }

        fn output_as_mut(&mut self) -> CeaResult<&mut Entity> {
            self.output_entity
                .as_mut()
                .ok_or(CeaError::MissingOutputEntity)
        }
    }
}
