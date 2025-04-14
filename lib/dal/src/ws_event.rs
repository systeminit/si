use std::num::ParseIntError;

use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use si_events::Actor;
use si_frontend_types as frontend_types;
use thiserror::Error;
use ulid::Ulid;

use crate::approval_requirement::{
    ApprovalRequirementDefinitionCreatedPayload, ApprovalRequirementDefinitionRemovedPayload,
    IndividualApproverPayload,
};
use crate::audit_logging::AuditLogsPublishedPayload;
use crate::change_set::event::{
    ChangeSetActorPayload, ChangeSetAppliedPayload, ChangeSetMergeVotePayload,
    ChangeSetRenamePayload, ChangeSetStateChangePayload,
};
use crate::component::{
    ComponentCreatedPayload, ComponentDeletedPayload, ComponentSetPositionPayload,
    ComponentUpdatedPayload, ComponentUpgradedPayload, ConnectionDeletedPayload,
    ConnectionUpsertedPayload, InferredEdgeRemovePayload, InferredEdgeUpsertPayload,
};
use crate::diagram::view::{
    ViewComponentsUpdatePayload, ViewDeletedPayload, ViewObjectCreatedPayload,
    ViewObjectRemovedPayload, ViewWsPayload,
};
use crate::func::runner::FuncRunLogUpdatedPayload;
use crate::func::{
    FuncWsEventCodeSaved, FuncWsEventFuncSummary, FuncWsEventGenerating, FuncWsEventPayload,
};
use crate::management::prototype::{
    ManagementFuncExecutedPayload, ManagementOperationsCompletePayload,
};
use crate::module::ModulesUpdatedPayload;
use crate::pkg::{
    ImportWorkspaceVotePayload, WorkspaceActorPayload, WorkspaceImportApprovalActorPayload,
};
use crate::prompt_override::PromptUpdatedPayload;
use crate::qualification::QualificationCheckPayload;
use crate::schema::variant::{
    SchemaVariantClonedPayload, SchemaVariantDeletedPayload, SchemaVariantReplacedPayload,
    SchemaVariantSavedPayload, SchemaVariantUpdatedPayload, TemplateGeneratedPayload,
};
use crate::secret::SecretDeletedPayload;
use crate::status::StatusUpdate;
use crate::user::OnlinePayload;
use crate::{
    user::CursorPayload, ChangeSetId, DalContext, FuncError, PropId, StandardModelError,
    TransactionsError, WorkspacePk,
};
use crate::{SchemaVariantError, SecretCreatedPayload, SecretUpdatedPayload};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WsEventError {
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
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
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
    ActionsListUpdated(ChangeSetId),
    ApprovalRequirementAddIndividualApprover(IndividualApproverPayload),
    ApprovalRequirementDefinitionCreated(ApprovalRequirementDefinitionCreatedPayload),
    ApprovalRequirementDefinitionRemoved(ApprovalRequirementDefinitionRemovedPayload),
    ApprovalRequirementRemoveIndividualApprover(IndividualApproverPayload),
    AsyncError(ErrorPayload),
    AsyncFinish(FinishPayload),
    AuditLogsPublished(AuditLogsPublishedPayload),
    ChangeSetAbandoned(ChangeSetActorPayload),
    ChangeSetAbandonVote(ChangeSetMergeVotePayload),
    ChangeSetApplied(ChangeSetAppliedPayload),
    ChangeSetApprovalStatusChanged(ChangeSetId),
    ChangeSetBeginAbandonProcess(ChangeSetActorPayload),
    ChangeSetBeginApprovalProcess(ChangeSetActorPayload),
    ChangeSetCancelAbandonProcess(ChangeSetActorPayload),
    ChangeSetCancelApprovalProcess(ChangeSetActorPayload),
    ChangeSetCanceled(ChangeSetId),
    ChangeSetCreated(ChangeSetId),
    ChangeSetMergeVote(ChangeSetMergeVotePayload),
    ChangeSetRename(ChangeSetRenamePayload),
    ChangeSetStatusChanged(ChangeSetStateChangePayload),
    ChangeSetWritten(ChangeSetId),
    CheckedQualifications(QualificationCheckPayload),
    ComponentCreated(ComponentCreatedPayload),
    ComponentDeleted(ComponentDeletedPayload),
    ComponentUpdated(ComponentUpdatedPayload),
    ComponentUpgraded(ComponentUpgradedPayload),
    ConnectionDeleted(ConnectionDeletedPayload),
    ConnectionUpserted(ConnectionUpsertedPayload),
    Cursor(CursorPayload),
    FuncArgumentsSaved(FuncWsEventPayload),
    FuncCodeSaved(FuncWsEventCodeSaved),
    FuncCreated(FuncWsEventFuncSummary),
    FuncDeleted(FuncWsEventPayload),
    FuncGenerating(FuncWsEventGenerating),
    FuncRunLogUpdated(FuncRunLogUpdatedPayload),
    FuncSaved(FuncWsEventPayload),
    FuncUpdated(FuncWsEventFuncSummary),
    ImportWorkspaceVote(ImportWorkspaceVotePayload),
    InferredEdgeRemove(InferredEdgeRemovePayload),
    InferredEdgeUpsert(InferredEdgeUpsertPayload),
    ManagementFuncExecuted(ManagementFuncExecutedPayload),
    ManagementOperationsComplete(ManagementOperationsCompletePayload),
    ModuleImported(Vec<si_frontend_types::SchemaVariant>),
    ModulesUpdated(ModulesUpdatedPayload),
    Online(OnlinePayload),
    PromptUpdated(PromptUpdatedPayload),
    ResourceRefreshed(ComponentUpdatedPayload),
    SchemaVariantCloned(SchemaVariantClonedPayload),
    SchemaVariantCreated(frontend_types::SchemaVariant),
    SchemaVariantDeleted(SchemaVariantDeletedPayload),
    SchemaVariantReplaced(SchemaVariantReplacedPayload),
    SchemaVariantSaved(SchemaVariantSavedPayload),
    SchemaVariantUpdated(frontend_types::SchemaVariant),
    SchemaVariantUpdateFinished(SchemaVariantUpdatedPayload),
    SecretCreated(SecretCreatedPayload),
    SecretDeleted(SecretDeletedPayload),
    SecretUpdated(SecretUpdatedPayload),
    SetComponentPosition(ComponentSetPositionPayload),
    StatusUpdate(StatusUpdate),
    TemplateGenerated(TemplateGeneratedPayload),
    ViewComponentsUpdate(ViewComponentsUpdatePayload),
    ViewCreated(ViewWsPayload),
    ViewDeleted(ViewDeletedPayload),
    ViewObjectCreated(ViewObjectCreatedPayload),
    ViewObjectRemoved(ViewObjectRemovedPayload),
    ViewUpdated(ViewWsPayload),
    WorkspaceImportBeginApprovalProcess(WorkspaceImportApprovalActorPayload),
    WorkspaceImportCancelApprovalProcess(WorkspaceActorPayload),
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
    actor: Option<Actor>,
    request_ulid: Option<ulid::Ulid>,
    payload: WsPayload,
}

impl WsEvent {
    pub async fn new_raw(
        workspace_pk: WorkspacePk,
        change_set_id: Option<ChangeSetId>,
        actor: Option<Actor>,
        request_ulid: Option<ulid::Ulid>,
        payload: WsPayload,
    ) -> WsEventResult<Self> {
        Ok(WsEvent {
            version: 1,
            workspace_pk,
            change_set_id,
            actor,
            request_ulid,
            payload,
        })
    }
    pub async fn new(ctx: &DalContext, payload: WsPayload) -> WsEventResult<Self> {
        let workspace_pk = match ctx.tenancy().workspace_pk_opt() {
            Some(pk) => pk,
            None => {
                return Err(WsEventError::NoWorkspaceInTenancy);
            }
        };
        let change_set_pk = ctx.change_set_id();
        Self::new_raw(
            workspace_pk,
            Some(change_set_pk),
            Some(ctx.events_actor()),
            ctx.request_ulid(),
            payload,
        )
        .await
    }

    pub async fn new_for_workspace(ctx: &DalContext, payload: WsPayload) -> WsEventResult<Self> {
        let workspace_pk = match ctx.tenancy().workspace_pk_opt() {
            Some(pk) => pk,
            None => {
                return Err(WsEventError::NoWorkspaceInTenancy);
            }
        };
        Self::new_raw(
            workspace_pk,
            None,
            Some(ctx.events_actor()),
            ctx.request_ulid(),
            payload,
        )
        .await
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
