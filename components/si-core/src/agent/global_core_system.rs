use crate::gen::agent::{GlobalCoreSystemDispatchFunctions, GlobalCoreSystemDispatcherBuilder};
use si_agent::prelude::*;
use si_cea::agent::prelude::*;

pub struct GlobalCoreSystemDispatchFunctionsImpl;

#[async_trait]
impl GlobalCoreSystemDispatchFunctions for GlobalCoreSystemDispatchFunctionsImpl {
    async fn create(
        _transport: &si_agent::Transport,
        _stream_header: si_agent::Header,
        _entity_event: &mut crate::protobuf::SystemEntityEvent,
    ) -> si_agent::AgentResult<()> {
        async { Ok(()) }.instrument(debug_span!("create")).await
    }

    async fn sync(
        _transport: &si_agent::Transport,
        _stream_header: si_agent::Header,
        _entity_event: &mut crate::protobuf::SystemEntityEvent,
    ) -> si_agent::AgentResult<()> {
        async { Ok(()) }.instrument(debug_span!("sync")).await
    }
}

pub fn dispatcher_builder(
) -> GlobalCoreSystemDispatcherBuilder<GlobalCoreSystemDispatchFunctionsImpl> {
    GlobalCoreSystemDispatcherBuilder::new()
}
