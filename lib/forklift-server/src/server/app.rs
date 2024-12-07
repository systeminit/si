use std::{future::Future, io, sync::Arc};

use audit_database::AuditDatabaseContext;
use si_data_nats::{jetstream::Context, ConnectionMetadata};
use telemetry::prelude::*;
use thiserror::Error;
use tokio_util::sync::CancellationToken;

mod audit_logs;
mod billing_events;

pub(crate) use audit_logs::AuditLogsAppSetupError;
pub(crate) use billing_events::BillingEventsAppSetupError;

#[derive(Debug, Error)]
pub enum AppSetupError {
    #[error("audit logs app setup: {0}")]
    AuditLogsAppSetup(#[from] AuditLogsAppSetupError),
    #[error("billing events app setup: {0}")]
    BillingEventsAppSetup(#[from] BillingEventsAppSetupError),
}

type Result<T> = std::result::Result<T, AppSetupError>;

#[instrument(
    name = "forklift.init.app.audit_logs",
    level = "info",
    skip_all,
    fields(durable_consumer_name)
)]
pub(crate) async fn audit_logs(
    jetstream_context: Context,
    durable_consumer_name: String,
    connection_metadata: Arc<ConnectionMetadata>,
    audit_database_context: AuditDatabaseContext,
    insert_concurrency_limit: usize,
    token: CancellationToken,
) -> Result<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>> {
    Ok(audit_logs::build_and_run(
        jetstream_context,
        durable_consumer_name,
        connection_metadata,
        audit_database_context,
        insert_concurrency_limit,
        token,
    )
    .await?)
}

#[instrument(
    name = "forklift.init.app.billing_events",
    level = "info",
    skip_all,
    fields(durable_consumer_name)
)]
pub(crate) async fn billing_events(
    jetstream_context: Context,
    durable_consumer_name: String,
    connection_metadata: Arc<ConnectionMetadata>,
    concurrency_limit: usize,
    data_warehouse_stream_name: Option<&str>,
    token: CancellationToken,
) -> Result<Box<dyn Future<Output = io::Result<()>> + Unpin + Send>> {
    Ok(billing_events::build_and_run(
        jetstream_context,
        durable_consumer_name,
        connection_metadata,
        concurrency_limit,
        data_warehouse_stream_name,
        token,
    )
    .await?)
}
