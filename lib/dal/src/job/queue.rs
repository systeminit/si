use std::{
    collections::{hash_map::Entry, HashMap, HashSet, VecDeque},
    sync::Arc,
};
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
            HashMap<ChangeSetId, (HashSet<Ulid>, AccessBuilder)>,
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
            .entry(change_set_id)
            .and_modify(|entry| entry.0.extend(ids.clone()))
            .or_insert((HashSet::from_iter(ids.clone().into_iter()), access_builder));
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

    /// Grab the dependent value update set for a change set and remove it from
    /// the queue (for sending via a rebase request)
    pub async fn take_dependent_values_for_change_set(
        &self,
        change_set_id: ChangeSetId,
    ) -> Option<Vec<Ulid>> {
        match self
            .attribute_value_based_jobs
            .lock()
            .await
            .entry(AttributeValueBasedJobIdentifier::DependentValuesUpdate)
        {
            Entry::Vacant(_) => None,
            Entry::Occupied(mut entry) => entry
                .get_mut()
                .remove(&change_set_id)
                .map(|(values, _)| values.into_iter().collect()),
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

            let Some(change_set_id) = jobs_for_changeset.keys().next().copied() else {
                continue;
            };

            let Some((ids, access_builder)) = jobs_for_changeset.remove(&change_set_id) else {
                continue;
            };

            // The logic of mapping AttributeValueBasedJobIdentifier into a job box would be better
            // in the job definitions module, but this works.
            return Some(match job_kind {
                AttributeValueBasedJobIdentifier::DependentValuesUpdate => {
                    DependentValuesUpdate::new(access_builder, Visibility::new(change_set_id))
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
