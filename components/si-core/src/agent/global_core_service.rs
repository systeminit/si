use crate::gen::agent::{GlobalCoreServiceDispatchFunctions, GlobalCoreServiceDispatcher};
use crate::model::ServiceEntityEvent;
use si_cea::agent::dispatch::prelude::*;

#[derive(Clone)]
pub struct GlobalCoreServiceDispatchFunctionsImpl;

#[async_trait]
impl GlobalCoreServiceDispatchFunctions for GlobalCoreServiceDispatchFunctionsImpl {
    type EntityEvent = ServiceEntityEvent;

    async fn create(
        _mqtt_client: &MqttClient,
        _entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async { Ok(()) }.instrument(debug_span!("create")).await
    }

    async fn edit_image(
        _mqtt_client: &MqttClient,
        _entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async { Ok(()) }.instrument(debug_span!("edit_image")).await
    }

    async fn edit_port(
        _mqtt_client: &MqttClient,
        _entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async { Ok(()) }.instrument(debug_span!("edit_port")).await
    }

    async fn edit_replicas(
        _mqtt_client: &MqttClient,
        _entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async { Ok(()) }
            .instrument(debug_span!("edit_replicas"))
            .await
    }

    async fn deploy(
        _mqtt_client: &MqttClient,
        _entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async { Ok(()) }.instrument(debug_span!("deploy")).await
    }
    async fn sync(
        _mqtt_client: &MqttClient,
        _entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async { Ok(()) }.instrument(debug_span!("sync")).await
    }
}

pub fn dispatcher() -> GlobalCoreServiceDispatcher<GlobalCoreServiceDispatchFunctionsImpl> {
    GlobalCoreServiceDispatcher::<GlobalCoreServiceDispatchFunctionsImpl>::new()
}
