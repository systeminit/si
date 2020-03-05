use anyhow::Result;
use si_cea::binary::server::prelude::*;
use si_ssh_key::{Component, Dispatcher, EntityEvent, Service, SshKeyServer};
use tokio;

#[tokio::main]
async fn main() -> Result<()> {
    gen_server_binary!(
        name: "SSH Key",
        dispatcher: Dispatcher,
        component: Component,
        entity_event: EntityEvent,
        service: Service,
        server: SshKeyServer
    );
    Ok(())
}
