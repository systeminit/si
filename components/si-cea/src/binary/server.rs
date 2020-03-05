pub mod prelude {
    pub use crate::{EntityEvent as _, gen_server_binary};
    pub use crate::Dispatch as _;
    pub use crate::MigrateComponent as _;
}

use tracing_subscriber::{EnvFilter, FmtSubscriber};

use crate::error::Result;

pub fn setup_tracing() -> Result<()> {
    let subscriber = FmtSubscriber::builder()
        .without_time()
        .with_ansi(true)
        .compact()
        .with_env_filter(EnvFilter::from_default_env())
        .finish();

    tracing::subscriber::set_global_default(subscriber)?;

    Ok(())
}

#[macro_export]
macro_rules! gen_server_binary {
    (
        name: $name: tt, 
        dispatcher: $dispatcher: ident, 
        component: $component: ident,
        entity_event: $entity_event: ident, 
        service: $service: ident, 
        server: $server: ident
    ) => {
        println!("*** Starting {} ***", $name);
        si_cea::binary::server::setup_tracing()?;

        println!("*** Loading settings ***");
        let settings = si_settings::Settings::new()?;

        println!("*** Connecting to the database ***");
        let db = si_data::Db::new(&settings)?;

        // Migrate
        println!("*** Migrating components ***");
        $component::migrate(&db).await?;

        // Agent Server
        println!("*** Spawning the Agent Server ***");
        let mut agent_dispatch = $dispatcher(si_cea::AgentDispatch::new());
        agent_dispatch.setup(&db).await?;
        tokio::spawn(async move {
            // Dispatcher
            let mut agent_server = si_cea::AgentServer::new("SSH Key", agent_dispatch);
            agent_server.run().await
        });

        // Finalizer
        println!("*** Spawning the Agent Finalizer ***");
        let finalizer_db = db.clone();
        tokio::spawn(async move {
            let mut finalizer = si_cea::AgentFinalizer::new(finalizer_db, $entity_event::type_name());
            finalizer.run::<$entity_event>().await
        });

        // GRPC Services
        let agent_client = si_cea::AgentClient::new().await?;

        let service = $service::new(db.clone(), agent_client);

        let listen_string = format!("0.0.0.0:{}", settings.service.port);

        let addr = listen_string.parse().unwrap();

        println!("--> {} service listening on {} <--", $name, addr);
        println!("--> Let us make stuff <--");

        tonic::transport::Server::builder()
            .add_service($server::new(service))
            .serve(addr)
            .await?;
    }
}
