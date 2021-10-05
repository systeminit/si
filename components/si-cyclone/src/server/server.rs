use super::{routes, Config, IncomingStream, UDSIncomingStream, UDSIncomingStreamError};
use thiserror::Error;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing::info;

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("hyper server error")]
    Hyper(#[from] hyper::Error),
    #[error("UDS incoming stream error")]
    UDS(#[from] UDSIncomingStreamError),
}

pub struct Server {
    config: Config,
}

impl Server {
    pub async fn init(config: Config) -> Result<Self, ServerError> {
        Ok(Self { config })
    }

    pub async fn run(self) -> Result<(), ServerError> {
        let routes = routes(&self.config)
            // TODO(fnichol): customize http tracing further, using:
            // https://docs.rs/tower-http/0.1.1/tower_http/trace/index.html
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::default().include_headers(true)),
            );

        match self.config.incoming_stream() {
            IncomingStream::HTTPSocket(socket_addr) => {
                info!("binding to HTTP socket; socket_addr={}", &socket_addr);
                axum::Server::bind(&socket_addr)
                    .serve(routes.into_make_service())
                    .await?;
            }
            IncomingStream::UnixDomainSocket(path) => {
                info!("binding to Unix domain socket; path={}", path.display());
                axum::Server::builder(UDSIncomingStream::create(path).await?)
                    .serve(routes.into_make_service())
                    .await?;
            }
        }

        Ok(())
    }
}
