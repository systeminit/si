use crate::gen::agent::{GlobalCoreServiceDispatchFunctions, GlobalCoreServiceDispatcherBuilder};
use si_agent::prelude::*;
use si_cea::agent::prelude::*;

#[derive(Clone)]
pub struct GlobalCoreServiceDispatchFunctionsImpl;

#[async_trait]
impl GlobalCoreServiceDispatchFunctions for GlobalCoreServiceDispatchFunctionsImpl {
    async fn create(
        _transport: &si_agent::Transport,
        _stream_header: si_agent::Header,
        _entity_event: &mut crate::protobuf::ServiceEntityEvent,
    ) -> si_agent::AgentResult<()> {
        async { Ok(()) }.instrument(debug_span!("create")).await
    }

    async fn deploy(
        _transport: &si_agent::Transport,
        _stream_header: si_agent::Header,
        _entity_event: &mut crate::protobuf::ServiceEntityEvent,
    ) -> si_agent::AgentResult<()> {
        async { Ok(()) }.instrument(debug_span!("deploy")).await
    }

    async fn sync(
        _transport: &si_agent::Transport,
        _stream_header: si_agent::Header,
        _entity_event: &mut crate::protobuf::ServiceEntityEvent,
    ) -> si_agent::AgentResult<()> {
        async { Ok(()) }.instrument(debug_span!("sync")).await
    }
}

pub fn dispatcher_builder(
) -> GlobalCoreServiceDispatcherBuilder<GlobalCoreServiceDispatchFunctionsImpl> {
    GlobalCoreServiceDispatcherBuilder::new()
}
