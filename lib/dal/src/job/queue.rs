use std::sync::Arc;

use ringmap::{
    RingMap,
    RingSet,
};
use si_id::{
    ActionId,
    AttributeValueId,
    ChangeSetId,
    WorkspacePk,
};
use tokio::sync::Mutex;

use super::definition::ActionJob;
use crate::job::{
    consumer::DalJob,
    definition::{
        DependentValuesUpdate,
        compute_validation::ComputeValidation,
    },
};

type ActionChangeSets = Arc<Mutex<RingSet<(WorkspacePk, ChangeSetId, ActionId)>>>;
type DependentValuesUpdateChangeSets = Arc<Mutex<RingSet<(WorkspacePk, ChangeSetId)>>>;
type ValidationChangeSets = Arc<Mutex<RingMap<(WorkspacePk, ChangeSetId), Vec<AttributeValueId>>>>;

#[derive(Debug, Clone, Default)]
pub struct JobQueue {
    action_change_sets: ActionChangeSets,
    dependent_value_update_change_sets: DependentValuesUpdateChangeSets,
    validation_change_sets: ValidationChangeSets,
}

impl JobQueue {
    pub async fn enqueue_action_job(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        action_id: ActionId,
    ) {
        self.action_change_sets
            .lock()
            .await
            .insert((workspace_id, change_set_id, action_id));
    }

    pub async fn enqueue_dependent_values_update_job(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
    ) {
        self.dependent_value_update_change_sets
            .lock()
            .await
            .insert((workspace_id, change_set_id));
    }

    pub async fn enqueue_validation_job(
        &self,
        workspace_id: WorkspacePk,
        change_set_id: ChangeSetId,
        attribute_value_id: AttributeValueId,
    ) {
        self.validation_change_sets
            .lock()
            .await
            .entry((workspace_id, change_set_id))
            .or_default()
            .push(attribute_value_id);
    }

    /// Pop jobs off queue in a prioritized, FIFO manner.
    pub async fn pop_job(&self) -> Option<Box<dyn DalJob>> {
        if let Some((workspace_id, change_set_id)) = self
            .dependent_value_update_change_sets
            .lock()
            .await
            .pop_front()
        {
            Some(DependentValuesUpdate::new(workspace_id, change_set_id))
        } else if let Some(((workspace_id, change_set_id), attribute_value_ids)) =
            self.validation_change_sets.lock().await.pop_front()
        {
            Some(ComputeValidation::new(
                workspace_id,
                change_set_id,
                attribute_value_ids,
            ))
        } else if let Some((workspace_id, change_set_id, action_id)) =
            self.action_change_sets.lock().await.pop_front()
        {
            Some(ActionJob::new(workspace_id, change_set_id, action_id))
        } else {
            None
        }
    }

    /// Grab the dependent value update set for a change set and remove it from
    /// the queue (for sending via a rebase request)
    pub async fn clear_dependent_values_jobs(&self) -> bool {
        let mut set = self.dependent_value_update_change_sets.lock().await;
        let was_populated = !set.is_empty();
        set.clear();

        was_populated
    }

    pub async fn size(&self) -> usize {
        self.action_change_sets.lock().await.len()
            + self.dependent_value_update_change_sets.lock().await.len()
            + self.validation_change_sets.lock().await.len()
    }
}
