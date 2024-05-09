use std::num::ParseIntError;

use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use thiserror::Error;
use ulid::Ulid;

use crate::change_set::event::{ChangeSetActorPayload, ChangeSetMergeVotePayload};
use crate::component::{
    ComponentCreatedPayload, ComponentDeletedPayload, ComponentSetPositionPayload,
    ComponentUpdatedPayload, ConnectionCreatedPayload, ConnectionDeletedPayload,
};
use crate::func::FuncWsEventPayload;
use crate::pkg::{
    ImportWorkspaceVotePayload, WorkspaceActorPayload, WorkspaceImportApprovalActorPayload,
};
use crate::qualification::QualificationCheckPayload;
use crate::schema::variant::{
    SchemaVariantClonedPayload, SchemaVariantCreatedPayload, SchemaVariantSavedPayload,
    SchemaVariantUpdatedPayload,
};
use crate::status::StatusUpdate;
use crate::user::OnlinePayload;
use crate::{
    action::ActionReturn,
    deprecated_action::prototype::ResourceRefreshedPayload,
    deprecated_action::{
        batch::DeprecatedActionBatchReturn, runner::ActionRunnerReturn, ActionId, ActionView,
        DeprecatedActionError,
    },
    func::binding::LogLinePayload,
    pkg::ModuleImportedPayload,
    user::CursorPayload,
    ChangeSetId, DalContext, DeprecatedActionPrototypeError, FuncError, PropId, StandardModelError,
    TransactionsError, WorkspacePk,
};
use crate::{SecretCreatedPayload, SecretUpdatedPayload};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WsEventError {
    #[error("deprecated action error: {0}")]
    DeprecatedAction(#[from] Box<DeprecatedActionError>),
    #[error("deprecated action error: {0}")]
    DeprecatedActionPrototype(#[from] Box<DeprecatedActionPrototypeError>),
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("no user in context")]
    NoUserInContext,
    #[error("no workspace in tenancy")]
    NoWorkspaceInTenancy,
    #[error("number parse string error: {0}")]
    ParseIntError(#[from] ParseIntError),
    #[error(transparent)]
    Pg(#[from] PgError),
    #[error("error serializing/deserializing json: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error(transparent)]
    Transactions(#[from] TransactionsError),
}

pub type WsEventResult<T> = Result<T, WsEventError>;

#[remain::sorted]
#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
#[serde(tag = "kind", content = "data")]
#[allow(clippy::large_enum_variant)]
pub enum WsPayload {
    ActionReturn(ActionReturn),
    AsyncError(ErrorPayload),
    AsyncFinish(FinishPayload),
    ChangeSetAbandoned(ChangeSetActorPayload),
    ChangeSetAbandonVote(ChangeSetMergeVotePayload),
    ChangeSetApplied(ChangeSetActorPayload),
    ChangeSetBeginAbandonProcess(ChangeSetActorPayload),
    ChangeSetBeginApprovalProcess(ChangeSetActorPayload),
    ChangeSetCancelAbandonProcess(ChangeSetActorPayload),
    ChangeSetCancelApprovalProcess(ChangeSetActorPayload),
    ChangeSetCanceled(ChangeSetId),
    ChangeSetCreated(ChangeSetId),
    ChangeSetMergeVote(ChangeSetMergeVotePayload),
    ChangeSetWritten(ChangeSetId),
    CheckedQualifications(QualificationCheckPayload),
    ComponentCreated(ComponentCreatedPayload),
    ComponentDeleted(ComponentDeletedPayload),
    ComponentUpdated(ComponentUpdatedPayload),
    ConnectionCreated(ConnectionCreatedPayload),
    ConnectionDeleted(ConnectionDeletedPayload),
    Cursor(CursorPayload),
    DeprecatedActionAdded(ActionView),
    DeprecatedActionBatchReturn(DeprecatedActionBatchReturn),
    DeprecatedActionRemoved(ActionId),
    DeprecatedActionRunnerReturn(ActionRunnerReturn),
    FuncDeleted(FuncWsEventPayload),
    FuncSaved(FuncWsEventPayload),
    ImportWorkspaceVote(ImportWorkspaceVotePayload),
    LogLine(LogLinePayload),
    ModuleImported(ModuleImportedPayload),
    Online(OnlinePayload),
    ResourceRefreshed(ResourceRefreshedPayload),
    // SchemaCreated(SchemaPk),
    SchemaVariantCloned(SchemaVariantClonedPayload),
    SchemaVariantCreated(SchemaVariantCreatedPayload),
    SchemaVariantSaved(SchemaVariantSavedPayload),
    SchemaVariantUpdateFinished(SchemaVariantUpdatedPayload),
    SecretCreated(SecretCreatedPayload),
    SecretUpdated(SecretUpdatedPayload),
    SetComponentPosition(ComponentSetPositionPayload),
    StatusUpdate(StatusUpdate),
    // WorkspaceExported(WorkspaceExportPayload),
    WorkspaceImportBeginApprovalProcess(WorkspaceImportApprovalActorPayload),
    WorkspaceImportCancelApprovalProcess(WorkspaceActorPayload),
    // WorkspaceImported(WorkspaceImportPayload),
}

#[remain::sorted]
#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq, Copy, Hash)]
#[serde(rename_all = "camelCase", tag = "kind", content = "id")]
pub enum StatusValueKind {
    Attribute(PropId),
    CodeGen,
    Internal,
    Qualification,
}

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct WsEvent {
    version: i64,
    workspace_pk: WorkspacePk,
    change_set_id: Option<ChangeSetId>,
    payload: WsPayload,
}

impl WsEvent {
    pub async fn new_raw(
        workspace_pk: WorkspacePk,
        change_set_id: Option<ChangeSetId>,
        payload: WsPayload,
    ) -> WsEventResult<Self> {
        Ok(WsEvent {
            version: 1,
            workspace_pk,
            change_set_id,
            payload,
        })
    }
    pub async fn new(ctx: &DalContext, payload: WsPayload) -> WsEventResult<Self> {
        let workspace_pk = match ctx.tenancy().workspace_pk() {
            Some(pk) => pk,
            None => {
                return Err(WsEventError::NoWorkspaceInTenancy);
            }
        };
        let change_set_pk = ctx.change_set_id();
        Self::new_raw(workspace_pk, Some(change_set_pk), payload).await
    }

    pub async fn new_for_workspace(ctx: &DalContext, payload: WsPayload) -> WsEventResult<Self> {
        let workspace_pk = match ctx.tenancy().workspace_pk() {
            Some(pk) => pk,
            None => {
                return Err(WsEventError::NoWorkspaceInTenancy);
            }
        };
        Self::new_raw(workspace_pk, None, payload).await
    }

    pub fn workspace_pk(&self) -> WorkspacePk {
        self.workspace_pk
    }

    pub fn set_workspace_pk(&mut self, workspace_pk: WorkspacePk) {
        self.workspace_pk = workspace_pk;
    }

    pub fn set_change_set_id(&mut self, change_set_id: Option<ChangeSetId>) {
        self.change_set_id = change_set_id;
    }

    pub fn change_set_id(&self) -> Option<ChangeSetId> {
        self.change_set_id
    }

    fn workspace_subject(&self) -> String {
        format!("si.workspace_pk.{}.event", self.workspace_pk)
    }

    /// Publishes the [`event`](Self) to the [`NatsTxn`](si_data_nats::NatsTxn). When the
    /// transaction is committed, the [`event`](Self) will be published for external use.
    pub async fn publish_on_commit(&self, ctx: &DalContext) -> WsEventResult<()> {
        ctx.txns()
            .await?
            .nats()
            .publish(self.workspace_subject(), &self)
            .await?;
        Ok(())
    }

    /// Publishes the [`event`](Self) immediately to the Nats stream, without
    /// waiting for the transactions to commit. Care should be taken to avoid
    /// sending data to the frontend, such as object ids, that will only be
    /// valid if the transaction commits successfully.
    pub async fn publish_immediately(&self, ctx: &DalContext) -> WsEventResult<()> {
        ctx.txns()
            .await?
            .nats()
            .publish_immediately(self.workspace_subject(), &self)
            .await?;
        Ok(())
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ErrorPayload {
    id: Ulid,
    error: String,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FinishPayload {
    id: Ulid,
}

impl WsEvent {
    pub async fn async_error(ctx: &DalContext, id: Ulid, error: String) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::AsyncError(ErrorPayload { id, error })).await
    }
    pub async fn async_finish(ctx: &DalContext, id: Ulid) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::AsyncFinish(FinishPayload { id })).await
    }
    pub async fn async_finish_workspace(ctx: &DalContext, id: Ulid) -> WsEventResult<Self> {
        WsEvent::new_for_workspace(ctx, WsPayload::AsyncFinish(FinishPayload { id })).await
    }
}
