use std::sync::Arc;

use dal::{
    DedicatedExecutor,
    DedicatedExecutorError,
};
use miniz_oxide::deflate;
use nats_std::header;
use serde::Serialize;
use si_data_nats::{
    HeaderMap,
    NatsClient,
    Subject,
};
use si_frontend_mv_types::object::patch::{
    IndexUpdate,
    PatchBatch,
    StreamingPatch,
};
use si_id::WorkspacePk;
use telemetry::prelude::*;
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

    #[instrument(
        name = "edda_updates.publish_patch_batch",
        level = "debug",
        skip_all,
        fields()
    )]
    pub(crate) async fn publish_patch_batch(&self, patch_batch: PatchBatch) -> Result<()> {
        if self.streaming_patches {
            return Ok(());
        }

        let mut id_buf = WorkspacePk::array_to_str_buf();

        self.serialize_compress_publish(
            subject::update_for(
                self.subject_prefix.as_deref(),
                patch_batch.meta.workspace_id.array_to_str(&mut id_buf),
                patch_batch.kind(),
            ),
            patch_batch,
            true,
        )
        .await
    }

    #[instrument(
        name = "edda_updates.publish_streaming_patch",
        level = "debug",
        skip_all,
        fields()
    )]
    pub(crate) async fn publish_streaming_patch(
        &self,
        streaming_patch: StreamingPatch,
    ) -> Result<()> {
        if !self.streaming_patches {
            return Ok(());
        }

        let mut id_buf = WorkspacePk::array_to_str_buf();

        self.serialize_compress_publish(
            subject::update_for(
                self.subject_prefix.as_deref(),
                streaming_patch.workspace_id.array_to_str(&mut id_buf),
                streaming_patch.message_kind(),
            ),
            streaming_patch,
            true,
        )
        .await
    }

    #[instrument(
        name = "edda_updates.publish_index_update",
        level = "debug",
        skip_all,
        fields()
    )]
    pub(crate) async fn publish_index_update(&self, index_update: IndexUpdate) -> Result<()> {
        let mut id_buf = WorkspacePk::array_to_str_buf();

        self.serialize_compress_publish(
            subject::update_for(
                self.subject_prefix.as_deref(),
                index_update.meta.workspace_id.array_to_str(&mut id_buf),
                index_update.kind(),
            ),
            index_update,
            false,
        )
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

        if payload.len() > self.max_payload {
            let compressed = if should_compress { payload.len() } else { 0 };
            error!(
                bytes.size.uncompressed = serialized_len,
                bytes.size.compressed = compressed,
                bytes.size.payload = payload.len(),
                "message payload size {} exceeds NATS max_payload size {}; {}",
                payload.len(),
                self.max_payload,
                "message will fail to be publised on NATS",
            );
        }

        self.nats
            .publish_with_headers(subject, headers, payload.into())
            .await
            .map_err(Into::into)
    }
}

mod subject {
    use si_data_nats::Subject;

    const UPDATES_SUBJECT_PREFIX: &str = "edda.updates";

    #[inline]
    pub fn update_for(prefix: Option<&str>, workspace_id: &str, kind: &str) -> Subject {
        nats_std::subject::prefixed(
            prefix,
            format!("{UPDATES_SUBJECT_PREFIX}.{workspace_id}.{kind}"),
        )
    }
}
