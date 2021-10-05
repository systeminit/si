use color_eyre::Result;
use si_cyclone::{
    app, telemetry, uds::UDSIncomingStream, Config, ConfigBuilderError, IncomingStream,
};
use std::convert::{TryFrom, TryInto};
use tracing::{debug, info};

mod cli;

#[tokio::main]
async fn main() -> Result<()> {
    color_eyre::install()?;
    telemetry::init()?;
    let config = cli::parse().try_into()?;

    Server::init(config).await?.run().await
}

struct Server {
    config: Config,
}

impl Server {
    async fn init(config: Config) -> Result<Self> {
        Ok(Self { config })
    }

    async fn run(self) -> Result<()> {
        match self.config.incoming_stream() {
            IncomingStream::HTTPSocket(socket_addr) => {
                info!("binding to HTTP socket; socket_addr={}", &socket_addr);
                axum::Server::bind(&socket_addr)
                    .serve(app(self.config).into_make_service())
                    .await?;
            }
            IncomingStream::UnixDomainSocket(path) => {
                info!("binding to Unix domain socket; path={}", path.display());
                axum::Server::builder(UDSIncomingStream::create(path).await?)
                    .serve(app(self.config).into_make_service())
                    .await?;
            }
        }

        Ok(())
    }
}

impl TryFrom<cli::Args> for Config {
    type Error = ConfigBuilderError;

    fn try_from(args: cli::Args) -> Result<Self, Self::Error> {
        let mut builder = Self::builder();

        if let Some(socket_addr) = args.bind_addr {
            builder.incoming_stream(IncomingStream::HTTPSocket(socket_addr));
        }
        if let Some(pathbuf) = args.bind_uds {
            builder.incoming_stream(IncomingStream::UnixDomainSocket(pathbuf));
        }

        if args.enable_ping {
            builder.enable_ping(true);
        } else if args.disable_ping {
            builder.enable_ping(false);
        }

        if args.enable_resolver {
            builder.enable_resolver(true);
        } else if args.disable_resolver {
            builder.enable_resolver(false);
        }

        builder.build()
    }
}
