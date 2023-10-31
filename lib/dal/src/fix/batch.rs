//! This module contains [`FixBatch`], which groups [`Fixs`](crate::Fix)
//! and indicates whether or not all "fixes" in the group have completed executing.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use telemetry::prelude::*;

use crate::{
    fix::{FixCompletionStatus, FixError, FixResult},
    impl_standard_model, pk, standard_model, standard_model_accessor, standard_model_has_many,
    DalContext, Fix, StandardModel, Tenancy, Timestamp, Visibility, WsEvent, WsEventResult,
    WsPayload,
};

pk!(FixBatchPk);
pk!(FixBatchId);

/// A batch of [`Fixs`](crate::Fix). Every [`Fix`](crate::Fix)
/// must belong at one and only one [`batch`](Self).
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct FixBatch {
    pk: FixBatchPk,
    id: FixBatchId,
    #[serde(flatten)]
    tenancy: Tenancy,
    #[serde(flatten)]
    timestamp: Timestamp,
    #[serde(flatten)]
    visibility: Visibility,

    // TODO(nick): automate with the logged in user.
    author: String,

    // This is a comma separated list of people involved in the ChangeSet
    actors: Option<String>,

    // TODO(nick): convert to Option<DateTime<Utc>> once standard model accessor can accommodate both
    // Option<T<U>> and can handle "timestamp with time zone <--> DateTime<Utc>".
    /// Indicates when the [`FixBatch`] started execution when populated.
    started_at: Option<String>,
    // TODO(nick): convert to Option<DateTime<Utc>> once standard model accessor can accommodate both
    // Option<T<U>> and can handle "timestamp with time zone <--> DateTime<Utc>".
    /// Indicates when the [`FixBatch`] finished execution when populated.
    finished_at: Option<String>,
    /// Indicates the state of the [`FixBatch`] when finished.
    completion_status: Option<FixCompletionStatus>,
}

impl_standard_model! {
    model: FixBatch,
    pk: FixBatchPk,
    id: FixBatchId,
    table_name: "fix_batches",
    history_event_label_base: "fix_batch",
    history_event_message_name: "FixBatch"
}

impl FixBatch {
    #[instrument(skip_all)]
    pub async fn new(ctx: &DalContext, author: impl AsRef<str>, actors: &str) -> FixResult<Self> {
        let author = author.as_ref();
        let row = ctx
            .txns()
            .await?
            .pg()
            .query_one(
                "SELECT object FROM fix_batch_create_v1($1, $2, $3, $4)",
                &[ctx.tenancy(), ctx.visibility(), &author, &actors],
            )
            .await?;
        let object = standard_model::finish_create_from_row(ctx, row).await?;
        Ok(object)
    }

    standard_model_accessor!(started_at, Option<String>, FixResult);
    standard_model_accessor!(finished_at, Option<String>, FixResult);
    standard_model_accessor!(
        completion_status,
        Option<Enum(FixCompletionStatus)>,
        FixResult
    );

    // TODO(nick): store the order (and what's sequential, conditional, parallel, etc.) someday.
    standard_model_has_many!(
        lookup_fn: fixes,
        table: "fix_belongs_to_fix_batch",
        model_table: "fixes",
        returns: Fix,
        result: FixResult,
    );

    /// A safe wrapper around setting the finished and completion status columns.
    pub async fn stamp_finished(&mut self, ctx: &DalContext) -> FixResult<FixCompletionStatus> {
        if self.started_at.is_some() {
            self.set_finished_at(ctx, Some(Utc::now().to_rfc3339()))
                .await?;

            // TODO(nick): getting what the batch completion status should be can be a query.
            let mut batch_completion_status = FixCompletionStatus::Success;
            for fix in self.fixes(ctx).await? {
                match fix
                    .completion_status()
                    .ok_or(FixError::EmptyCompletionStatus)?
                {
                    FixCompletionStatus::Success => {}
                    FixCompletionStatus::Failure => {
                        // If we see failures, we should still continue to see if there's an error.
                        batch_completion_status = FixCompletionStatus::Failure
                    }
                    FixCompletionStatus::Error | FixCompletionStatus::Unstarted => {
                        // Only break on an error since errors take precedence over failures.
                        batch_completion_status = FixCompletionStatus::Error;
                        break;
                    }
                }
            }

            self.set_completion_status(ctx, Some(batch_completion_status))
                .await?;
            Ok(batch_completion_status)
        } else {
            Err(FixError::NotYetStarted)
        }
    }

    /// A safe wrapper around setting the started column.
    pub async fn stamp_started(&mut self, ctx: &DalContext) -> FixResult<()> {
        if self.started_at.is_some() {
            Err(FixError::AlreadyStarted)
        } else if self.finished_at.is_some() {
            Err(FixError::AlreadyFinished)
        } else if self.fixes(ctx).await?.is_empty() {
            Err(FixError::NoFixesInBatch(self.id))
        } else {
            self.set_started_at(ctx, Some(Utc::now().to_rfc3339()))
                .await?;
            Ok(())
        }
    }

    pub fn author(&self) -> String {
        self.author.clone()
    }

    pub fn actors(&self) -> Option<String> {
        self.actors.clone()
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FixBatchReturn {
    id: FixBatchId,
    status: FixCompletionStatus,
}

impl WsEvent {
    pub async fn fix_batch_return(
        ctx: &DalContext,
        id: FixBatchId,
        status: FixCompletionStatus,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::FixBatchReturn(FixBatchReturn { id, status }),
        )
        .await
    }
}
