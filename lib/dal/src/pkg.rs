use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use si_pkg::{FuncSpecBackendKind, FuncSpecBackendResponseType, SiPkgError, SpecError};
use std::collections::HashMap;
use thiserror::Error;
use url::ParseError;

use crate::attribute::prototype::AttributePrototypeError;
use crate::attribute::prototype::argument::{
    AttributePrototypeArgumentError, AttributePrototypeArgumentId,
};
use crate::attribute::value::AttributeValueError;
use crate::func::argument::FuncArgumentId;
use crate::management::prototype::ManagementPrototypeError;
use crate::schema::variant::SchemaVariantError;
use crate::{AttributePrototypeId, FuncId, HistoryEventError, PropId, PropKind};
use crate::{
    DalContext, FuncBackendKind, FuncBackendResponseType, OutputSocketId, SchemaError,
    TransactionsError, UserPk, WorkspaceError, WorkspacePk, WsEvent, WsEventResult, WsPayload,
    action::prototype::ActionPrototypeError,
    change_set::ChangeSetError,
    func::{FuncError, argument::FuncArgumentError},
    prop::PropError,
    socket::input::InputSocketError,
    socket::output::OutputSocketError,
    workspace_snapshot::WorkspaceSnapshotError,
};

use crate::module::{ModuleError, ModulesUpdatedPayload};
use crate::socket::connection_annotation::ConnectionAnnotationError;
pub use import::{ImportOptions, import_pkg, import_pkg_from_pkg};

pub mod export;
pub mod import;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum PkgError {
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] ActionPrototypeError),
    #[error("attribute function for context {0:?} has key {1} but is not setting a prop value")]
    AttributeFuncForKeyMissingProp(import::AttrFuncContext, String),
    #[error("attribute function for prop {0} has a key {1} but prop kind is {2} not a map)")]
    AttributeFuncForKeySetOnWrongKind(PropId, String, PropKind),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attrbute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("AttributePrototypeArgument {0} missing FuncArgument {1}")]
    AttributePrototypeArgumentMissingFuncArgument(AttributePrototypeArgumentId, FuncArgumentId),
    #[error("attribute value error: {0}")]
    AttributeValueError(#[from] AttributeValueError),
    #[error("change set error: {0}")]
    ChangeSet(#[from] ChangeSetError),
    #[error("connection annotation error: {0}")]
    ConnectionAnnotation(#[from] ConnectionAnnotationError),
    #[error("expected data on an SiPkg node, but none found: {0}")]
    DataNotFound(String),
    #[error("func error: {0}")]
    Func(#[from] FuncError),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] FuncArgumentError),
    #[error("func argument for {0} not found with name {1}")]
    FuncArgumentNotFoundByName(FuncId, String),
    #[error("history event error: {0}")]
    HistoryEvent(#[from] HistoryEventError),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error("found multiple intrinsic func specs for name: {0}")]
    IntrinsicFuncSpecsMultipleForName(String),
    #[error("found no intrinsic func specs for name: {0}")]
    IntrinsicFuncSpecsNoneForName(String),
    #[error("management prototype error: {0}")]
    ManagementPrototype(#[from] ManagementPrototypeError),
    #[error("Missing Func {1} for AttributePrototype {0}")]
    MissingAttributePrototypeFunc(AttributePrototypeId, FuncId),
    #[error("Func {0} missing from exported funcs")]
    MissingExportedFunc(FuncId),
    #[error("Cannot find FuncArgument {0} for Func {1}")]
    MissingFuncArgument(String, FuncId),
    #[error("Package asked for a function with the unique id {0} but none could be found ({1})")]
    MissingFuncUniqueId(String, &'static str),
    #[error("Cannot find InputSocket for name: {0}")]
    MissingInputSocketName(String),
    #[error("Intrinsic function {0} not found")]
    MissingIntrinsicFunc(String),
    #[error("Unique id missing for node in workspace backup: {0}")]
    MissingUniqueIdForNode(String),
    #[error("module error: {0}")]
    Module(#[from] ModuleError),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] OutputSocketError),
    #[error("output socket {0} missing attribute prototype")]
    OutputSocketMissingPrototype(OutputSocketId),
    #[error("Package with that hash already installed: {0}")]
    PackageAlreadyInstalled(String),
    #[error("si pkg error: {0}")]
    Pkg(#[from] SiPkgError),
    #[error("pkg spec error: {0}")]
    PkgSpec(#[from] SpecError),
    #[error("prop error: {0}")]
    Prop(#[from] PropError),
    #[error("prop {0} missing attribute prototype")]
    PropMissingPrototype(PropId),
    #[error("prop {0} not found")]
    PropNotFoundByName(String),
    #[error("prop spec structure is invalid: {0}")]
    PropSpecChildrenInvalid(String),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(
        "taking output socket as input for a prop is unsupported for name ({0}) and socket name ({1})"
    )]
    TakingOutputSocketAsInputForPropUnsupported(String, String),
    #[error("transactions error: {0}")]
    Transactions(#[from] TransactionsError),
    #[error("ulid decode error: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("url parse error: {0}")]
    Url(#[from] ParseError),
    #[error("workspace error: {0}")]
    Workspace(#[from] WorkspaceError),
    #[error("workspace export not supported")]
    WorkspaceExportNotSupported(),
    #[error("workspace pk not found on context")]
    WorkspacePkNone,
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnaphot(#[from] WorkspaceSnapshotError),
}

pub type PkgResult<T> = Result<T, PkgError>;

impl From<FuncBackendKind> for FuncSpecBackendKind {
    fn from(value: FuncBackendKind) -> Self {
        match value {
            FuncBackendKind::Array => Self::Array,
            FuncBackendKind::Boolean => Self::Boolean,
            FuncBackendKind::Diff => Self::Diff,
            FuncBackendKind::Float => Self::Float,
            FuncBackendKind::Identity => Self::Identity,
            FuncBackendKind::Integer => Self::Integer,
            FuncBackendKind::JsAction => Self::JsAction,
            FuncBackendKind::JsAttribute => Self::JsAttribute,
            FuncBackendKind::Json => Self::Json,
            FuncBackendKind::JsReconciliation => Self::JsReconciliation,
            FuncBackendKind::JsSchemaVariantDefinition => Self::JsSchemaVariantDefinition,
            FuncBackendKind::JsValidation => Self::JsValidation,
            FuncBackendKind::Map => Self::Map,
            FuncBackendKind::Object => Self::Object,
            FuncBackendKind::String => Self::String,
            FuncBackendKind::Unset => Self::Unset,
            FuncBackendKind::Validation => Self::Validation,
            FuncBackendKind::JsAuthentication => Self::JsAuthentication,
            FuncBackendKind::Management => Self::Management,
            FuncBackendKind::ResourcePayloadToValue => Self::ResourcePayloadToValue,
            FuncBackendKind::NormalizeToArray => Self::NormalizeToArray,
        }
    }
}

impl From<FuncSpecBackendKind> for FuncBackendKind {
    fn from(value: FuncSpecBackendKind) -> Self {
        match value {
            FuncSpecBackendKind::Array => Self::Array,
            FuncSpecBackendKind::Boolean => Self::Boolean,
            FuncSpecBackendKind::Diff => Self::Diff,
            FuncSpecBackendKind::Float => Self::Float,
            FuncSpecBackendKind::Identity => Self::Identity,
            FuncSpecBackendKind::Integer => Self::Integer,
            FuncSpecBackendKind::JsAction => Self::JsAction,
            FuncSpecBackendKind::JsAttribute => Self::JsAttribute,
            FuncSpecBackendKind::Json => Self::Json,
            FuncSpecBackendKind::JsReconciliation => Self::JsReconciliation,
            FuncSpecBackendKind::JsSchemaVariantDefinition => Self::JsSchemaVariantDefinition,
            FuncSpecBackendKind::JsValidation => Self::JsValidation,
            FuncSpecBackendKind::Map => Self::Map,
            FuncSpecBackendKind::Object => Self::Object,
            FuncSpecBackendKind::String => Self::String,
            FuncSpecBackendKind::Unset => Self::Unset,
            FuncSpecBackendKind::Validation => Self::Validation,
            FuncSpecBackendKind::JsAuthentication => Self::JsAuthentication,
            FuncSpecBackendKind::Management => Self::Management,
            FuncSpecBackendKind::ResourcePayloadToValue => Self::ResourcePayloadToValue,
            FuncSpecBackendKind::NormalizeToArray => Self::NormalizeToArray,
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
            FuncBackendResponseType::Float => Self::Float,
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
            FuncBackendResponseType::Management => Self::Management,
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
            FuncSpecBackendResponseType::Float => Self::Float,
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
            FuncSpecBackendResponseType::Management => Self::Management,
        }
    }
}

/// A generic hash map of hash maps for tracking the presence of a thing in each change set. If a
/// thing is asked for in a specific change set, and not found, the HEAD change set will be
/// checked.
#[derive(Debug)]
pub struct ChangeSetThingMap<Key, Thing> {
    inner: HashMap<Key, Thing>,
    overrides: HashMap<Key, Key>,
}

impl<Key, Thing> Default for ChangeSetThingMap<Key, Thing>
where
    Key: Eq + PartialEq + std::hash::Hash,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<Key, Thing> ChangeSetThingMap<Key, Thing>
where
    Key: Eq + PartialEq + std::hash::Hash,
{
    pub fn new() -> Self {
        Self {
            inner: HashMap::new(),
            overrides: HashMap::new(),
        }
    }

    pub fn get(&self, key: &Key) -> Option<&Thing> {
        match self.overrides.get(key) {
            Some(override_key) => self.inner.get(override_key),
            None => self.inner.get(key),
        }
    }

    pub fn insert(&mut self, key: Key, thing: Thing) -> Option<Thing> {
        self.inner.insert(key, thing)
    }

    pub fn insert_override(&mut self, to_override_unique_id: Key, override_unique_id: Key) {
        self.overrides
            .insert(to_override_unique_id, override_unique_id);
    }
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceImportPayload {
    workspace_pk: Option<WorkspacePk>,
    user_pk: Option<UserPk>,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ImportWorkspaceVotePayload {
    workspace_pk: Option<WorkspacePk>,
    user_pk: UserPk,
    vote: String,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceActorPayload {
    workspace_pk: Option<WorkspacePk>,
    user_pk: Option<UserPk>,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceImportApprovalActorPayload {
    workspace_pk: Option<WorkspacePk>,
    user_pk: Option<UserPk>,
    created_at: DateTime<Utc>,
    created_by: String,
    name: String,
}

impl WsEvent {
    pub async fn module_imported(
        ctx: &DalContext,
        schema_variants: Vec<si_frontend_types::SchemaVariant>,
    ) -> WsEventResult<Self> {
        WsEvent::new(ctx, WsPayload::ModuleImported(schema_variants)).await
    }

    pub async fn modules_updated(ctx: &DalContext) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ModulesUpdated(ModulesUpdatedPayload {
                change_set_id: ctx.change_set_id(),
            }),
        )
        .await
    }

    pub async fn import_workspace_vote(
        ctx: &DalContext,
        workspace_pk: Option<WorkspacePk>,
        user_pk: UserPk,
        vote: String,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ImportWorkspaceVote(ImportWorkspaceVotePayload {
                workspace_pk,
                user_pk,
                vote,
            }),
        )
        .await
    }

    pub async fn workspace_import_begin_approval_process(
        ctx: &DalContext,
        workspace_pk: Option<WorkspacePk>,
        user_pk: Option<UserPk>,
        workspace_export_created_at: DateTime<Utc>,
        workspace_export_created_by: String,
        workspace_export_name: String,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::WorkspaceImportBeginApprovalProcess(WorkspaceImportApprovalActorPayload {
                workspace_pk,
                user_pk,
                created_at: workspace_export_created_at,
                created_by: workspace_export_created_by,
                name: workspace_export_name,
            }),
        )
        .await
    }

    pub async fn workspace_import_cancel_approval_process(
        ctx: &DalContext,
        workspace_pk: Option<WorkspacePk>,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::WorkspaceImportCancelApprovalProcess(WorkspaceActorPayload {
                workspace_pk,
                user_pk,
            }),
        )
        .await
    }
}
