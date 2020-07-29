use anyhow::Context;
use si_agent::{prelude::*, Dispatchable};
use si_cea::binary::server::prelude::*;
use si_kubernetes::{
    agent::{aws_eks_kubernetes_kubernetes_deployment, aws_eks_kubernetes_kubernetes_service},
    gen::{
        finalize::{kubernetes_deployment_entity_event, kubernetes_service_entity_event},
        service::{Server, Service},
    },
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server_name = "kubernetes";

    println!("*** Starting {} ***", server_name);
    setup_tracing("si-kubernetes").context("failed to setup tracing")?;

    println!("*** Loading settings ***");
    let settings = si_settings::Settings::new()?;

    println!("*** Connecting to the database ***");
    let db = si_data::Db::new(&settings).context("failed to connect to the database")?;

    println!("*** Initializing service ***");
    let service = Service::new(db.clone());

    println!("*** Running service migrations ***");
    service.migrate().await?;

    spawn_finalized_listener(server_name, settings.vernemq_server_uri(), db.clone())
        .await
        .context("failed to spawn finalized listener")?;

    spawn_agent(server_name, settings.vernemq_server_uri(), &db)
        .await
        .context("failed to spawn agent")?;

    spawn_service(server_name, service, settings.service.port)
        .await
        .context("failed to spawn service")?;

    Ok(())
}

/// Configures and spawns a `FinalizedListener` which subscribes to and finalizes objects,
/// according to the objects' implementation details.
async fn spawn_finalized_listener(
    server_name: &str,
    transport_server_uri: impl Into<String>,
    db: Db,
) -> anyhow::Result<()> {
    let mut listener_builder = FinalizedListener::builder(server_name, transport_server_uri, db);
    listener_builder.finalizer(kubernetes_deployment_entity_event::finalizer()?);
    listener_builder.finalizer(kubernetes_service_entity_event::finalizer()?);
    let listener = listener_builder.build().await?;

    tokio::spawn(listener.run());

    Ok(())
}

/// Configure and spawn an `Agent` instance which subscribes to and takes action on dispatched
/// entity events.
async fn spawn_agent(
    server_name: &str,
    transport_server_uri: impl Into<String>,
    db: &Db,
) -> anyhow::Result<()> {
    println!("*** Spawning the Agent ***");
    let mut agent_builder = Agent::builder(
        server_name,
        transport_server_uri,
        si_agent::TEMP_AGENT_ID,
        si_agent::TEMP_AGENT_INSTALLATION_ID,
    );
    agent_builder.dispatcher(
        build_dispatcher(
            &db,
            aws_eks_kubernetes_kubernetes_deployment::dispatcher_builder(),
        )
        .await?,
    );
    agent_builder.dispatcher(
        build_dispatcher(
            &db,
            aws_eks_kubernetes_kubernetes_service::dispatcher_builder(),
        )
        .await?,
    );
    let agent = agent_builder.build().await?;

    tokio::spawn(agent.run());

    Ok(())
}

/// Configures and spawns a gRPC service to handle service requests.
async fn spawn_service(server_name: &str, service: Service, port: u16) -> anyhow::Result<()> {
    let addr = format!("0.0.0.0:{}", port).parse().unwrap();

    println!("--> {} service listening on {} <--", server_name, addr);
    tonic::transport::Server::builder()
        .add_service(Server::new(service))
        .serve(addr)
        .await?;

    Ok(())
}

/// Builds a dispatcher implementation from a given `DispatchBuilder`.
async fn build_dispatcher(
    db: &Db,
    mut dispatch_builder: impl DispatchBuilder,
) -> anyhow::Result<impl Dispatchable> {
    let dispatch_key = dispatch_key_for(db, &dispatch_builder).await?;
    dispatch_builder.dispatch_key(dispatch_key);

    Ok(dispatch_builder.build()?)
}

/// Resolves and builds a `DispatchKey` for the given `DispatchBuilder`.
///
/// **Note**: This function is largely a temporary measure as an `Agent` is being created and set
/// to run in the main service, as opposed to an Agent deployment that could happen beyond the
/// network boundary of System Initiative's core service network. In this future state, the
/// integration and integration service identifiers would be configured and provided the `Settings`
/// interface. In this way, we avoid Agents having awareness or the power to use a database
/// connection within their implementations.
async fn dispatch_key_for(db: &Db, builder: &impl DispatchBuilder) -> anyhow::Result<DispatchKey> {
    let integration_name = builder.integration_name();
    let integration_service_name = builder.integration_service_name();
    let object_type = builder.object_type();

    let integration: si_account::Integration = db
        .lookup_by_natural_key(format!("global:integration:{}", integration_name))
        .await?;
    let integration_service_lookup_id = format!(
        "{}:integration_service:{}",
        integration
            .id
            .as_ref()
            .ok_or_else(|| si_data::DataError::RequiredField("id".to_string()))?,
        integration_service_name
    );
    let integration_service: si_account::IntegrationService = db
        .lookup_by_natural_key(integration_service_lookup_id)
        .await?;

    Ok(DispatchKey::new(
        integration.id()?,
        integration_service.id()?,
        object_type,
    ))
}
