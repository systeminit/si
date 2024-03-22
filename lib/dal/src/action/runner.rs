//! This module contains the concept of "actions".

use chrono::{DateTime, Utc};
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};
use si_data_pg::PgError;
use si_events::ContentHash;
use si_layer_cache::LayerDbError;
use std::collections::HashMap;
use std::sync::Arc;
use strum::{AsRefStr, Display, EnumIter, EnumString};
use telemetry::prelude::*;
use thiserror::Error;

use crate::action::batch::ActionBatchId;
use crate::change_set_pointer::ChangeSetPointerError;
use crate::func::binding_return_value::FuncBindingReturnValueError;
use crate::workspace_snapshot::content_address::ContentAddress;
use crate::workspace_snapshot::edge_weight::EdgeWeightKindDiscriminants;
use crate::workspace_snapshot::edge_weight::{EdgeWeight, EdgeWeightError, EdgeWeightKind};
use crate::workspace_snapshot::node_weight::{NodeWeight, NodeWeightError};
use crate::workspace_snapshot::WorkspaceSnapshotError;
use crate::{
    component::resource::ResourceView,
    func::backend::js_action::ActionRunResult,
    layer_db_types::{ActionRunnerContent, ActionRunnerContentV1},
    pk, ActionId, ActionKind, ActionPrototype, ActionPrototypeError, ActionPrototypeId, Component,
    ComponentError, ComponentId, DalContext, Func, FuncError, HistoryEventError, SchemaError,
    SchemaVariantError, Timestamp, TransactionsError, WsEvent, WsEventError, WsEventResult,
    WsPayload,
};
use veritech_client::ResourceStatus;

/// The completion status of a [`ActionRunner`]
#[remain::sorted]
#[derive(
    Deserialize,
    Serialize,
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    ToSql,
    FromSql,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum ActionCompletionStatus {
    Error,
    Failure,
    Success,
    Unstarted,
}

#[remain::sorted]
#[derive(Error, Debug)]
pub enum ActionRunnerError {
    #[error(transparent)]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("cannot stamp action runner as started since it already finished")]
    AlreadyFinished,
    #[error("cannot stamp action runner as started since it already started")]
    AlreadyStarted,
    #[error("cannot set action runner for {0}: batch ({1}) already finished")]
    BatchAlreadyFinished(ActionRunnerId, ActionBatchId),
    #[error("cannot set action runner for {0}: batch ({1}) already started")]
    BatchAlreadyStarted(ActionRunnerId, ActionBatchId),
    #[error(transparent)]
    ChangeSetPointer(#[from] ChangeSetPointerError),
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error(transparent)]
    EdgeWeight(#[from] EdgeWeightError),
    #[error("completion status is empty")]
    EmptyCompletionStatus,
    #[error(transparent)]
    Func(#[from] FuncError),
    #[error(transparent)]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error(transparent)]
    HistoryEvent(#[from] HistoryEventError),
    #[error("action run status cannot be converted to action completion status")]
    IncompatibleActionRunStatus,
    #[error("layer db error: {0}")]
    LayerDb(#[from] LayerDbError),
    #[error("missing action runner: {0}")]
    MissingActionRunner(ActionRunnerId),
    #[error("missing finished timestamp for action runner: {0}")]
    MissingFinishedTimestampForActionRunner(ActionRunnerId),
    #[error("missing started timestamp for action runner: {0}")]
    MissingStartedTimestampForActionRunner(ActionRunnerId),
    #[error("node weight error: {0}")]
    NodeWeight(#[from] NodeWeightError),
    #[error("not found: {0}")]
    NotFound(ActionRunnerId),
    #[error("not found for action: {0}")]
    NotFoundForAction(ActionId),
    #[error("cannot stamp action runner as finished since it has not yet been started")]
    NotYetStarted,
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error(transparent)]
    Schema(#[from] SchemaError),
    #[error(transparent)]
    SchemaVariant(#[from] SchemaVariantError),
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
    #[error("could not acquire lock: {0}")]
    TryLock(#[from] tokio::sync::TryLockError),
    #[error(transparent)]
    WorkspaceSnaphot(#[from] WorkspaceSnapshotError),
    #[error(transparent)]
    WsEvent(#[from] WsEventError),
}

pub type ActionRunnerResult<T> = Result<T, ActionRunnerError>;

pk!(ActionRunnerId);

/// A record of a "action runner" after it has been executed.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct ActionRunner {
    pub id: ActionRunnerId,
    pub timestamp: Timestamp,

    pub component_id: ComponentId,
    pub component_name: String,
    pub schema_name: String,
    pub func_name: String,
    pub action_prototype_id: ActionPrototypeId,
    pub action_kind: ActionKind,
    pub resource: Option<ActionRunResult>,

    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub completion_status: Option<ActionCompletionStatus>,
    pub completion_message: Option<String>,
}

impl From<ActionRunner> for ActionRunnerContentV1 {
    fn from(content: ActionRunner) -> Self {
        Self {
            component_id: content.component_id,
            component_name: content.component_name,
            schema_name: content.schema_name,
            func_name: content.func_name,
            action_prototype_id: content.action_prototype_id,
            action_kind: content.action_kind,
            resource: content
                .resource
                .map(|r| serde_json::to_string(&r))
                .transpose()
                .expect("unable to serialize resource"),
            started_at: content.started_at,
            finished_at: content.finished_at,
            completion_status: content.completion_status,
            completion_message: content.completion_message,
            timestamp: content.timestamp,
        }
    }
}

impl ActionRunner {
    pub fn assemble(id: ActionRunnerId, content: ActionRunnerContentV1) -> Self {
        Self {
            id,
            component_id: content.component_id,
            component_name: content.component_name,
            schema_name: content.schema_name,
            func_name: content.func_name,
            action_prototype_id: content.action_prototype_id,
            action_kind: content.action_kind,
            resource: content
                .resource
                .map(|r| serde_json::from_str(&r))
                .transpose()
                .expect("unable to deserialize resource"),
            started_at: content.started_at,
            finished_at: content.finished_at,
            completion_status: content.completion_status,
            completion_message: content.completion_message,
            timestamp: content.timestamp,
        }
    }

    /// Create [`Self`] and ensure it belongs to a [`ActionBatch`](crate::ActionBatch)
    /// since every [`action`](Self) must belong to a [`batch`](crate::ActionBatch).
    pub async fn new(
        ctx: &DalContext,
        action_batch_id: ActionBatchId,
        component_id: ComponentId,
        component_name: String,
        action_prototype_id: ActionPrototypeId,
    ) -> ActionRunnerResult<Self> {
        let timestamp = Timestamp::now();

        let component = Component::get_by_id(ctx, component_id).await?;
        let prototype = ActionPrototype::get_by_id(ctx, action_prototype_id).await?;
        let func = Func::get_by_id(ctx, prototype.func_id(ctx).await?).await?;
        let func_name = func
            .display_name
            .clone()
            .unwrap_or_else(|| func.name.clone());
        let schema_name = component.schema(ctx).await?.name().to_owned();

        let content = ActionRunnerContentV1 {
            component_id,
            component_name,
            schema_name,
            func_name,
            action_kind: prototype.kind,
            action_prototype_id,
            started_at: None,
            resource: None,
            finished_at: None,
            completion_status: None,
            completion_message: None,
            timestamp,
        };

        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(ActionRunnerContent::V1(content.clone()).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        let change_set = ctx.change_set_pointer()?;
        let id = change_set.generate_ulid()?;
        let node_weight =
            NodeWeight::new_content(change_set, id, ContentAddress::ActionRunner(hash))?;

        let workspace_snapshot = ctx.workspace_snapshot()?;

        workspace_snapshot.add_node(node_weight.to_owned()).await?;
        workspace_snapshot
            .add_edge(
                action_batch_id,
                EdgeWeight::new(change_set, EdgeWeightKind::new_use())?,
                id,
            )
            .await?;

        Ok(ActionRunner::assemble(id.into(), content))
    }

    pub async fn get_by_id(ctx: &DalContext, id: ActionRunnerId) -> ActionRunnerResult<Self> {
        let workspace_snapshot = ctx.workspace_snapshot()?;
        let node_index = workspace_snapshot.get_node_index_by_id(id).await?;
        let node_weight = workspace_snapshot.get_node_weight(node_index).await?;
        let hash = node_weight.content_hash();

        let content: ActionRunnerContent = ctx
            .layer_db()
            .cas()
            .try_read_as(&hash)
            .await?
            .ok_or_else(|| WorkspaceSnapshotError::MissingContentFromStore(id.into()))?;

        // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
        let ActionRunnerContent::V1(inner) = content;

        Ok(Self::assemble(id, inner))
    }

    /// Executes the [`action runner`](Self). Returns true if some resource got updated, false if not
    pub async fn run(&mut self, ctx: &DalContext) -> ActionRunnerResult<Option<ActionRunResult>> {
        // Stamp started and run the workflow.
        self.stamp_started(ctx).await?;

        let action_prototype = ActionPrototype::get_by_id(ctx, self.action_prototype_id).await?;

        Ok(match action_prototype.run(ctx, self.component_id).await {
            Ok(Some(run_result)) => {
                let completion_status = match run_result.status {
                    Some(ResourceStatus::Ok) | Some(ResourceStatus::Warning) => {
                        ActionCompletionStatus::Success
                    }
                    Some(ResourceStatus::Error) => ActionCompletionStatus::Failure,
                    None => ActionCompletionStatus::Unstarted,
                };

                self.stamp_finished(
                    ctx,
                    completion_status,
                    run_result.message.clone(),
                    Some(run_result.clone()),
                )
                .await?;

                Some(run_result)
            }
            Ok(None) => {
                error!("ActionRunner did not return a value!");
                self.stamp_finished(
                    ctx,
                    ActionCompletionStatus::Error,
                    Some("ActionRunner did not return a value".into()),
                    None,
                )
                .await?;

                None
            }
            Err(e) => {
                error!("Unable to run action: {e}");
                self.stamp_finished(
                    ctx,
                    ActionCompletionStatus::Error,
                    Some(format!("{e:?}")),
                    None,
                )
                .await?;

                None
            }
        })
    }

    /// A safe wrapper around setting completion-related columns.
    pub async fn stamp_finished(
        &mut self,
        ctx: &DalContext,
        completion_status: ActionCompletionStatus,
        completion_message: Option<String>,
        resource: Option<ActionRunResult>,
    ) -> ActionRunnerResult<()> {
        if self.started_at.is_some() {
            self.set_finished_at(ctx).await?;
            self.set_completion_status(ctx, Some(completion_status))
                .await?;
            if completion_message.is_some() {
                self.set_completion_message(ctx, completion_message).await?;
            }
            self.set_resource(ctx, resource).await?;

            Ok(())
        } else {
            Err(ActionRunnerError::NotYetStarted)
        }
    }

    async fn update_content(&self, ctx: &DalContext) -> ActionRunnerResult<()> {
        let content = ActionRunnerContentV1::from(self.clone());

        let (hash, _) = ctx
            .layer_db()
            .cas()
            .write(
                Arc::new(ActionRunnerContent::V1(content).into()),
                None,
                ctx.events_tenancy(),
                ctx.events_actor(),
            )
            .await?;

        ctx.workspace_snapshot()?
            .update_content(ctx.change_set_pointer()?, self.id.into(), hash)
            .await?;

        Ok(())
    }

    pub async fn set_resource(
        &mut self,
        ctx: &DalContext,
        resource: Option<ActionRunResult>,
    ) -> ActionRunnerResult<()> {
        self.resource = resource;
        self.update_content(ctx).await
    }

    pub async fn set_completion_message(
        &mut self,
        ctx: &DalContext,
        message: Option<String>,
    ) -> ActionRunnerResult<()> {
        self.completion_message = message;
        self.update_content(ctx).await
    }

    pub async fn set_completion_status(
        &mut self,
        ctx: &DalContext,
        status: Option<ActionCompletionStatus>,
    ) -> ActionRunnerResult<()> {
        self.completion_status = status;
        self.update_content(ctx).await
    }

    pub async fn set_started_at(&mut self, ctx: &DalContext) -> ActionRunnerResult<()> {
        self.started_at = Some(Utc::now());
        self.update_content(ctx).await
    }

    pub async fn set_finished_at(&mut self, ctx: &DalContext) -> ActionRunnerResult<()> {
        self.finished_at = Some(Utc::now());
        self.update_content(ctx).await
    }

    /// A safe wrapper around setting the started column.
    pub async fn stamp_started(&mut self, ctx: &DalContext) -> ActionRunnerResult<()> {
        if self.started_at.is_some() {
            Err(ActionRunnerError::AlreadyStarted)
        } else if self.finished_at.is_some() {
            Err(ActionRunnerError::AlreadyFinished)
        } else {
            self.set_started_at(ctx).await?;
            Ok(())
        }
    }

    /// Generates a [`ActionHistoryView`] based on [`self`](ActionRunner).
    pub async fn history_view(&self) -> ActionRunnerResult<ActionHistoryView> {
        Ok(ActionHistoryView {
            id: self.id,
            status: self
                .completion_status
                .unwrap_or(ActionCompletionStatus::Unstarted),
            action_kind: self.action_kind,
            display_name: self.func_name.clone(),
            schema_name: self.schema_name.clone(),
            component_name: self.component_name.clone(),
            component_id: self.component_id,
            resource: if let Some(resource) = self.resource.clone() {
                Some(ResourceView {
                    status: resource.status,
                    message: resource.message,
                    data: resource
                        .payload
                        .as_deref()
                        .map(serde_json::from_str)
                        .transpose()?,
                    logs: resource.logs,
                    last_synced: resource.last_synced,
                })
            } else {
                None
            },
            started_at: self.started_at.as_ref().map(|s| s.to_string()),
            finished_at: self.finished_at.as_ref().map(|s| s.to_string()),
        })
    }

    pub async fn for_batch(
        ctx: &DalContext,
        batch_id: ActionBatchId,
    ) -> ActionRunnerResult<Vec<Self>> {
        let workspace_snapshot = ctx.workspace_snapshot()?;

        let nodes = workspace_snapshot
            .outgoing_targets_for_edge_weight_kind(batch_id, EdgeWeightKindDiscriminants::Use)
            .await?;
        let mut node_weights = Vec::with_capacity(nodes.len());
        let mut content_hashes = Vec::with_capacity(nodes.len());
        for node in nodes {
            let weight = workspace_snapshot.get_node_weight(node).await?;
            content_hashes.push(weight.content_hash());
            node_weights.push(weight);
        }

        let content_map: HashMap<ContentHash, ActionRunnerContent> = ctx
            .layer_db()
            .cas()
            .try_read_many_as(&content_hashes)
            .await?;

        let mut actions = Vec::with_capacity(node_weights.len());
        for node_weight in node_weights {
            match content_map.get(&node_weight.content_hash()) {
                Some(content) => {
                    // NOTE(nick,jacob,zack): if we had a v2, then there would be migration logic here.
                    let ActionRunnerContent::V1(inner) = content;

                    actions.push(Self::assemble(node_weight.id().into(), inner.clone()));
                }
                None => Err(WorkspaceSnapshotError::MissingContentFromStore(
                    node_weight.id(),
                ))?,
            }
        }
        Ok(actions)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionHistoryView {
    id: ActionRunnerId,
    status: ActionCompletionStatus,
    action_kind: ActionKind,
    display_name: String,
    schema_name: String,
    component_name: String,
    component_id: ComponentId,
    started_at: Option<String>,
    finished_at: Option<String>,
    resource: Option<ResourceView>,
}

impl ActionHistoryView {
    pub fn status(&self) -> ActionCompletionStatus {
        self.status
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionRunnerReturn {
    id: ActionRunnerId,
    batch_id: ActionBatchId,
    action: ActionKind,
    status: ActionCompletionStatus,
    output: Vec<String>,
}

impl WsEvent {
    pub async fn action_return(
        ctx: &DalContext,
        id: ActionRunnerId,
        batch_id: ActionBatchId,
        action: ActionKind,
        status: ActionCompletionStatus,
        output: Vec<String>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ActionRunnerReturn(ActionRunnerReturn {
                id,
                batch_id,
                action,
                status,
                output,
            }),
        )
        .await
    }
}
