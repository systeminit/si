use std::sync::Arc;

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

        let app = crate::axum_integration::create_router(self.service, &self.config);

        axum::Server::bind(&self.config.bind_address)
            .serve(app.into_make_service())
            .with_graceful_shutdown(async move {
                self.shutdown_token.cancelled().await;
            })
            .await
            .map_err(std::io::Error::other)?;
        Ok(())
    }
}

