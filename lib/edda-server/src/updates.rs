use std::sync::Arc;

use dal::{
    DedicatedExecutor,
    DedicatedExecutorError,
};
use edda_core::nats;
use miniz_oxide::deflate;
use nats_std::header;
use serde::Serialize;
use si_data_nats::{
    HeaderMap,
    NatsClient,
    Subject,
};
use si_frontend_mv_types::object::patch::{
    ChangesetIndexUpdate,
    ChangesetPatchBatch,
    DeploymentIndexUpdate,
    DeploymentPatchBatch,
    StreamingPatch,
};
use si_id::WorkspacePk;
use telemetry::{
    OpenTelemetrySpanExt,
    prelude::*,
};
use telemetry_nats::propagation;
use telemetry_utils::monotonic;
use thiserror::Error;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum EddaUpdatesError {
    #[error("compute executor error: {0}")]
    ComputeExecutor(#[from] DedicatedExecutorError),
    #[error("Nats error: {0}")]
    Nats(#[from] si_data_nats::Error),
    #[error("error serializing object: {0}")]
    Serialize(#[source] serde_json::Error),
}

type Result<T> = std::result::Result<T, EddaUpdatesError>;

type Error = EddaUpdatesError;

#[derive(Clone, Debug)]
pub(crate) struct EddaUpdates {
    nats: NatsClient,
    compute_executor: DedicatedExecutor,
    subject_prefix: Option<Arc<str>>,
    max_payload: usize,
    streaming_patches: bool,
}

impl EddaUpdates {
    pub(crate) fn new(
        nats: NatsClient,
        compute_executor: DedicatedExecutor,
        streaming_patches: bool,
    ) -> Self {
        let subject_prefix = nats
            .metadata()
            .subject_prefix()
            .map(|p| p.to_string().into());
        let max_payload = nats.server_info().max_payload;
        Self {
            nats,
            compute_executor,
            subject_prefix,
            max_payload,
            streaming_patches,
        }
    }

    pub(crate) async fn publish_change_set_patch_batch(
        &self,
        patch_batch: ChangesetPatchBatch,
    ) -> Result<()> {
        // Only publish a change set patch batch if we are not streaming patches.
        if self.streaming_patches {
            Ok(())
        } else {
            self.publish_change_set_patch_batch_inner(patch_batch).await
        }
    }

    #[instrument(
        name = "edda_updates.publish_change_set_patch_batch",
        level = "info",
        skip_all
    )]
    async fn publish_change_set_patch_batch_inner(
        &self,
        patch_batch: ChangesetPatchBatch,
    ) -> Result<()> {
        let mut id_buf = WorkspacePk::array_to_str_buf();

        let subject = nats::subject::workspace_update_for(
            self.subject_prefix.as_deref(),
            patch_batch.meta.workspace_id.array_to_str(&mut id_buf),
            patch_batch.kind(),
        );

        monotonic!(edda_updates_publish = 1, kind = "change_set_patch_batch");
        let span = current_span_for_instrument_at!("info");
        self.serialize_compress_publish(subject, patch_batch, true, &span)
            .await
    }

    #[instrument(
        name = "edda_updates.publish_deployment_patch_batch",
        level = "info",
        skip_all
    )]
    pub(crate) async fn publish_deployment_patch_batch(
        &self,
        patch_batch: DeploymentPatchBatch,
    ) -> Result<()> {
        let subject = nats::subject::deployment_update_for(
            self.subject_prefix.as_deref(),
            patch_batch.kind(),
        );

        monotonic!(edda_updates_publish = 1, kind = "deployment_patch_batch");
        let span = current_span_for_instrument_at!("info");
        self.serialize_compress_publish(subject, patch_batch, true, &span)
            .await
    }

    pub(crate) async fn publish_streaming_patch(
        &self,
        streaming_patch: StreamingPatch,
    ) -> Result<()> {
        // Only publish streaming patch if we are streaming patches.
        if self.streaming_patches {
            self.publish_streaming_patch_inner(streaming_patch).await
        } else {
            Ok(())
        }
    }

    #[instrument(
        name = "edda_updates.publish_streaming_patch",
        level = "info",
        skip_all
    )]
    async fn publish_streaming_patch_inner(&self, streaming_patch: StreamingPatch) -> Result<()> {
        let mut id_buf = WorkspacePk::array_to_str_buf();

        monotonic!(
            edda_updates_publish = 1,
            kind = "change_set_streaming_patch"
        );
        let span = current_span_for_instrument_at!("info");
        self.serialize_compress_publish(
            nats::subject::workspace_update_for(
                self.subject_prefix.as_deref(),
                streaming_patch.workspace_id.array_to_str(&mut id_buf),
                streaming_patch.message_kind(),
            ),
            streaming_patch,
            true,
            &span,
        )
        .await
    }

    #[instrument(
        name = "edda_updates.publish_change_set_index_update",
        level = "info",
        skip_all
    )]
    pub(crate) async fn publish_change_set_index_update(
        &self,
        index_update: ChangesetIndexUpdate,
    ) -> Result<()> {
        let mut id_buf = WorkspacePk::array_to_str_buf();

        let subject = nats::subject::workspace_update_for(
            self.subject_prefix.as_deref(),
            index_update.meta.workspace_id.array_to_str(&mut id_buf),
            index_update.kind(),
        );

        monotonic!(edda_updates_publish = 1, kind = "change_set_index_update");
        let span = current_span_for_instrument_at!("info");
        self.serialize_compress_publish(subject, index_update, false, &span)
            .await
    }

    #[instrument(
        name = "edda_updates.publish_deployment_index_update",
        level = "info",
        skip_all
    )]
    pub(crate) async fn publish_deployment_index_update(
        &self,
        index_update: DeploymentIndexUpdate,
    ) -> Result<()> {
        let subject = nats::subject::deployment_update_for(
            self.subject_prefix.as_deref(),
            index_update.kind(),
        );

        monotonic!(edda_updates_publish = 1, kind = "deployment_index_update");
        let span = current_span_for_instrument_at!("info");
        self.serialize_compress_publish(subject, index_update, false, &span)
            .await
    }

    #[instrument(
        name = "edda_updates.serialize_compress_publish",
        level = "debug",
        skip_all,
        fields(
            bytes.size.compressed = Empty,
            bytes.size.uncompressed = Empty,
        ),
    )]
    async fn serialize_compress_publish<S>(
        &self,
        subject: Subject,
        object: S,
        should_compress: bool,
        parent_span: &Span,
    ) -> Result<()>
    where
        S: Serialize + Sync + Send + 'static,
    {
        let span = current_span_for_instrument_at!("debug");

        let (serialized_len, payload) = if should_compress {
            self.compute_executor
                .spawn(async move {
                    let serialized = serde_json::to_vec(&object).map_err(Error::Serialize)?;
                    Ok::<_, EddaUpdatesError>((
                        serialized.len(),
                        deflate::compress_to_vec(&serialized, 6),
                    ))
                })
                .await??
        } else {
            let serialized = serde_json::to_vec(&object).map_err(Error::Serialize)?;
            (serialized.len(), serialized)
        };

        span.record("bytes.size.uncompressed", serialized_len);
        if should_compress {
            span.record("bytes.size.compressed", payload.len());
        }

        let mut headers = HeaderMap::new();
        header::insert_content_type(&mut headers, header::value::ContentType::JSON);
        if should_compress {
            header::insert_content_encoding(&mut headers, header::value::ContentEncoding::DEFLATE);
        }

        // Inject with a given parent span so that we can see patches before they are sent to
        // clients processing MV patches.
        propagation::inject_opentelemetry_context(&parent_span.context(), &mut headers);

        if payload.len() > self.max_payload {
            let compressed = if should_compress { payload.len() } else { 0 };
            error!(
                bytes.size.uncompressed = serialized_len,
                bytes.size.compressed = compressed,
                bytes.size.payload = payload.len(),
                "message payload size {} exceeds NATS max_payload size {}; {}",
                payload.len(),
                self.max_payload,
                "message will fail to be published on NATS",
            );
        }

        self.nats
            .publish_with_headers(subject, headers, payload.into())
            .await
            .map_err(Into::into)
    }
}
