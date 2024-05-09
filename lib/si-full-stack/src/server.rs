use si_layer_cache::db::LayerDbConfig;
use telemetry::prelude::*;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use veritech_server::VeritechShutdownHandle;

use crate::{error::SiFullStackResult, Config, SiFullStackError};

#[derive(Debug)]
pub(crate) struct Server {
    config: Config,
    cancel_token: CancellationToken,
    shutdown_rx: tokio::sync::oneshot::Receiver<bool>,
    task_tracker: TaskTracker,
}

impl Server {
    fn new(config: Config, shutdown_rx: tokio::sync::oneshot::Receiver<bool>) -> Self {
        let task_tracker = TaskTracker::new();
        let cancel_token = CancellationToken::new();

        Self {
            config,
            cancel_token,
            shutdown_rx,
            task_tracker,
        }
    }

    pub(crate) async fn run(
        config: Config,
        shutdown_rx: tokio::sync::oneshot::Receiver<bool>,
    ) -> SiFullStackResult<()> {
        // Create our new Full Stack Server
        let server = Server::new(config, shutdown_rx);

        // Initialize crypto
        server.initialize_sodium_oxide()?;

        // Start veritech
        let (veritech_join_handle, veritech_shutdown_handle) = server.start_veritech().await?;

        // Start rebaser
        let rebaser_join_handle = server.start_rebaser().await?;

        // Start pinga
        let pinga_join_handle = server.start_pinga().await?;

        // Run the
        server.task_tracker.close();

        tokio::select! {
            result = veritech_join_handle => {
                error!(?result, "Veritech exited from a full stack server before service shutdown; bug!");
            }
            result = rebaser_join_handle => {
                error!(?result, "Rebaser exited from a full stack server before service shutdown; bug!");
            }
            () = server.cancel_token.cancelled() => {
                veritech_shutdown_handle.shutdown().await;
            }
        }

        server.task_tracker.wait().await;

        Ok(())
    }

    fn initialize_sodium_oxide(&self) -> SiFullStackResult<()> {
        debug!("initializing sodium oxide");
        sodiumoxide::init().map_err(|_| SiFullStackError::SodiumOxideInit)?;
        Ok(())
    }

    // Sort these
    async fn start_rebaser(
        &self,
    ) -> SiFullStackResult<tokio::task::JoinHandle<Result<(), rebaser_server::ServerError>>> {
        let config: rebaser_server::Config = rebaser_server::ConfigFile::default().try_into()?;

        let server = rebaser_server::Server::from_services(
            config.instance_id(),
            services_context,
            self.cancel_token.clone(),
        )?;
        let join_handle = self.task_tracker.spawn(server.run());
        Ok(join_handle);
    }

    async fn start_pinga(
        &self,
    ) -> SiFullStackResult<tokio::task::JoinHandle<Result<(), rebaser_server::ServerError>>> {
    let config: pinga_server::Config = {
        let mut layer_db_config = LayerDbConfig::default_for_service("pinga");
        layer_db_config.pg_pool_config = self.config.layer_cache_pg_pool();

        let mut config_file = pinga_server::ConfigBuilder::default()
                .pg_pool(self.config.pg())
                .layer_db_config()
        pinga_server::detect_and_configure_development(&mut config_file)?;
        config_file.try_into()?
    };
    config.

    let server = pinga_server::Server::from_config(
        config_file, self.cancel_token.clone(), self.task_tracker.clone()
    )
    .wrap_err("failed to create Pinga server")?;

    Ok(server)
    }


    async fn start_veritech(
        &self,
    ) -> SiFullStackResult<(
        tokio::task::JoinHandle<Result<(), veritech_server::ServerError>>,
        VeritechShutdownHandle,
    )> {
        let veritech_config: veritech_server::Config = {
            let mut config_file = veritech_server::ConfigFile {
                nats: self.config.nats(),
                ..Default::default()
            };
            veritech_server::detect_and_configure_development(&mut config_file)?;
            config_file.try_into()?
        };
        let server = veritech_server::Server::for_cyclone_uds(veritech_config).await?;
        let shutdown_handle = server.shutdown_handle();
        let join_handle = self.task_tracker.spawn(server.run());
        Ok((join_handle, shutdown_handle))
    }
}
