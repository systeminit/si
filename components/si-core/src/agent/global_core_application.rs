use crate::gen::agent::{GlobalCoreApplicationDispatchFunctions, GlobalCoreApplicationDispatcher};
use crate::model::ApplicationEntityEvent;
use si_cea::agent::dispatch::prelude::*;

#[derive(Clone)]
pub struct GlobalCoreApplicationDispatchFunctionsImpl;

#[async_trait]
impl GlobalCoreApplicationDispatchFunctions for GlobalCoreApplicationDispatchFunctionsImpl {
    type EntityEvent = ApplicationEntityEvent;

    async fn create(
        _mqtt_client: &MqttClient,
        _entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async { Ok(()) }.instrument(debug_span!("create")).await
    }

    async fn sync(
        _mqtt_client: &MqttClient,
        _entity_event: &mut Self::EntityEvent,
    ) -> CeaResult<()> {
        async { Ok(()) }.instrument(debug_span!("sync")).await
    }
}

pub fn dispatcher() -> GlobalCoreApplicationDispatcher<GlobalCoreApplicationDispatchFunctionsImpl> {
    GlobalCoreApplicationDispatcher::<GlobalCoreApplicationDispatchFunctionsImpl>::new()
}
