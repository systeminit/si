use crate::gen::agent::{GlobalCoreSystemDispatchFunctions, GlobalCoreSystemDispatcher};
use crate::model::SystemEntityEvent;
use si_cea::agent::dispatch::prelude::*;

#[derive(Clone)]
pub struct GlobalCoreSystemDispatchFunctionsImpl;

#[async_trait]
impl GlobalCoreSystemDispatchFunctions for GlobalCoreSystemDispatchFunctionsImpl {
    type EntityEvent = SystemEntityEvent;

    async fn create(
        _mqtt_client: &MqttClient,
        _entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async { Ok(()) }.instrument(debug_span!("create")).await
    }

    async fn edit_phantom(
        _mqtt_client: &MqttClient,
        _entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async { Ok(()) }
            .instrument(debug_span!("edit_phantom"))
            .await
    }

    async fn sync(
        _mqtt_client: &MqttClient,
        _entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async { Ok(()) }.instrument(debug_span!("sync")).await
    }
}

pub fn dispatcher() -> GlobalCoreSystemDispatcher<GlobalCoreSystemDispatchFunctionsImpl> {
    GlobalCoreSystemDispatcher::<GlobalCoreSystemDispatchFunctionsImpl>::new()
}
