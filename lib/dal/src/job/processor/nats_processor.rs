use async_trait::async_trait;
use futures::StreamExt;
use si_data_nats::{NatsClient, Subject};
use telemetry::prelude::*;
use telemetry_nats::propagation;
use tokio::task::JoinSet;

use crate::job::{
    consumer::JobInfo,
    producer::{BlockingJobError, BlockingJobResult, JobProducer, JobProducerError},
    queue::JobQueue,
};

use super::{JobQueueProcessor, JobQueueProcessorError, JobQueueProcessorResult};

const NATS_JOB_QUEUE: &str = "pinga-jobs";

#[derive(Clone, Debug)]
pub struct NatsProcessor {
    client: NatsClient,
    pinga_subject: Subject,
}

impl NatsProcessor {
    pub fn new(client: NatsClient) -> Self {
        let pinga_subject = if let Some(prefix) = client.metadata().subject_prefix() {
            format!("{prefix}.{NATS_JOB_QUEUE}").into()
        } else {
            NATS_JOB_QUEUE.into()
        };

        Self {
            client,
            pinga_subject,
        }
    }

    #[instrument(
        name = "nats_processor.push_all_jobs",
        level = "debug",
        skip_all,
        fields()
    )]
    async fn push_all_jobs(&self, queue: JobQueue) -> JobQueueProcessorResult<()> {
        let headers = propagation::empty_injected_headers();

        while let Some(element) = queue.fetch_job().await {
            let job_info = JobInfo::new(element)?;

            if let Err(err) = self
                .client
                .publish_with_headers(
                    self.pinga_subject.clone(),
                    headers.clone(),
                    serde_json::to_vec(&job_info)?.into(),
                )
                .await
            {
                error!("Nats job push failed, some jobs will be dropped");
                return Err(JobQueueProcessorError::Transport(Box::new(err)));
            }
        }
        Ok(())
    }
}

#[async_trait]
impl JobQueueProcessor for NatsProcessor {
    async fn block_on_job(&self, job: Box<dyn JobProducer + Send + Sync>) -> BlockingJobResult {
        let mut job_info = JobInfo::new_blocking(job)
            .map_err(|e: JobProducerError| BlockingJobError::JobProducer(e.to_string()))?;

        job_info.blocking = true;

        let job_reply_inbox = Subject::from(self.client.new_inbox());
        let mut reply_subscriber = self
            .client
            .subscribe(job_reply_inbox.clone())
            .await
            .map_err(|e| BlockingJobError::Nats(e.to_string()))?;
        self.client
            .publish_with_reply_and_headers(
                self.pinga_subject.clone(),
                job_reply_inbox,
                propagation::empty_injected_headers(),
                serde_json::to_vec(&job_info)
                    .map_err(|e| BlockingJobError::Serde(e.to_string()))?
                    .into(),
            )
            .await
            .map_err(|e| BlockingJobError::Nats(e.to_string()))?;

        match reply_subscriber.next().await {
            Some(message) => serde_json::from_slice::<BlockingJobResult>(message.payload())
                .map_err(|e| BlockingJobError::Serde(e.to_string()))?,
            None => Err(BlockingJobError::Nats(
                "Subscriber or connection no longer valid".to_string(),
            )),
        }
    }

    async fn block_on_jobs(
        &self,
        jobs: Vec<Box<dyn JobProducer + Send + Sync>>,
    ) -> BlockingJobResult {
        let span = Span::current();

        let mut dispatched_jobs = JoinSet::new();

        // Fan out, dispatching all queued jobs to pinga over nats.
        for job in jobs {
            let job_processor = Self::new(self.client.clone());
            let parent_span = span.clone();

            dispatched_jobs.spawn(async move {
                job_processor
                    .block_on_job(job)
                    .instrument(info_span!(parent: parent_span, "job_processor.block_on_job"))
                    .await
            });
        }

        let mut results = Vec::new();
        // Wait for all queued jobs to finish (regardless of success), before exiting.
        loop {
            match dispatched_jobs.join_next().await {
                // All jobs done.
                None => break,
                Some(Ok(Ok(_))) => { /* Nothing to do. Job succeeded. */ }
                Some(Ok(Err(job_error))) => {
                    results.push(job_error);
                }
                Some(Err(join_err)) => {
                    results.push(BlockingJobError::JobExecution(join_err.to_string()));
                }
            }
        }

        info!("processed_queue");

        if !results.is_empty() {
            Err(BlockingJobError::JobExecution(
                results
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
        let span = Span::current();
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
        let span = Span::current();
        span.record("queue.size", queue.size().await);

        self.block_on_jobs(queue.drain().await)
            .instrument(info_span!("nats_processor.block_on_jobs"))
            .await?;

        Ok(())
    }
}
