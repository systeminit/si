use std::{
    result,
    str::Utf8Error,
    sync::Arc,
};

use dal::{
    DalContextBuilder,
    WorkspacePk,
    job::{
        consumer::{
            JobConsumer,
            JobConsumerError,
            JobInfo,
        },
        definition::{
            ActionJob,
            DependentValuesUpdate,
            compute_validation::ComputeValidation,
        },
        producer::BlockingJobError,
    },
};
use nats_std::headers;
use naxum::{
    Json,
    extract::{
        State,
        message_parts::Headers,
    },
    response::{
        IntoResponse,
        Response,
    },
};
use si_data_nats::Subject;
use telemetry::prelude::*;
use telemetry_nats::propagation;
use telemetry_utils::metric;
use thiserror::Error;

use crate::{
    app_state::AppState,
    server::ServerMetadata,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("job consumer error: {0}")]
    JobConsumer(#[from] JobConsumerError),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
    #[error("unknown job kind: {0}")]
    UnknownJobKind(String),
    #[error("utf8 error when creating subject")]
    Utf8(#[source] Utf8Error),
}

type Result<T> = result::Result<T, HandlerError>;

impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        error!(si.error.message = ?self, "failed to process message");
        Response::default_internal_server_error()
    }
}

pub async fn process_request(
    State(state): State<AppState>,
    subject: Subject,
    Headers(maybe_headers): Headers,
    Json(job_info): Json<JobInfo>,
) -> Result<()> {
    let workspace_id = job_info.access_builder.tenancy().workspace_pk()?;
    let change_set_id = job_info.visibility.change_set_id;

    let span = Span::current();
    span.record("si.workspace.id", workspace_id.to_string());
    span.record("si.change_set.id", change_set_id.to_string());

    let reply_subject = match maybe_headers
        .and_then(|headers| headers.get(headers::REPLY_INBOX).map(|v| v.to_string()))
    {
        Some(header_value) => Some(Subject::from_utf8(header_value).map_err(HandlerError::Utf8)?),
        None => None,
    };

    execute_job(
        state.metadata,
        state.concurrency_limit,
        state.ctx_builder,
        workspace_id,
        subject,
        reply_subject,
        job_info,
    )
    .await;
    Ok(())
}

#[instrument(
    name = "execute_job",
    level = "info",
    skip_all,
    fields(
        // TODO: revive these fields as needed
        // concurrency.at_capacity = concurrency_limit == concurrency_count,
        // concurrency.count = concurrency_count,
        concurrency.limit = concurrency_limit,
        job.id = job_info.id,
        job.instance = metadata.instance_id(),
        job.invoked_args = Empty,
        job.invoked_name = job_info.kind,
        job.invoked_provider = metadata.job_invoked_provider(),
        job.trigger = "pubsub",
        messaging.destination = Empty,
        messaging.destination_kind = "topic",
        messaging.operation = "process",
        otel.kind = SpanKind::Consumer.as_str(),
        otel.name = Empty,
        otel.status_code = Empty,
        otel.status_message = Empty,
        si.change_set.id = %job_info.visibility.change_set_id,
        si.job.blocking = job_info.blocking,
        si.workspace.id = Empty,
    )
)]
async fn execute_job(
    metadata: Arc<ServerMetadata>,
    concurrency_limit: usize,
    ctx_builder: DalContextBuilder,
    workspace_id: WorkspacePk,
    subject: Subject,
    maybe_reply_subject: Option<Subject>,
    job_info: JobInfo,
) {
    let span = current_span_for_instrument_at!("info");
    let id = job_info.id.clone();

    let arg_str = serde_json::to_string(&job_info.arg)
        .unwrap_or_else(|_| "arg failed to serialize".to_string());
    let otel_name = {
        let mut parts = subject.as_str().split('.');
        match (
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
            parts.next(),
        ) {
            (Some(p1), Some(p2), Some(_workspace_id), Some(_change_set_id), Some(kind)) => {
                format!("{p1}.{p2}.:workspace_id.:change_set_id.{kind} process")
            }
            _ => format!("{} process", subject.as_str()),
        }
    };

    span.record("job.invoked_arg", arg_str);
    span.record("messaging.destination", subject.as_str());
    span.record("otel.name", otel_name.as_str());
    span.record("si.workspace.id", workspace_id.to_string());
    let job_kind = job_info.kind.clone();
    metric!(counter.pinga_job_in_progress = 1, label = job_kind);
    let reply_message = match execute_job_inner(ctx_builder.clone(), job_info).await {
        Ok(_) => {
            span.record_ok();
            Ok(())
        }
        Err(err) => {
            error!(
                si.error.message = ?err,
                job.invocation_id = %id,
                job.instance = metadata.instance_id(),
                "job execution failed"
            );
            let new_err = Err(BlockingJobError::JobExecution(err.to_string()));
            span.record_err(err);

            new_err
        }
    };

    // If a reply subject is set then the caller has requested we publish a reply
    if let Some(reply_subject) = maybe_reply_subject {
        if let Ok(message) = serde_json::to_vec(&reply_message) {
            if let Err(err) = ctx_builder
                .nats_conn()
                .publish_with_headers(
                    reply_subject,
                    propagation::empty_injected_headers(),
                    message.into(),
                )
                .await
            {
                error!(error = ?err, "Unable to notify spawning job of blocking job completion");
            };
        }
    }
    metric!(counter.pinga_job_in_progress = -1, label = job_kind);
}

async fn execute_job_inner(mut ctx_builder: DalContextBuilder, job_info: JobInfo) -> Result<()> {
    if job_info.blocking {
        ctx_builder.set_blocking();
    }

    let job = match job_info.kind.as_str() {
        stringify!(DependentValuesUpdate) => {
            Box::new(DependentValuesUpdate::try_from(job_info.clone())?)
                as Box<dyn JobConsumer + Send + Sync>
        }
        stringify!(ActionJob) => {
            Box::new(ActionJob::try_from(job_info.clone())?) as Box<dyn JobConsumer + Send + Sync>
        }
        stringify!(ComputeValidation) => Box::new(ComputeValidation::try_from(job_info.clone())?)
            as Box<dyn JobConsumer + Send + Sync>,
        kind => return Err(HandlerError::UnknownJobKind(kind.to_owned())),
    };

    info!("Processing job");

    job.run_job(ctx_builder.clone()).await?;

    info!("Finished processing job");

    Ok(())
}
