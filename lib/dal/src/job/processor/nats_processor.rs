use async_trait::async_trait;
use pinga_client::PingaClient;
use pinga_core::api_types::{
    job_execution_request::JobArgsVCurrent,
    job_execution_response::JobExecutionResultVCurrent,
};
use si_data_nats::NatsClient;
use telemetry::prelude::*;
use tokio::task::JoinSet;

use crate::job::{
    consumer::DalJob,
    processor::{
        JobQueueProcessor,
        JobQueueProcessorError,
        JobQueueProcessorResult,
    },
    producer::{
        BlockingJobError,
        BlockingJobResult,
    },
    queue::JobQueue,
};

#[derive(Clone, Debug)]
pub struct NatsProcessor {
    pinga: PingaClient,
}

impl NatsProcessor {
    pub async fn new(client: NatsClient) -> JobQueueProcessorResult<Self> {
        let pinga = PingaClient::new(client)
            .await
            .map_err(|err| JobQueueProcessorError::Transport(Box::new(err)))?;

        Ok(Self { pinga })
    }

    #[instrument(
        name = "nats_processor.push_all_jobs",
        level = "debug",
        skip_all,
        fields()
    )]
    async fn push_all_jobs(&self, queue: JobQueue) -> JobQueueProcessorResult<()> {
        while let Some(job) = queue.pop_job().await {
            match job.args() {
                JobArgsVCurrent::Action { action_id } => {
                    self.pinga
                        .dispatch_action_job(
                            job.workspace_id(),
                            job.change_set_id(),
                            action_id,
                            false,
                        )
                        .await?;
                }
                JobArgsVCurrent::DependentValuesUpdate => {
                    self.pinga
                        .dispatch_dependent_values_update_job(
                            job.workspace_id(),
                            job.change_set_id(),
                            false,
                        )
                        .await?;
                }
                JobArgsVCurrent::Validation {
                    attribute_value_ids,
                } => {
                    self.pinga
                        .dispatch_validation_job(
                            job.workspace_id(),
                            job.change_set_id(),
                            attribute_value_ids,
                            false,
                        )
                        .await?;
                }
            }
        }

        Ok(())
    }
}

#[async_trait]
impl JobQueueProcessor for NatsProcessor {
    async fn block_on_job(&self, job: Box<dyn DalJob>) -> BlockingJobResult {
        let (_request_id, response_fut) = match job.args() {
            JobArgsVCurrent::Action { action_id } => {
                self.pinga
                    .await_action_job(job.workspace_id(), job.change_set_id(), action_id, false)
                    .await?
            }
            JobArgsVCurrent::DependentValuesUpdate => {
                self.pinga
                    .await_dependent_values_update_job(
                        job.workspace_id(),
                        job.change_set_id(),
                        false,
                    )
                    .await?
            }
            JobArgsVCurrent::Validation {
                attribute_value_ids,
            } => {
                self.pinga
                    .await_validation_job(
                        job.workspace_id(),
                        job.change_set_id(),
                        attribute_value_ids,
                        false,
                    )
                    .await?
            }
        };

        // TODO(fnichol): hrm, no timeout, so we wait forever? That's probably not expected?
        let job_response = response_fut.await?;

        // TODO(fnichol): I don't think we want to return a `Result::Err` if the job ran to
        // completion but encountered an error. However, currently a nontrivial amount of code may
        // rely on this function signature return, so this preserves prior behavior--for now if the
        // job ran to completion but encountered an error. However, currently a nontrivial amount
        // of code may rely on this function signature return, so this preserves prior
        // behavior--for now
        match &job_response.result {
            JobExecutionResultVCurrent::Ok => Ok(()),
            JobExecutionResultVCurrent::Err { message } => {
                Err(BlockingJobError::JobExecution(message.clone()))
            }
        }
    }

    async fn block_on_jobs(&self, jobs: Vec<Box<dyn DalJob>>) -> BlockingJobResult {
        let span = Span::current();

        let mut dispatched_jobs = JoinSet::new();

        // Fan out, dispatching all queued jobs to pinga over nats.
        for job in jobs {
            let job_processor = self.clone();
            let parent_span = span.clone();

            dispatched_jobs.spawn(async move {
                job_processor
                    .block_on_job(job)
                    .instrument(info_span!(parent: parent_span, "job_processor.block_on_job"))
                    .await
            });
        }

        let mut job_errors = Vec::new();
        // Wait for all queued jobs to finish (regardless of success), before exiting.
        loop {
            match dispatched_jobs.join_next().await {
                // All jobs done.
                None => break,
                Some(Ok(Ok(_))) => { /* Nothing to do. Job succeeded. */ }
                Some(Ok(Err(job_error))) => {
                    job_errors.push(job_error);
                }
                Some(Err(join_err)) => {
                    job_errors.push(BlockingJobError::JobExecution(join_err.to_string()));
                }
            }
        }

        if !job_errors.is_empty() {
            Err(BlockingJobError::JobExecution(
                job_errors
                    .iter()
                    .map(|e| e.to_string())
                    .collect::<Vec<String>>()
                    .join("\n"),
            ))
        } else {
            Ok(())
        }
    }

    #[instrument(
        name = "nats_processor.process_queue",
        level = "info",
        skip_all,
        fields(
            queue.size = Empty,
        )
    )]
    async fn process_queue(&self, queue: JobQueue) -> JobQueueProcessorResult<()> {
        let span = current_span_for_instrument_at!("info");

        span.record("queue.size", queue.size().await);

        self.push_all_jobs(queue).await?;

        Ok(())
    }

    #[instrument(
        name = "nats_processor.blocking_process_queue",
        level = "info",
        skip_all,
        fields(
            queue.size = Empty,
        )
    )]
    async fn blocking_process_queue(&self, queue: JobQueue) -> JobQueueProcessorResult<()> {
        let span = current_span_for_instrument_at!("info");

        span.record("queue.size", queue.size().await);

        let mut jobs = Vec::with_capacity(queue.size().await);
        while let Some(element) = queue.pop_job().await {
            jobs.push(element);
        }
        self.block_on_jobs(jobs)
            .instrument(info_span!("nats_processor.block_on_jobs"))
            .await?;

        Ok(())
    }
}
