// #![warn(
//     clippy::unwrap_in_result,
//     clippy::unwrap_used,
//     clippy::panic,
//     clippy::missing_panics_doc,
//     clippy::panic_in_result_fn
// )]
// #![allow(clippy::missing_errors_doc)]

use futures::TryStreamExt;
use std::{io, net::ToSocketAddrs, result, sync::Arc};

use serde::{Deserialize, Serialize};
use si_std::SensitiveString;
use spicedb_client::{builder::WriteRelationshipsRequestBuilder, SpicedbClient};
use spicedb_grpc::authzed::api::v1::{relationship_update::Operation, WriteRelationshipsRequest};
use telemetry::prelude::*;
use thiserror::Error;
use types::Relationships;
use url::Url;

mod types;

pub use types::{Permission, ReadSchemaResponse, Relationship, ZedToken};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum Error {
    #[error("error connecting to spicedb at {1}: {0}")]
    Connection(#[source] spicedb_client::result::Error, Url),
    #[error("spicedb endpoint has no host part: {0}")]
    EndpointNoHost(Url),
    #[error("cannot determine spicedb endpoint port number: {0}")]
    EndpointUnknownPort(Url),
    #[error("GRPC streaming error: {0}")]
    GRPC(#[source] spicedb_client::result::Error),
    #[error("error resolving ip addr for spicedb endpoint hostname: {0}")]
    ResolveHostname(#[source] io::Error),
    #[error("resolved hostname returned no entries")]
    ResolveHostnameNoEntries,
    #[error("spicedb client error: {0}")]
    SpiceDb(#[from] spicedb_client::result::Error),
    #[error("tokio task join error: {0}")]
    TokioJoin(#[from] tokio::task::JoinError),
}

pub type SpiceDbError = Error;

type Result<T> = result::Result<T, Error>;

pub struct Client {
    inner: SpicedbClient,
    metadata: Arc<ConnectionMetadata>,
}

impl Client {
    #[instrument(
        name = "spicedb_client::new",
        level = "debug",
        skip_all,
        fields(
            db.connection_string = Empty,
            db.system = Empty,
            network.peer.address = Empty,
            network.protocol.name = Empty,
            network.transport = Empty,
            otel.kind = SpanKind::Client.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = Empty,
            server.port = Empty,
        ),
    )]
    pub async fn new(config: &SpiceDbConfig) -> Result<Self> {
        let span = current_span_for_instrument_at!("debug");

        let db_system = "spicedb";
        let db_connection_string = config.endpoint.to_string();

        let network_transport = "ip_tcp";
        let network_protocol_name = "spicedb";

        let server_port = config
            .endpoint
            .port_or_known_default()
            .ok_or(Error::EndpointUnknownPort(config.endpoint.clone()))
            .map_err(|err| span.record_err(err))?;

        let server_address = match config.endpoint.host() {
            Some(url::Host::Domain(domain)) => {
                let domain = domain.to_owned();
                // Resolve hostname to an IP address
                tokio::task::spawn_blocking(move || {
                    (format!("{domain}:{server_port}"))
                        .to_socket_addrs()
                        .map_err(Error::ResolveHostname)
                        .and_then(|mut iter| iter.next().ok_or(Error::ResolveHostnameNoEntries))
                        .map(|socket_addr| socket_addr.ip().to_string())
                })
                .await
                .map_err(|err| span.record_err(err))?
                .map_err(|err| span.record_err(err))?
            }
            Some(url::Host::Ipv4(addr)) => addr.to_string(),
            Some(url::Host::Ipv6(addr)) => addr.to_string(),
            None => return Err(span.record_err(Error::EndpointNoHost(config.endpoint.clone()))),
        };

        let metadata = ConnectionMetadata {
            db_system,
            db_connection_string,
            network_peer_address: server_address.clone(),
            network_protocol_name,
            network_transport,
            server_address,
            server_port,
        };

        span.record(
            "db.connection_string",
            metadata.db_connection_string.as_str(),
        );
        span.record("db.system", metadata.db_system);
        span.record(
            "network.peer.address",
            metadata.network_peer_address.as_str(),
        );
        span.record("network.protocol.name", metadata.network_protocol_name);
        span.record("network.transport", metadata.network_transport);
        span.record("server.address", metadata.server_address.as_str());
        span.record("server.port", metadata.server_port);

        let inner = SpicedbClient::from_url_and_preshared_key(
            config.endpoint.to_string(),
            config.preshared_key.as_str(),
        )
        .await
        .map_err(|err| span.record_err(Error::Connection(err, config.endpoint.clone())))?;

        span.record_ok();
        Ok(Self {
            inner,
            metadata: Arc::new(metadata),
        })
    }

    #[instrument(
        name = "spicedb_client.read_schema",
        level = "debug",
        skip_all,
        fields(
            db.connection_string = %self.metadata.db_connection_string(),
            db.system = %self.metadata.db_system(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Client.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        ),
    )]
    pub async fn read_schema(&mut self) -> Result<ReadSchemaResponse> {
        let span = current_span_for_instrument_at!("debug");

        let resp = self
            .inner
            .read_schema()
            .await
            .map_err(|err| span.record_err(Error::SpiceDb(err)))?
            .into();

        span.record_ok();
        Ok(resp)
    }

    #[instrument(
        name = "spicedb_client.write_schema",
        level = "debug",
        skip_all,
        fields(
            db.connection_string = %self.metadata.db_connection_string(),
            db.system = %self.metadata.db_system(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Client.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        ),
    )]
    pub async fn write_schema(&mut self, schema: impl ToString) -> Result<Option<ZedToken>> {
        let span = current_span_for_instrument_at!("debug");

        let resp = self
            .inner
            .write_schema(schema)
            .await
            .map_err(|err| span.record_err(Error::SpiceDb(err)))?
            .written_at
            .map(|value| value.into());

        span.record_ok();
        Ok(resp)
    }

    #[instrument(
        name = "spicedb_client.read_relationships",
        level = "debug",
        skip_all,
        fields(
            db.connection_string = %self.metadata.db_connection_string(),
            db.system = %self.metadata.db_system(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Client.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        ),
    )]
    pub async fn read_relationship(&mut self, relationship: Relationship) -> Result<Relationships> {
        let span = current_span_for_instrument_at!("debug");
        let mut relationships = vec![];

        let results: result::Result<Vec<_>, _> = self
            .inner
            .read_relationships(relationship.into_request())
            .await?
            .try_collect()
            .await;

        for r in results.map_err(|e| span.record_err(Error::GRPC(e.into())))? {
            if let Some(relationship) = r.relationship {
                relationships.push(relationship.into());
            }
        }

        span.record_ok();
        Ok(relationships)
    }

    #[instrument(
        name = "spicedb_client.create_relationships",
        level = "debug",
        skip_all,
        fields(
            db.connection_string = %self.metadata.db_connection_string(),
            db.system = %self.metadata.db_system(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Client.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        ),
    )]
    pub async fn create_relationships(
        &mut self,
        relationships: Relationships,
    ) -> Result<Option<ZedToken>> {
        self.update_relationships(relationships, Operation::Create)
            .await
    }

    #[instrument(
        name = "spicedb_client.delete_relationships",
        level = "debug",
        skip_all,
        fields(
            db.connection_string = %self.metadata.db_connection_string(),
            db.system = %self.metadata.db_system(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Client.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        ),
    )]
    pub async fn delete_relationships(
        &mut self,
        relationships: Relationships,
    ) -> Result<Option<ZedToken>> {
        self.update_relationships(relationships, Operation::Delete)
            .await
    }

    async fn update_relationships(
        &mut self,
        relationships: Relationships,
        operation: Operation,
    ) -> Result<Option<ZedToken>> {
        let span = current_span_for_instrument_at!("debug");

        let request: WriteRelationshipsRequest = WriteRelationshipsRequest::new(
            relationships
                .into_iter()
                .map(|r| r.into_relationship_update(operation)),
        );

        let resp = self
            .inner
            .write_relationships(request)
            .await
            .map_err(|err| span.record_err(Error::SpiceDb(err)))?
            .written_at
            .map(|value| value.into());

        span.record_ok();
        Ok(resp)
    }

    #[instrument(
        name = "spicedb_client.check_permissions",
        level = "debug",
        skip_all,
        fields(
            db.connection_string = %self.metadata.db_connection_string(),
            db.system = %self.metadata.db_system(),
            network.peer.address = self.metadata.network_peer_address(),
            network.protocol.name = self.metadata.network_protocol_name(),
            network.transport = self.metadata.network_transport(),
            otel.kind = SpanKind::Client.as_str(),
            otel.status_code = Empty,
            otel.status_message = Empty,
            server.address = self.metadata.server_address(),
            server.port = self.metadata.server_port(),
        ),
    )]
    pub async fn check_permissions(&mut self, permission: Permission) -> Result<bool> {
        let span = current_span_for_instrument_at!("debug");

        let resp = self
            .inner
            .check_permission(permission.into_request())
            .await
            .map_err(|err| span.record_err(Error::SpiceDb(err)))?
            .permissionship;

        span.record_ok();
        Ok(Permission::has_permission(resp))
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SpiceDbConfig {
    pub endpoint: Url,
    pub preshared_key: SensitiveString,
}

impl Default for SpiceDbConfig {
    fn default() -> Self {
        Self {
            endpoint: Url::parse("http://localhost:50051").expect("string is a valid URL"),
            preshared_key: SensitiveString::from("hobgoblin"),
        }
    }
}

#[derive(Clone, Debug)]
pub struct ConnectionMetadata {
    db_system: &'static str,
    db_connection_string: String,
    network_peer_address: String,
    network_protocol_name: &'static str,
    network_transport: &'static str,
    server_address: String,
    server_port: u16,
}

impl ConnectionMetadata {
    pub fn db_system(&self) -> &str {
        self.db_system
    }

    pub fn db_connection_string(&self) -> &str {
        &self.db_connection_string
    }

    pub fn network_peer_address(&self) -> &str {
        &self.network_peer_address
    }

    pub fn network_protocol_name(&self) -> &str {
        self.network_protocol_name
    }

    pub fn network_transport(&self) -> &str {
        self.network_transport
    }

    pub fn server_address(&self) -> &str {
        &self.server_address
    }

    pub fn server_port(&self) -> u16 {
        self.server_port
    }
}
