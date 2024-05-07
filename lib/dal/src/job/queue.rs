use std::{collections::HashMap, collections::HashSet, collections::VecDeque, sync::Arc};
use tokio::sync::Mutex;
use ulid::Ulid;

use super::producer::JobProducer;
use crate::job::definition::compute_validation::ComputeValidation;
use crate::job::definition::{AttributeValueBasedJobIdentifier, DependentValuesUpdate};
use crate::{AccessBuilder, ChangeSetId, Visibility};

type AttributeValueBasedJobs = Arc<
    Mutex<
        HashMap<
            AttributeValueBasedJobIdentifier,
            HashMap<(ChangeSetId, AccessBuilder), HashSet<Ulid>>,
        >,
    >,
>;

#[derive(Debug, Clone, Default)]
pub struct JobQueue {
    queue: Arc<Mutex<VecDeque<Box<dyn JobProducer + Send + Sync>>>>,
    attribute_value_based_jobs: AttributeValueBasedJobs,
}

impl JobQueue {
    pub fn new() -> Self {
        Self {
            queue: Default::default(),
            attribute_value_based_jobs: Default::default(),
        }
    }

    pub async fn enqueue_attribute_value_job(
        &self,
        change_set_id: ChangeSetId,
        access_builder: AccessBuilder,
        job_kind: AttributeValueBasedJobIdentifier,
        ids: Vec<impl Into<Ulid>>,
    ) {
        let mut lock = self.attribute_value_based_jobs.lock().await;
        let ids: Vec<Ulid> = ids.into_iter().map(|id| id.into()).collect();
        lock.entry(job_kind)
            .or_default()
            .entry((change_set_id, access_builder))
            .or_default()
            .extend(ids);
    }

    pub async fn enqueue_job(&self, job: Box<dyn JobProducer + Send + Sync>) {
        let mut lock = self.queue.lock().await;

        lock.push_back(job);
    }

    pub async fn fetch_job(&self) -> Option<Box<dyn JobProducer + Send + Sync>> {
        match self.queue.lock().await.pop_front() {
            Some(job) => Some(job),
            None => self
                .fetch_attribute_value_based_job()
                .await
                .map(|job| job as Box<dyn JobProducer + Send + Sync>),
        }
    }

    pub async fn fetch_attribute_value_based_job(
        &self,
    ) -> Option<Box<dyn JobProducer + Send + Sync>> {
        for job_kind in AttributeValueBasedJobIdentifier::in_priority_order() {
            let mut jobs_for_kind = self.attribute_value_based_jobs.lock().await;

            let Some(jobs_for_changeset) = jobs_for_kind.get_mut(&job_kind) else {
                continue;
            };

            let key = jobs_for_changeset.keys().next().copied();

            let Some((change_set_id, access_builder)) = key else {
                continue;
            };

            let Some(ids) = jobs_for_changeset.remove(&(change_set_id, access_builder)) else {
                continue;
            };

            // The logic of mapping AttributeValueBasedJobIdentifier into a job box would be better
            // in the job definitions module, but this works.
            return Some(match job_kind {
                AttributeValueBasedJobIdentifier::DependentValuesUpdate => {
                    DependentValuesUpdate::new(
                        access_builder,
                        Visibility::new(change_set_id),
                        ids.into_iter().collect(),
                    )
                }
                AttributeValueBasedJobIdentifier::ComputeValidation => ComputeValidation::new(
                    access_builder,
                    Visibility::new(change_set_id),
                    // NOTE(nick): despite dependent values update accepting ids for any node, we
                    // know that validations must take in attribute values ids. This conversion is
                    // safe given that the enqueue methods require an explicit list of attribute
                    // value ids.
                    ids.iter().map(|id| (*id).into()).collect(),
                ),
            });
        }

        None
    }

    pub async fn is_empty(&self) -> bool {
        self.queue.lock().await.is_empty()
            && self.attribute_value_based_jobs.lock().await.is_empty()
    }

    pub async fn size(&self) -> usize {
        self.queue.lock().await.len()
            + (!self.attribute_value_based_jobs.lock().await.is_empty() as usize)
    }
}
