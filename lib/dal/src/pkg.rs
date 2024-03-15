use serde::{Deserialize, Serialize};
use si_pkg::{FuncSpecBackendKind, FuncSpecBackendResponseType, SiPkgError, SpecError};
use std::collections::HashMap;
use thiserror::Error;

use crate::attribute::prototype::argument::AttributePrototypeArgumentError;
use crate::attribute::prototype::AttributePrototypeError;
use crate::schema::variant::SchemaVariantError;
use crate::{
    change_set::ChangeSetError,
    func::{argument::FuncArgumentError, FuncError},
    installed_pkg::InstalledPkgError,
    prop::PropError,
    socket::input::InputSocketError,
    socket::output::OutputSocketError,
    workspace_snapshot::WorkspaceSnapshotError,
    ActionPrototypeError, ChangeSetId, DalContext, FuncBackendKind, FuncBackendResponseType,
    OutputSocketId, SchemaError, SchemaVariantId, TransactionsError,
};
use crate::{FuncId, PropId, PropKind};

use crate::socket::connection_annotation::ConnectionAnnotationError;
pub use import::{import_pkg, import_pkg_from_pkg, ImportOptions};

mod import;

// mod export;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum PkgError {
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("attribute function for context {0:?} has key {1} but is not setting a prop value")]
    AttributeFuncForKeyMissingProp(import::AttrFuncContext, String),
    #[error("attribute function for prop {0} has a key {1} but prop kind is {2} not a map)")]
    AttributeFuncForKeySetOnWrongKind(PropId, String, PropKind),
    #[error(transparent)]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attrbute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error(transparent)]
    ChangeSetPointer(#[from] ChangeSetError),
    #[error(transparent)]
    ConnectionAnnotation(#[from] ConnectionAnnotationError),
    #[error("expected data on an SiPkg node, but none found: {0}")]
    DataNotFound(String),
    #[error(transparent)]
    Func(#[from] FuncError),
    #[error(transparent)]
    FuncArgument(#[from] FuncArgumentError),
    #[error("func argument for {0} not found with name {1}")]
    FuncArgumentNotFoundByName(FuncId, String),
    #[error("func {0} could not be found by name")]
    FuncNotFoundByName(String),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error(transparent)]
    InstalledPkg(#[from] InstalledPkgError),
    #[error("Cannot find FuncArgument {0} for Func {1}")]
    MissingFuncArgument(String, FuncId),
    #[error("Package asked for a function with the unique id {0} but none could be found")]
    MissingFuncUniqueId(String),
    #[error("Cannot find InputSocket for name: {0}")]
    MissingInputSocketName(String),
    #[error("Unique id missing for node in workspace backup: {0}")]
    MissingUniqueIdForNode(String),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error("output socket {0} missing attribute prototype")]
    OutputSocketMissingPrototype(OutputSocketId),
    #[error("Package with that hash already installed: {0}")]
    PackageAlreadyInstalled(String),
    #[error(transparent)]
    Pkg(#[from] SiPkgError),
    #[error(transparent)]
    PkgSpec(#[from] SpecError),
    #[error(transparent)]
    Prop(#[from] PropError),
    #[error("prop {0} missing attribute prototype")]
    PropMissingPrototype(PropId),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error(transparent)]
    WorkspaceSnaphot(#[from] WorkspaceSnapshotError),
}

pub type PkgResult<T> = Result<T, PkgError>;

impl From<FuncBackendKind> for FuncSpecBackendKind {
    fn from(value: FuncBackendKind) -> Self {
        match value {
            FuncBackendKind::Array => Self::Array,
            FuncBackendKind::Boolean => Self::Boolean,
            FuncBackendKind::Diff => Self::Diff,
            FuncBackendKind::Identity => Self::Identity,
            FuncBackendKind::Integer => Self::Integer,
            FuncBackendKind::JsAction => Self::JsAction,
            FuncBackendKind::JsAttribute => Self::JsAttribute,
            FuncBackendKind::JsReconciliation => Self::JsReconciliation,
            FuncBackendKind::JsSchemaVariantDefinition => Self::JsSchemaVariantDefinition,
            FuncBackendKind::JsValidation => Self::JsValidation,
            FuncBackendKind::Map => Self::Map,
            FuncBackendKind::Object => Self::Object,
            FuncBackendKind::String => Self::String,
            FuncBackendKind::Unset => Self::Unset,
            FuncBackendKind::Validation => Self::Validation,
            FuncBackendKind::JsAuthentication => Self::JsAuthentication,
        }
    }
}

impl From<FuncSpecBackendKind> for FuncBackendKind {
    fn from(value: FuncSpecBackendKind) -> Self {
        match value {
            FuncSpecBackendKind::Array => Self::Array,
            FuncSpecBackendKind::Boolean => Self::Boolean,
            FuncSpecBackendKind::Diff => Self::Diff,
            FuncSpecBackendKind::Identity => Self::Identity,
            FuncSpecBackendKind::Integer => Self::Integer,
            FuncSpecBackendKind::JsAction => Self::JsAction,
            FuncSpecBackendKind::JsAttribute => Self::JsAttribute,
            FuncSpecBackendKind::JsReconciliation => Self::JsReconciliation,
            FuncSpecBackendKind::JsSchemaVariantDefinition => Self::JsSchemaVariantDefinition,
            FuncSpecBackendKind::JsValidation => Self::JsValidation,
            FuncSpecBackendKind::Map => Self::Map,
            FuncSpecBackendKind::Object => Self::Object,
            FuncSpecBackendKind::String => Self::String,
            FuncSpecBackendKind::Unset => Self::Unset,
            FuncSpecBackendKind::Validation => Self::Validation,
            FuncSpecBackendKind::JsAuthentication => Self::JsAuthentication,
        }
    }
}

impl From<FuncBackendResponseType> for FuncSpecBackendResponseType {
    fn from(value: FuncBackendResponseType) -> Self {
        match value {
            FuncBackendResponseType::Action => Self::Action,
            FuncBackendResponseType::Array => Self::Array,
            FuncBackendResponseType::Boolean => Self::Boolean,
            FuncBackendResponseType::CodeGeneration => Self::CodeGeneration,
            FuncBackendResponseType::Identity => Self::Identity,
            FuncBackendResponseType::Integer => Self::Integer,
            FuncBackendResponseType::Json => Self::Json,
            FuncBackendResponseType::Map => Self::Map,
            FuncBackendResponseType::Object => Self::Object,
            FuncBackendResponseType::Qualification => Self::Qualification,
            FuncBackendResponseType::Reconciliation => Self::Reconciliation,
            FuncBackendResponseType::SchemaVariantDefinition => Self::SchemaVariantDefinition,
            FuncBackendResponseType::String => Self::String,
            FuncBackendResponseType::Unset => Self::Unset,
            FuncBackendResponseType::Validation => Self::Validation,
            FuncBackendResponseType::Void => Self::Void,
        }
    }
}

impl From<FuncSpecBackendResponseType> for FuncBackendResponseType {
    fn from(value: FuncSpecBackendResponseType) -> Self {
        match value {
            FuncSpecBackendResponseType::Action => Self::Action,
            FuncSpecBackendResponseType::Array => Self::Array,
            FuncSpecBackendResponseType::Boolean => Self::Boolean,
            FuncSpecBackendResponseType::CodeGeneration => Self::CodeGeneration,
            FuncSpecBackendResponseType::Identity => Self::Identity,
            FuncSpecBackendResponseType::Integer => Self::Integer,
            FuncSpecBackendResponseType::Json => Self::Json,
            FuncSpecBackendResponseType::Map => Self::Map,
            FuncSpecBackendResponseType::Object => Self::Object,
            FuncSpecBackendResponseType::Qualification => Self::Qualification,
            FuncSpecBackendResponseType::Reconciliation => Self::Reconciliation,
            FuncSpecBackendResponseType::SchemaVariantDefinition => Self::SchemaVariantDefinition,
            FuncSpecBackendResponseType::String => Self::String,
            FuncSpecBackendResponseType::Unset => Self::Unset,
            FuncSpecBackendResponseType::Validation => Self::Validation,
            FuncSpecBackendResponseType::Void => Self::Void,
        }
    }
}

/// A generic hash map of hash maps for tracking the presence of a thing in each change set. If a
/// thing is asked for in a specific change set, and not found, the HEAD change set will be
/// checked.
pub struct ChangeSetThingMap<Key, Thing> {
    inner: HashMap<ChangeSetId, HashMap<Key, Thing>>,
    default_change_set_id: ChangeSetId,
}

// TODO(nick): we need a better strategy for tracking the head change set in the thing map.
impl<Key, Thing> ChangeSetThingMap<Key, Thing>
where
    Key: Eq + PartialEq + std::hash::Hash,
{
    pub async fn new(ctx: &DalContext) -> PkgResult<Self> {
        let head_thing_map = HashMap::new();

        let default_change_set_id = ctx.get_workspace_default_change_set_id().await?;

        let mut change_set_map = HashMap::new();
        change_set_map.insert(default_change_set_id, head_thing_map);

        Ok(Self {
            inner: change_set_map,
            default_change_set_id,
        })
    }

    pub fn get(&self, change_set_id: Option<ChangeSetId>, key: &Key) -> Option<&Thing> {
        match self
            .inner
            .get(&change_set_id.unwrap_or(self.default_change_set_id))
        {
            Some(change_set_map) => change_set_map.get(key).or_else(|| {
                self.inner
                    .get(&self.default_change_set_id)
                    .and_then(|things| things.get(key))
            }),
            None => self
                .inner
                .get(&self.default_change_set_id)
                .and_then(|things| things.get(key)),
        }
    }

    pub fn insert(
        &mut self,
        change_set_id: Option<ChangeSetId>,
        key: Key,
        thing: Thing,
    ) -> Option<Thing> {
        self.inner
            .entry(change_set_id.unwrap_or(self.default_change_set_id))
            .or_default()
            .insert(key, thing)
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub struct ModuleImportedPayload {
    schema_variant_ids: Vec<SchemaVariantId>,
}

// #[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
// pub struct WorkspaceImportPayload {
//     workspace_pk: Option<WorkspacePk>,
//     user_pk: Option<UserPk>,
// }
//
// #[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
// pub struct WorkspaceExportPayload {
//     workspace_pk: Option<WorkspacePk>,
//     user_pk: Option<UserPk>,
// }
//
// #[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
// pub struct ImportWorkspaceVotePayload {
//     workspace_pk: Option<WorkspacePk>,
//     user_pk: UserPk,
//     vote: String,
// }
//
// #[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
// pub struct WorkspaceActorPayload {
//     workspace_pk: Option<WorkspacePk>,
//     user_pk: Option<UserPk>,
// }
//
// #[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
// #[serde(rename_all = "camelCase")]
// pub struct WorkspaceImportApprovalActorPayload {
//     workspace_pk: Option<WorkspacePk>,
//     user_pk: Option<UserPk>,
//     created_at: DateTime<Utc>,
//     created_by: String,
//     name: String,
// }
//
// impl WsEvent {
//     pub async fn module_imported(
//         ctx: &DalContext,
//         schema_variant_ids: Vec<SchemaVariantId>,
//     ) -> WsEventResult<Self> {
//         WsEvent::new(
//             ctx,
//             WsPayload::ModuleImported(ModuleImportedPayload { schema_variant_ids }),
//         )
//         .await
//     }
//
//     pub async fn workspace_imported(
//         ctx: &DalContext,
//         workspace_pk: Option<WorkspacePk>,
//         user_pk: Option<UserPk>,
//     ) -> WsEventResult<Self> {
//         WsEvent::new(
//             ctx,
//             WsPayload::WorkspaceImported(WorkspaceImportPayload {
//                 workspace_pk,
//                 user_pk,
//             }),
//         )
//         .await
//     }
//
//     pub async fn workspace_exported(
//         ctx: &DalContext,
//         workspace_pk: Option<WorkspacePk>,
//         user_pk: Option<UserPk>,
//     ) -> WsEventResult<Self> {
//         WsEvent::new(
//             ctx,
//             WsPayload::WorkspaceExported(WorkspaceExportPayload {
//                 workspace_pk,
//                 user_pk,
//             }),
//         )
//         .await
//     }
//
//     pub async fn import_workspace_vote(
//         ctx: &DalContext,
//         workspace_pk: Option<WorkspacePk>,
//         user_pk: UserPk,
//         vote: String,
//     ) -> WsEventResult<Self> {
//         WsEvent::new(
//             ctx,
//             WsPayload::ImportWorkspaceVote(ImportWorkspaceVotePayload {
//                 workspace_pk,
//                 user_pk,
//                 vote,
//             }),
//         )
//         .await
//     }
//
//     pub async fn workspace_import_begin_approval_process(
//         ctx: &DalContext,
//         workspace_pk: Option<WorkspacePk>,
//         user_pk: Option<UserPk>,
//         workspace_export_created_at: DateTime<Utc>,
//         workspace_export_created_by: String,
//         workspace_export_name: String,
//     ) -> WsEventResult<Self> {
//         WsEvent::new(
//             ctx,
//             WsPayload::WorkspaceImportBeginApprovalProcess(WorkspaceImportApprovalActorPayload {
//                 workspace_pk,
//                 user_pk,
//                 created_at: workspace_export_created_at,
//                 created_by: workspace_export_created_by,
//                 name: workspace_export_name,
//             }),
//         )
//         .await
//     }
//
//     pub async fn workspace_import_cancel_approval_process(
//         ctx: &DalContext,
//         workspace_pk: Option<WorkspacePk>,
//         user_pk: Option<UserPk>,
//     ) -> WsEventResult<Self> {
//         WsEvent::new(
//             ctx,
//             WsPayload::WorkspaceImportCancelApprovalProcess(WorkspaceActorPayload {
//                 workspace_pk,
//                 user_pk,
//             }),
//         )
//         .await
//     }
// }
