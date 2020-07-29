use crate::gen::agent::{GlobalCoreEdgeDispatchFunctions, GlobalCoreEdgeDispatcherBuilder};
use si_agent::prelude::*;
use si_cea::agent::prelude::*;

pub struct GlobalCoreEdgeDispatchFunctionsImpl;

#[async_trait]
impl GlobalCoreEdgeDispatchFunctions for GlobalCoreEdgeDispatchFunctionsImpl {
    async fn create(
        _transport: &si_agent::Transport,
        _stream_header: si_agent::Header,
        _entity_event: &mut crate::protobuf::EdgeEntityEvent,
    ) -> si_agent::AgentResult<()> {
        async { Ok(()) }.instrument(debug_span!("create")).await
    }

    async fn sync(
        _transport: &si_agent::Transport,
        _stream_header: si_agent::Header,
        _entity_event: &mut crate::protobuf::EdgeEntityEvent,
    ) -> si_agent::AgentResult<()> {
        async { Ok(()) }.instrument(debug_span!("sync")).await
    }
}

pub fn dispatcher_builder() -> GlobalCoreEdgeDispatcherBuilder<GlobalCoreEdgeDispatchFunctionsImpl>
{
    GlobalCoreEdgeDispatcherBuilder::new()
}
