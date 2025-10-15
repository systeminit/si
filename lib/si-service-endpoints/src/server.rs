use std::sync::Arc;

use telemetry::prelude::*;
use tokio_util::sync::CancellationToken;

use crate::{
    DefaultServiceEndpoints,
    Result,
    ServiceEndpointsConfig,
};

pub struct EndpointsServer {
    service: Arc<DefaultServiceEndpoints>,
    config: ServiceEndpointsConfig,
    shutdown_token: CancellationToken,
}

impl EndpointsServer {
    pub fn new(
        service: Arc<DefaultServiceEndpoints>,
        config: ServiceEndpointsConfig,
        shutdown_token: CancellationToken,
    ) -> Self {
        Self {
            service,
            config,
            shutdown_token,
        }
    }

    pub async fn run(self) -> Result<()> {
        if !self.config.enabled {
            return Ok(());
        }

        let app = crate::axum_integration::create_router(self.service.clone(), &self.config);

        let listener = tokio::net::TcpListener::bind(&self.config.bind_address).await?;
        let actual_addr = listener.local_addr()?;

        info!(
            service = self.service.service_name(),
            address = %actual_addr,
            health_endpoint = self.config.health_endpoint,
            config_endpoint = self.config.config_endpoint,
            "service endpoints listening"
        );

        axum::Server::from_tcp(listener.into_std()?)
            .map_err(std::io::Error::other)?
            .serve(app.into_make_service())
            .with_graceful_shutdown(async move {
                self.shutdown_token.cancelled().await;
            })
            .await
            .map_err(std::io::Error::other)?;
        Ok(())
    }
}
