//! This module contains [`ActionBatch`], which groups [`ActionRunners`](crate::ActionRunner)
//! and indicates whether or not all "actions" in the group have completed executing.

use chrono::{DateTime, Utc};
use content_store::{ContentHash, Store, StoreError};
use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use std::collections::HashMap;
use telemetry::prelude::*;
use thiserror::Error;

use crate::change_set_pointer::ChangeSetPointerError;
use crate::workspace_snapshot::content_address::{ContentAddress, ContentAddressDiscriminants};
use crate::workspace_snapshot::edge_weight::{
    EdgeWeight, EdgeWeightError, EdgeWeightKind, EdgeWeightKindDiscriminants,
};
use crate::workspace_snapshot::node_weight::category_node_weight::CategoryNodeKind;
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    func::binding_return_value::FuncBindingReturnValueError,
    layer_db_types::{ActionBatchContent, ActionBatchContentV1},
    pk, ActionCompletionStatus, ActionPrototypeError, ActionRunner, ActionRunnerError,
    ComponentError, DalContext, FuncError, HistoryEventError, SchemaError, Timestamp,
    TransactionsError, WsEvent, WsEventError, WsEventResult, WsPayload,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ActionBatchError {
    #[error(transparent)]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error(transparent)]
    ActionRunner(#[from] ActionRunnerError),
    #[error("cannot stamp batch as started since it already finished")]
    AlreadyFinished,
    #[error("cannot stamp batch as started since it already started")]
    AlreadyStarted,
    #[error(transparent)]
    ChangeSetPointer(#[from] ChangeSetPointerError),
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error("edge weight error: {0}")]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("completion status is empty")]
    EmptyCompletionStatus,
    #[error(transparent)]
    Func(#[from] FuncError),
    #[error(transparent)]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error(transparent)]
    HistoryEvent(#[from] HistoryEventError),
    #[error("no action runners in batch: action batch is empty")]
    NoActionRunnersInBatch(ActionBatchId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("cannot stamp batch as finished since it has not yet been started")]
    NotYetStarted,
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    Schema(#[from] SchemaError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Store(#[from] StoreError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnapshot(#[from] WorkspaceSnapshotError),
    #[error(transparent)]
    WsEvent(#[from] WsEventError),
}

pub type ActionBatchResult<T, E = ActionBatchError> = std::result::Result<T, E>;

/// A batch of [`ActionRunners`](crate::ActionRunner). Every [`ActionRunner`](crate::ActionRunner)
/// must belong at one and only one [`batch`](Self).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ActionBatch {
    pub id: ActionBatchId,
    pub timestamp: Timestamp,

    // TODO(nick): automate with the logged in user.
    pub author: String,

    // This is a comma separated list of people involved in the ChangeSet
    pub actors: String,

    /// Indicates when the [`ActionBatch`] started execution when populated.
    pub started_at: Option<DateTime<Utc>>,
    /// Indicates when the [`ActionBatch`] finished execution when populated.
    pub finished_at: Option<DateTime<Utc>>,
    /// Indicates the state of the [`ActionBatch`] when finished.
    pub completion_status: Option<ActionCompletionStatus>,
}

impl From<ActionBatch> for ActionBatchContentV1 {
    fn from(batch: ActionBatch) -> Self {
        Self {
            author: batch.author,
            actors: batch.actors,
            started_at: batch.started_at,
            finished_at: batch.finished_at,
            completion_status: batch.completion_status,
            timestamp: batch.timestamp,
        }
    }
}

impl ActionBatch {
    pub fn assemble(id: ActionBatchId, content: ActionBatchContentV1) -> Self {
        Self {
            id,
            author: content.author,
            actors: content.actors,
            started_at: content.started_at,
            finished_at: content.finished_at,
            completion_status: content.completion_status,
            timestamp: content.timestamp,
        }
    }

    pub async fn new(
        ctx: &DalContext,
        author: impl AsRef<str>,
        actors: &str,
    ) -> ActionBatchResult<Self> {
        let timestamp = Timestamp::now();

        let content = ActionBatchContentV1 {
            author: author.as_ref().to_owned(),
            actors: actors.to_owned(),
            started_at: None,
            finished_at: None,
            completion_status: None,
            timestamp,
        };

        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&ActionBatchContent::V1(content.clone()))?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::ActionBatch(hash))?;

        let workspace_snapshot = ctx.workspace_snapshot()?;

        workspace_snapshot.add_node(node_weight.to_owned()).await?;

        // Root --> ActionBatch Category --> Component (this)
        let category_id = workspace_snapshot
            .get_category_node(None, CategoryNodeKind::ActionBatch)
            .await?;
        workspace_snapshot
            .add_edge(
                category_id,
                EdgeWeight::new(change_set, EdgeWeightKind::Use)?,
                id,
            )
            .await?;

        Ok(Self::assemble(id.into(), content))
    }

    pub async fn list(ctx: &DalContext) -> ActionBatchResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let mut action_batchs = vec![];
        let action_batch_category_node_id = workspace_snapshot
            .get_category_node(None, CategoryNodeKind::ActionBatch)
            .await?;

        let action_batch_node_indices = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(
                action_batch_category_node_id,
                EdgeWeightKindDiscriminants::Use,
            )
            .await?;

        let mut node_weights = vec![];
        let mut hashes = vec![];
        for index in action_batch_node_indices {
            let node_weight = workspace_snapshot
                .get_node_weight(index)
                .await?
                .get_content_node_weight_of_kind(ContentAddressDiscriminants::ActionBatch)?;
            hashes.push(node_weight.content_hash());
            node_weights.push(node_weight);
        }

        let contents: HashMap<ContentHash, ActionBatchContent> = ctx
            .content_store()
            .lock()
            .await
            .get_bulk(hashes.as_slice())
            .await?;

        for node_weight in node_weights {
            match contents.get(&node_weight.content_hash()) {
                Some(content) => {
                    // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
                    let ActionBatchContent::V1(inner) = content;

                    action_batchs.push(Self::assemble(node_weight.id().into(), inner.to_owned()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }

        Ok(action_batchs)
    }

    pub async fn runners(&self, ctx: &DalContext) -> ActionBatchResult<Vec<ActionRunner>> {
        Ok(ActionRunner::for_batch(ctx, self.id).await?)
    }

    pub async fn set_completion_status(
        &mut self,
        ctx: &DalContext,
        status: Option<ActionCompletionStatus>,
    ) -> ActionBatchResult<()> {
        self.completion_status = status;
        let content = ActionBatchContentV1::from(self.clone());

        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&ActionBatchContent::V1(content.clone()))?;

        ctx.workspace_snapshot()?
            .update_content(ctx.change_set_pointer()?, self.id.into(), hash)
            .await?;
        Ok(())
    }

    pub async fn set_started_at(&mut self, ctx: &DalContext) -> ActionBatchResult<()> {
        self.started_at = Some(Utc::now());
        let content = ActionBatchContentV1::from(self.clone());

        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&ActionBatchContent::V1(content.clone()))?;

        ctx.workspace_snapshot()?
            .update_content(ctx.change_set_pointer()?, self.id.into(), hash)
            .await?;
        Ok(())
    }

    pub async fn set_finished_at(&mut self, ctx: &DalContext) -> ActionBatchResult<()> {
        self.finished_at = Some(Utc::now());
        let content = ActionBatchContentV1::from(self.clone());

        let hash = ctx
            .content_store()
            .lock()
            .await
            .add(&ActionBatchContent::V1(content.clone()))?;

        ctx.workspace_snapshot()?
            .update_content(ctx.change_set_pointer()?, self.id.into(), hash)
            .await?;
        Ok(())
    }

    /// A safe wrapper around setting the finished and completion status columns.
    pub async fn stamp_finished(
        &mut self,
        ctx: &DalContext,
    ) -> ActionBatchResult<ActionCompletionStatus> {
        if self.started_at.is_some() {
            self.set_finished_at(ctx).await?;

            // TODO(nick): getting what the batch completion status should be can be a query.
            let mut batch_completion_status = ActionCompletionStatus::Success;
            for runner in self.runners(ctx).await? {
                match runner
                    .completion_status
                    .ok_or(ActionBatchError::EmptyCompletionStatus)?
                {
                    ActionCompletionStatus::Success => {}
                    ActionCompletionStatus::Failure => {
                        // If we see failures, we should still continue to see if there's an error.
                        batch_completion_status = ActionCompletionStatus::Failure
                    }
                    ActionCompletionStatus::Error | ActionCompletionStatus::Unstarted => {
                        // Only break on an error since errors take precedence over failures.
                        batch_completion_status = ActionCompletionStatus::Error;
                        break;
                    }
                }
            }

            self.set_completion_status(ctx, Some(batch_completion_status))
                .await?;
            Ok(batch_completion_status)
        } else {
            Err(ActionBatchError::NotYetStarted)
        }
    }

    /// A safe wrapper around setting the started column.
    pub async fn stamp_started(&mut self, ctx: &DalContext) -> ActionBatchResult<()> {
        if self.started_at.is_some() {
            Err(ActionBatchError::AlreadyStarted)
        } else if self.finished_at.is_some() {
            Err(ActionBatchError::AlreadyFinished)
        } else if self.runners(ctx).await?.is_empty() {
            Err(ActionBatchError::NoActionRunnersInBatch(self.id))
        } else {
            self.set_started_at(ctx).await?;
            Ok(())
        }
    }

    pub fn author(&self) -> String {
        self.author.clone()
    }

    pub fn actors(&self) -> String {
        self.actors.clone()
    }
}

pk!(ActionBatchId);

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionBatchReturn {
    id: ActionBatchId,
    status: ActionCompletionStatus,
}

impl WsEvent {
    pub async fn action_batch_return(
        ctx: &DalContext,
        id: ActionBatchId,
        status: ActionCompletionStatus,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ActionBatchReturn(ActionBatchReturn { id, status }),
        )
        .await
    }
}
