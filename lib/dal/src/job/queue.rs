use super::producer::JobProducer;
use crate::job::definition::DependentValuesUpdate;
use crate::{AccessBuilder, AttributeValueId, ChangeSetId, Visibility};
use std::{collections::HashMap, collections::HashSet, collections::VecDeque, sync::Arc};
use tokio::sync::Mutex;

type DependentValuesUpdates =
    Arc<Mutex<HashMap<(ChangeSetId, AccessBuilder), HashSet<AttributeValueId>>>>;

#[derive(Debug, Clone, Default)]
pub struct JobQueue {
    queue: Arc<Mutex<VecDeque<Box<dyn JobProducer + Send + Sync>>>>,
    dependent_values_update_ids: DependentValuesUpdates,
}

impl JobQueue {
    pub fn new() -> Self {
        Self {
            queue: Default::default(),
            dependent_values_update_ids: Default::default(),
        }
    }

    pub async fn enqueue_dependent_values_update(
        &self,
        change_set_id: ChangeSetId,
        access_builder: AccessBuilder,
        ids: Vec<AttributeValueId>,
    ) {
        let mut lock = self.dependent_values_update_ids.lock().await;

        lock.entry((change_set_id, access_builder))
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
                .fetch_dependent_values_update()
                .await
                .map(|job| job as Box<dyn JobProducer + Send + Sync>),
        }
    }

    pub async fn fetch_dependent_values_update(&self) -> Option<Box<DependentValuesUpdate>> {
        let key = self
            .dependent_values_update_ids
            .lock()
            .await
            .keys()
            .next()
            .copied();
        if let Some((change_set_id, access_builder)) = key {
            let maybe_ids: Option<HashSet<AttributeValueId>> = self
                .dependent_values_update_ids
                .lock()
                .await
                .remove(&(change_set_id, access_builder));
            maybe_ids.map(|ids| {
                DependentValuesUpdate::new(
                    access_builder,
                    Visibility::new(change_set_id),
                    ids.into_iter().collect(),
                )
            })
        } else {
            None
        }
    }

    pub async fn is_empty(&self) -> bool {
        self.queue.lock().await.is_empty()
            && self.dependent_values_update_ids.lock().await.is_empty()
    }

    pub async fn size(&self) -> usize {
        self.queue.lock().await.len()
            + (!self.dependent_values_update_ids.lock().await.is_empty() as usize)
    }
}
