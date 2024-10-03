//! This create provides centralized logic for working with the billing events NATS Jetstream stream.

#![warn(
    bad_style,
    clippy::missing_panics_doc,
    clippy::panic,
    clippy::panic_in_result_fn,
    clippy::unwrap_in_result,
    clippy::unwrap_used,
    dead_code,
    improper_ctypes,
    missing_debug_implementations,
    missing_docs,
    no_mangle_generic_items,
    non_shorthand_field_patterns,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    unconditional_recursion,
    unreachable_pub,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Error;
use si_data_nats::{
    async_nats::jetstream::{
        context::{CreateStreamError, PublishError},
        stream::{Config, DiscardPolicy, RetentionPolicy, Stream},
    },
    jetstream,
};
use si_events::{
    ChangeSetId, ChangeSetStatus, ComponentId, FuncRunId, SchemaId, SchemaVariantId, UserPk,
    WorkspacePk, WorkspaceSnapshotAddress,
};
use telemetry::prelude::*;
use telemetry_nats::propagation;
use thiserror::Error;

const STREAM_NAME: &str = "BILLING_EVENTS";
const WORKSPACE_UPDATE_SUBJECT: &str = "billing.workspace_update";

#[allow(missing_docs)]
#[remain::sorted]
#[derive(Debug, Error)]
pub enum BillingEventsError {
    #[error("create stream error: {0}")]
    CreateStream(#[from] CreateStreamError),
    #[error("publish error: {0}")]
    Publish(#[from] PublishError),
    #[error("serde json error: {0}")]
    SerdeJson(#[from] Error),
}

type BillingEventsResult<T> = Result<T, BillingEventsError>;

/// The kind of the [event](BillingEvent) published.
#[remain::sorted]
#[derive(Debug, Clone, Serialize, Deserialize, Copy)]
pub enum BillingEventKind {
    /// An event that is published when a change set's status is updated.
    ChangeSetStatusUpdate,
    /// An event that is published when the HEAD change set's pointer is updated.
    HeadChangeSetPointerUpdate,
    /// An event that is published when a resource is created.
    ResourceCreate,
    /// An event that is published when a resource is deleted.
    ResourceDelete,
}

/// A billing event published for a workspace and change set over a NATS Jetstream stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingEvent {
    /// The ID of the workspace.
    pub workspace_id: WorkspacePk,
    /// The ID of the change set.
    pub change_set_id: ChangeSetId,
    /// The UTC Timestamp when it was created
    pub event_timestamp: DateTime<Utc>,

    /// The specific snapshot that the change set is pointing at.
    pub workspace_snapshot_address: WorkspaceSnapshotAddress,
    /// The status of the change set.
    pub change_set_status: ChangeSetStatus,
    /// The user who requested the update (if any)
    pub merge_requested_by_user_id: Option<UserPk>,

    /// The total number of resources (conditional based on the event kind).
    pub resource_count: Option<usize>,

    /// The ID of the component (conditional based on the event kind).
    pub component_id: Option<ComponentId>,
    /// The name of the component (conditional based on the event kind).
    pub component_name: Option<String>,
    /// The ID of the schema variant (conditional based on the event kind).
    pub schema_variant_id: Option<SchemaVariantId>,
    /// The ID of the schema (conditional based on the event kind).
    pub schema_id: Option<SchemaId>,
    /// The name of the schema (conditional based on the event kind).
    pub schema_name: Option<String>,
    /// The ID of the func run (conditional based on the event kind).
    pub func_run_id: Option<FuncRunId>,

    /// The kind of billing event.
    pub kind: BillingEventKind,
}

/// A wrapper around the billing events stream's NATS Jetstream context with helper methods for
/// interacting with the stream.
#[derive(Debug, Clone)]
pub struct BillingEventsWorkQueue {
    context: jetstream::Context,
}

impl BillingEventsWorkQueue {
    /// Create a new instance of billing events work queue and ensures the underlying stream is
    /// found or created.
    pub async fn get_or_create(context: jetstream::Context) -> BillingEventsResult<Self> {
        // Ensure the stream is created before we start publishing to it.
        let result = Self { context };
        result.stream().await?;
        Ok(result)
    }

    /// Returns a reference to the NATS Jetstream stream name.
    pub fn steam_name(&self) -> &str {
        STREAM_NAME
    }

    /// Publishes a workspace update.
    #[instrument(
        name = "billing_events_work_queue.publish_workspace_update",
        level = "info",
        skip_all,
        fields(
            si.workspace.id = workspace_id,
        )
    )]
    pub async fn publish_workspace_update(
        &self,
        workspace_id: &str,
        message: &impl Serialize,
    ) -> BillingEventsResult<()> {
        self.publish_message_inner(WORKSPACE_UPDATE_SUBJECT, workspace_id, message)
            .await
    }

    /// Returns the billing evenets stream.
    pub async fn stream(&self) -> BillingEventsResult<Stream> {
        let config = Config {
            name: self.prefixed_stream_name(STREAM_NAME),
            description: Some("Work queue of billing events".to_string()),
            subjects: vec![self.prefixed_subject(WORKSPACE_UPDATE_SUBJECT, ">")],
            retention: RetentionPolicy::WorkQueue,
            allow_direct: true,
            // Enable this to apply backpressure (stop allowing changes until we can process
            // billing events that have already happened).
            // discard_new_per_subject: true
            discard: DiscardPolicy::New,
            // This prevents deletion entirely, only allowing messages to disappear when ACKed
            // deny_delete: true,
            // deny_purge: true,
            ..Default::default()
        };
        Ok(self.context.get_or_create_stream(config).await?)
    }

    /// Provides the [`WORKSPACE_UPDATE_SUBJECT`] with an appropriate prefix and suffix.
    pub fn workspace_update_subject(&self, suffix: &str) -> String {
        self.prefixed_subject(WORKSPACE_UPDATE_SUBJECT, suffix)
    }

    async fn publish_message_inner(
        &self,
        subject: &str,
        parameters: &str,
        message: &impl Serialize,
    ) -> BillingEventsResult<()> {
        let subject = self.prefixed_subject(subject, parameters);
        let ack = self
            .context
            .publish_with_headers(
                subject,
                propagation::empty_injected_headers(),
                serde_json::to_vec(message)?.into(),
            )
            .await?;
        ack.await?;
        Ok(())
    }

    fn prefixed_stream_name(&self, stream_name: &str) -> String {
        match self.context.metadata().subject_prefix() {
            Some(prefix) => format!("{prefix}_{stream_name}"),
            None => stream_name.to_owned(),
        }
    }

    fn prefixed_subject(&self, subject: &str, suffix: &str) -> String {
        match self.context.metadata().subject_prefix() {
            Some(prefix) => format!("{prefix}.{subject}.{suffix}"),
            None => format!("{subject}.{suffix}"),
        }
    }
}
