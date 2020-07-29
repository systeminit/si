use crate::gen::agent::{
    GlobalCoreApplicationDispatchFunctions, GlobalCoreApplicationDispatcherBuilder,
};
use si_agent::prelude::*;
use si_cea::agent::prelude::*;

pub struct GlobalCoreApplicationDispatchFunctionsImpl;

#[async_trait]
impl GlobalCoreApplicationDispatchFunctions for GlobalCoreApplicationDispatchFunctionsImpl {
    async fn create(
        _transport: &si_agent::Transport,
        _stream_header: si_agent::Header,
        _entity_event: &mut crate::protobuf::ApplicationEntityEvent,
    ) -> si_agent::AgentResult<()> {
        async { Ok(()) }.instrument(debug_span!("create")).await
    }

    async fn sync(
        _transport: &si_agent::Transport,
        _stream_header: si_agent::Header,
        _entity_event: &mut crate::protobuf::ApplicationEntityEvent,
    ) -> si_agent::AgentResult<()> {
        async { Ok(()) }.instrument(debug_span!("sync")).await
    }
}

pub fn dispatcher_builder(
) -> GlobalCoreApplicationDispatcherBuilder<GlobalCoreApplicationDispatchFunctionsImpl> {
    GlobalCoreApplicationDispatcherBuilder::new()
}
