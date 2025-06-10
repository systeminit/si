use std::{
    result,
    sync::Arc,
};

use dal::{
    DalContextBuilder,
    WorkspacePk,
    job::{
        consumer::{
            JobConsumer,
            JobConsumerError,
        },
        definition::{
            ActionJob,
            DependentValuesUpdate,
            ManagementFuncJob,
            compute_validation::ComputeValidation,
        },
    },
};
use naxum::{
    extract::State,
    response::{
        IntoResponse,
        Response,
    },
};
use pinga_core::api_types::{
    ApiWrapper,
    ContentInfo,
    job_execution_request::{
        JobArgsVCurrent,
        JobExecutionRequest,
    },
    job_execution_response::{
        JobExecutionResponse,
        JobExecutionResponseVCurrent,
        JobExecutionResultVCurrent,
    },
};
use si_data_nats::{
    HeaderMap,
    NatsClient,
    Subject,
};
use telemetry::prelude::*;
use telemetry_nats::propagation;
use telemetry_utils::metric;
use thiserror::Error;

use crate::{
    app_state::AppState,
    extract::{
        ApiTypesNegotiate,
        HeaderReply,
    },
    server::ServerMetadata,
};

#[remain::sorted]
#[derive(Debug, Error)]
pub enum HandlerError {
    #[error("job consumer error: {0}")]
    JobConsumer(#[from] JobConsumerError),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::Error),
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
    HeaderReply(maybe_reply): HeaderReply,
    ApiTypesNegotiate(request): ApiTypesNegotiate<JobExecutionRequest>,
) -> Result<()> {
    let AppState {
        metadata,
        concurrency_limit,
        nats,
        ctx_builder,
    } = state;

    let workspace_id = request.workspace_id;
    let change_set_id = request.change_set_id;

    let span = Span::current();
    span.record("si.workspace.id", workspace_id.to_string());
    span.record("si.change_set.id", change_set_id.to_string());

    execute_job(
        metadata,
        concurrency_limit,
        nats,
        ctx_builder,
        workspace_id,
        subject,
        maybe_reply,
        request,
    )
    .await;

    Ok(())
}

#[instrument(
    name = "execute_job", // will be `pinga jobs.:workspace_id.:change_set_id.$kind process`
    level = "info",
    skip_all,
    fields(
        // TODO: revive these fields as needed
        // concurrency.at_capacity = concurrency_limit == concurrency_count,
        // concurrency.count = concurrency_count,
        concurrency.limit = concurrency_limit,
        job.id = %request.id,
        job.instance = metadata.instance_id(),
        job.invoked_args = Empty,
        job.invoked_name = request.args.as_ref(),
        job.invoked_provider = metadata.job_invoked_provider(),
        job.trigger = "pubsub",
        messaging.destination = Empty,
        messaging.destination_kind = "topic",
        messaging.operation = "process",
        otel.kind = SpanKind::Consumer.as_str(),
        otel.name = Empty,
        otel.status_code = Empty,
        otel.status_message = Empty,
        si.change_set.id = %request.change_set_id,
        si.job.blocking = request.is_job_blocking,
        si.workspace.id = %request.workspace_id,
    )
)]
#[allow(clippy::too_many_arguments)]
async fn execute_job(
    metadata: Arc<ServerMetadata>,
    concurrency_limit: usize,
    nats: NatsClient,
    ctx_builder: DalContextBuilder,
    workspace_id: WorkspacePk,
    subject: Subject,
    maybe_reply: Option<Subject>,
    request: JobExecutionRequest,
) {
    let span = current_span_for_instrument_at!("info");
    let id = request.id;
    let job_kind: &'static str = (&request.args).into();

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

    span.record("messaging.destination", subject.as_str());
    span.record("otel.name", otel_name.as_str());
    span.record("si.workspace.id", workspace_id.to_string());

    metric!(counter.pinga_jobs_in_progress = 1, label = job_kind);

    let execution_result = match try_execute_job(ctx_builder, request.clone()).await {
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
            Err(span.record_err(err))
        }
    };

    // If a reply was requested, send it
    if let Some(reply) = maybe_reply {
        let response = JobExecutionResponse::new_current(JobExecutionResponseVCurrent {
            id: request.id,
            workspace_id: request.workspace_id,
            change_set_id: request.change_set_id,
            result: match execution_result {
                Ok(_) => JobExecutionResultVCurrent::Ok,
                Err(err) => JobExecutionResultVCurrent::Err {
                    message: err.to_string(),
                },
            },
        });

        let info = ContentInfo::from(&response);

        let mut headers = HeaderMap::new();
        propagation::inject_headers(&mut headers);
        info.inject_into_headers(&mut headers);

        let payload = match response.to_vec() {
            Ok(p) => p,
            Err(err) => {
                error!(si.error.message = ?err, "failed to serialize response body");
                return;
            }
        };

        if let Err(err) = nats
            .publish_with_headers(reply, headers, payload.into())
            .await
        {
            error!(
                si.error.message = ?err,
                "unable to publish response of blocking job completion",
            );
        };
    }

    metric!(counter.pinga_jobs_in_progress = -1, label = job_kind);
}

async fn try_execute_job(
    mut ctx_builder: DalContextBuilder,
    request: JobExecutionRequest,
) -> Result<()> {
    if request.is_job_blocking {
        ctx_builder.set_blocking();
    }

    let job = match &request.args {
        JobArgsVCurrent::Action { action_id } => {
            ActionJob::new(request.workspace_id, request.change_set_id, *action_id)
                as Box<dyn JobConsumer + Send + Sync>
        }
        JobArgsVCurrent::DependentValuesUpdate => {
            DependentValuesUpdate::new(request.workspace_id, request.change_set_id)
                as Box<dyn JobConsumer + Send + Sync>
        }
        JobArgsVCurrent::Validation {
            attribute_value_ids,
        } => ComputeValidation::new(
            request.workspace_id,
            request.change_set_id,
            attribute_value_ids.clone(),
        ) as Box<dyn JobConsumer + Send + Sync>,
        JobArgsVCurrent::ManagementFunc {
            component_id,
            prototype_id,
            view_id,
            request_ulid,
        } => ManagementFuncJob::new(
            request.workspace_id,
            request.change_set_id,
            *prototype_id,
            *component_id,
            *view_id,
            *request_ulid,
        ) as Box<dyn JobConsumer + Send + Sync>,
    };

    info!("Processing job");

    job.run_job(ctx_builder).await?;

    info!("Finished processing job");

    Ok(())
}
