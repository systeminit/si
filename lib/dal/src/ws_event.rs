use serde::{Deserialize, Serialize};
use si_data_nats::NatsError;
use si_data_pg::PgError;
use thiserror::Error;
use ulid::Ulid;

use crate::change_set::{ChangeSetActorPayload, ChangeSetMergeVotePayload};
use crate::component::{ComponentCreatedPayload, ComponentUpdatedPayload};
use crate::qualification::QualificationCheckPayload;
use crate::secret::{SecretCreatedPayload, SecretUpdatedPayload};
use crate::user::OnlinePayload;
use crate::{
    func::binding::LogLinePayload, pkg::ModuleImportedPayload, user::CursorPayload, ChangeSetPk,
    DalContext, PropId, StandardModelError, TransactionsError, WorkspacePk,
};

#[remain::sorted]
#[derive(Error, Debug)]
pub enum WsEventError {
    #[error("nats txn error: {0}")]
    Nats(#[from] NatsError),
    #[error("no user in context")]
    NoUserInContext,
    #[error("no workspace in tenancy")]
    NoWorkspaceInTenancy,
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
    //    ActionAdded(ActionAddedPayload),
    //    ActionRemoved(ActionRemovedPayload),
    AsyncError(ErrorPayload),
    AsyncFinish(FinishPayload),
    ChangeSetAbandoned(ChangeSetActorPayload),
    ChangeSetAbandonVote(ChangeSetMergeVotePayload),
    ChangeSetApplied(ChangeSetActorPayload),
    ChangeSetBeginAbandonProcess(ChangeSetActorPayload),
    ChangeSetBeginApprovalProcess(ChangeSetActorPayload),
    ChangeSetCancelAbandonProcess(ChangeSetActorPayload),
    ChangeSetCancelApprovalProcess(ChangeSetActorPayload),
    ChangeSetCanceled(ChangeSetPk),
    ChangeSetCreated(ChangeSetPk),
    ChangeSetMergeVote(ChangeSetMergeVotePayload),
    ChangeSetWritten(ChangeSetPk),
    CheckedQualifications(QualificationCheckPayload),
    // CodeGenerated(CodeGeneratedPayload),
    ComponentCreated(ComponentCreatedPayload),
    ComponentUpdated(ComponentUpdatedPayload),
    Cursor(CursorPayload),
    // FixBatchReturn(FixBatchReturn),
    // FixReturn(FixReturn),
    // ImportWorkspaceVote(ImportWorkspaceVotePayload),
    LogLine(LogLinePayload),
    ModuleImported(ModuleImportedPayload),
    Online(OnlinePayload),
    // ResourceRefreshed(ResourceRefreshedPayload),
    // SchemaCreated(SchemaPk),
    // SchemaVariantDefinitionCloned(SchemaVariantDefinitionClonedPayload),
    // SchemaVariantDefinitionCreated(SchemaVariantDefinitionCreatedPayload),
    // SchemaVariantDefinitionFinished(FinishSchemaVariantDefinitionPayload),
    // SchemaVariantDefinitionSaved(SchemaVariantDefinitionSavedPayload),
    SecretCreated(SecretCreatedPayload),
    SecretUpdated(SecretUpdatedPayload),
    // StatusUpdate(StatusMessage),
    // WorkspaceExported(WorkspaceExportPayload),
    // WorkspaceImportBeginApprovalProcess(WorkspaceImportApprovalActorPayload),
    // WorkspaceImportCancelApprovalProcess(WorkspaceActorPayload),
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

// #[derive(Deserialize, Serialize, Debug, Clone, Copy, Eq, Hash, PartialEq)]
// #[serde(rename_all = "camelCase")]
// pub struct AttributeValueStatusUpdate {
//     value_id: AttributeValueId,
//     component_id: ComponentId,
//     value_kind: StatusValueKind,
// }

// impl AttributeValueStatusUpdate {
//     pub fn new(
//         value_id: AttributeValueId,
//         component_id: ComponentId,
//         value_kind: StatusValueKind,
//     ) -> Self {
//         Self {
//             value_id,
//             component_id,
//             value_kind,
//         }
//     }
// }

#[derive(Deserialize, Serialize, Debug, Clone, Eq, PartialEq)]
pub struct WsEvent {
    version: i64,
    workspace_pk: WorkspacePk,
    change_set_pk: ChangeSetPk,
    payload: WsPayload,
}

impl WsEvent {
    pub async fn new_raw(
        workspace_pk: WorkspacePk,
        change_set_pk: ChangeSetPk,
        payload: WsPayload,
    ) -> WsEventResult<Self> {
        Ok(WsEvent {
            version: 1,
            workspace_pk,
            change_set_pk,
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
        let change_set_pk = ctx.visibility().change_set_pk;
        Self::new_raw(workspace_pk, change_set_pk, payload).await
    }

    pub fn workspace_pk(&self) -> WorkspacePk {
        self.workspace_pk
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
}

// #[remain::sorted]
// #[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
// #[serde(tag = "type", content = "data")]
// pub enum AttributePrototypeContextKind {
//     ExternalProvider { name: String },
//     Prop { path: String, kind: PropKind },
// }
//
// #[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
// pub struct AttributePrototypeView {
//     pub id: AttributePrototypeId,
//     pub func_id: FuncId,
//     pub func_name: String,
//     pub variant: Option<FuncVariant>,
//     pub key: Option<String>,
//     pub context: AttributePrototypeContextKind,
// }
//
// #[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
// pub struct FinishSchemaVariantDefinitionPayload {
//     pub task_id: Ulid,
//     pub schema_variant_id: SchemaVariantId,
//     pub detached_attribute_prototypes: Vec<AttributePrototypeView>,
// }
//
// impl WsEvent {
//     pub async fn schema_variant_definition_finish(
//         ctx: &DalContext,
//         payload: FinishSchemaVariantDefinitionPayload,
//     ) -> WsEventResult<Self> {
//         WsEvent::new(ctx, WsPayload::SchemaVariantDefinitionFinished(payload)).await
//     }
// }
