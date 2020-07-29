use crate::gen::agent::{
    AwsEksKubernetesKubernetesServiceDispatchFunctions,
    AwsEksKubernetesKubernetesServiceDispatcherBuilder,
};
use si_agent::prelude::*;
use si_cea::agent::prelude::*;

pub struct AwsEksKubernetesKubernetesServiceDispatchFunctionsImpl;

#[async_trait]
impl AwsEksKubernetesKubernetesServiceDispatchFunctions
    for AwsEksKubernetesKubernetesServiceDispatchFunctionsImpl
{
    async fn create(
        _transport: &si_agent::Transport,
        _stream_header: si_agent::Header,
        _entity_event: &mut crate::protobuf::KubernetesServiceEntityEvent,
    ) -> si_agent::AgentResult<()> {
        async { Ok(()) }.instrument(debug_span!("create")).await
    }

    async fn sync(
        _transport: &si_agent::Transport,
        _stream_header: si_agent::Header,
        _entity_event: &mut crate::protobuf::KubernetesServiceEntityEvent,
    ) -> si_agent::AgentResult<()> {
        async { Ok(()) }.instrument(debug_span!("sync")).await
    }
}

pub fn dispatcher_builder() -> AwsEksKubernetesKubernetesServiceDispatcherBuilder<
    AwsEksKubernetesKubernetesServiceDispatchFunctionsImpl,
> {
    AwsEksKubernetesKubernetesServiceDispatcherBuilder::new()
}
