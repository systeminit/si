use std::collections::HashMap;

pub use UpdateMode::{
    SkipExisting,
    UpdateExisting,
};
use chrono::{
    DateTime,
    Utc,
};
pub use import::{
    ImportOptions,
    Thing,
    ThingMap,
    import_func,
    import_funcs_for_module_update,
    import_only_new_funcs,
    import_pkg,
    import_pkg_from_pkg,
    import_schema_variant,
};
use serde::{
    Deserialize,
    Serialize,
};
use si_pkg::{
    FuncSpecBackendKind,
    FuncSpecBackendResponseType,
    SiPkgError,
    SpecError,
};
use thiserror::Error;
use url::ParseError;

use crate::{
    AttributePrototypeId,
    DalContext,
    FuncBackendKind,
    FuncBackendResponseType,
    FuncId,
    OutputSocketId,
    PropId,
    PropKind,
    SchemaError,
    TransactionsError,
    UserPk,
    WorkspaceError,
    WorkspacePk,
    WsEvent,
    WsEventResult,
    WsPayload,
    action::prototype::ActionPrototypeError,
    attribute::{
        prototype::{
            AttributePrototypeError,
            argument::{
                AttributePrototypeArgumentError,
                AttributePrototypeArgumentId,
            },
        },
        value::AttributeValueError,
    },
    change_set::ChangeSetError,
    func::{
        FuncError,
        argument::{
            FuncArgumentError,
            FuncArgumentId,
        },
        binding::FuncBindingError,
    },
    management::prototype::ManagementPrototypeError,
    module::{
        ModuleError,
        ModulesUpdatedPayload,
    },
    prop::PropError,
    schema::variant::SchemaVariantError,
    socket::{
        connection_annotation::ConnectionAnnotationError,
        input::InputSocketError,
        output::OutputSocketError,
    },
    workspace_snapshot::WorkspaceSnapshotError,
};

pub mod export;
pub mod import;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum UpdateMode {
    #[default]
    SkipExisting,
    UpdateExisting,
}

#[remain::sorted]
#[derive(Debug, Error)]
pub enum PkgError {
    #[error("action prototype error: {0}")]
    ActionPrototype(#[from] Box<ActionPrototypeError>),
    #[error("attribute function for context {0:?} has key {1} but is not setting a prop value")]
    AttributeFuncForKeyMissingProp(import::AttrFuncContext, String),
    #[error("attribute function for prop {0} has a key {1} but prop kind is {2} not a map)")]
    AttributeFuncForKeySetOnWrongKind(PropId, String, PropKind),
    #[error("attribute prototype error: {0}")]
    AttributePrototype(#[from] Box<AttributePrototypeError>),
    #[error("attrbute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] Box<AttributePrototypeArgumentError>),
    #[error("AttributePrototypeArgument {0} missing FuncArgument {1}")]
    AttributePrototypeArgumentMissingFuncArgument(AttributePrototypeArgumentId, FuncArgumentId),
    #[error("attribute value error: {0}")]
    AttributeValueError(#[from] Box<AttributeValueError>),
    #[error("change set error: {0}")]
    ChangeSet(#[from] Box<ChangeSetError>),
    #[error("connection annotation error: {0}")]
    ConnectionAnnotation(#[from] Box<ConnectionAnnotationError>),
    #[error("expected data on an SiPkg node, but none found: {0}")]
    DataNotFound(String),
    #[error("func error: {0}")]
    Func(#[from] Box<FuncError>),
    #[error("func argument error: {0}")]
    FuncArgument(#[from] Box<FuncArgumentError>),
    #[error("func argument for {0} not found with name {1}")]
    FuncArgumentNotFoundByName(FuncId, String),
    #[error("action prototype error: {0}")]
    FuncBinding(#[from] Box<FuncBindingError>),
    #[error("input socket error: {0}")]
    InputSocket(#[from] Box<InputSocketError>),
    #[error("found multiple intrinsic func specs for name: {0}")]
    IntrinsicFuncSpecsMultipleForName(String),
    #[error("found no intrinsic func specs for name: {0}")]
    IntrinsicFuncSpecsNoneForName(String),
    #[error("management prototype error: {0}")]
    ManagementPrototype(#[from] Box<ManagementPrototypeError>),
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
    Module(#[from] Box<ModuleError>),
    #[error("output socket error: {0}")]
    OutputSocket(#[from] Box<OutputSocketError>),
    #[error("output socket {0} missing attribute prototype")]
    OutputSocketMissingPrototype(OutputSocketId),
    #[error("Package with that hash already installed: {0}")]
    PackageAlreadyInstalled(String),
    #[error("si pkg error: {0}")]
    Pkg(#[from] SiPkgError),
    #[error("pkg spec error: {0}")]
    PkgSpec(#[from] SpecError),
    #[error("prop error: {0}")]
    Prop(#[from] Box<PropError>),
    #[error("prop {0} missing attribute prototype")]
    PropMissingPrototype(PropId),
    #[error("prop {0} not found")]
    PropNotFoundByName(String),
    #[error("prop spec structure is invalid: {0}")]
    PropSpecChildrenInvalid(String),
    #[error("schema error: {0}")]
    Schema(#[from] Box<SchemaError>),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] Box<SchemaVariantError>),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error("si db error: {0}")]
    SiDb(#[from] si_db::SiDbError),
    #[error(
        "taking output socket as input for a prop is unsupported for name ({0}) and socket name ({1})"
    )]
    TakingOutputSocketAsInputForPropUnsupported(String, String),
    #[error("transactions error: {0}")]
    Transactions(#[from] Box<TransactionsError>),
    #[error("ulid decode error: {0}")]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("url parse error: {0}")]
    Url(#[from] ParseError),
    #[error("workspace error: {0}")]
    Workspace(#[from] Box<WorkspaceError>),
    #[error("workspace export not supported")]
    WorkspaceExportNotSupported(),
    #[error("workspace pk not found on context")]
    WorkspacePkNone,
    #[error("workspace snapshot error: {0}")]
    WorkspaceSnaphot(#[from] Box<WorkspaceSnapshotError>),
}

impl From<ActionPrototypeError> for PkgError {
    fn from(value: ActionPrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributePrototypeError> for PkgError {
    fn from(value: AttributePrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributePrototypeArgumentError> for PkgError {
    fn from(value: AttributePrototypeArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<AttributeValueError> for PkgError {
    fn from(value: AttributeValueError) -> Self {
        Box::new(value).into()
    }
}

impl From<ChangeSetError> for PkgError {
    fn from(value: ChangeSetError) -> Self {
        Box::new(value).into()
    }
}

impl From<ConnectionAnnotationError> for PkgError {
    fn from(value: ConnectionAnnotationError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncError> for PkgError {
    fn from(value: FuncError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncArgumentError> for PkgError {
    fn from(value: FuncArgumentError) -> Self {
        Box::new(value).into()
    }
}

impl From<FuncBindingError> for PkgError {
    fn from(value: FuncBindingError) -> Self {
        Box::new(value).into()
    }
}

impl From<InputSocketError> for PkgError {
    fn from(value: InputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<ManagementPrototypeError> for PkgError {
    fn from(value: ManagementPrototypeError) -> Self {
        Box::new(value).into()
    }
}

impl From<ModuleError> for PkgError {
    fn from(value: ModuleError) -> Self {
        Box::new(value).into()
    }
}

impl From<OutputSocketError> for PkgError {
    fn from(value: OutputSocketError) -> Self {
        Box::new(value).into()
    }
}

impl From<PropError> for PkgError {
    fn from(value: PropError) -> Self {
        Box::new(value).into()
    }
}

impl From<SchemaError> for PkgError {
    fn from(value: SchemaError) -> Self {
        Box::new(value).into()
    }
}

impl From<SchemaVariantError> for PkgError {
    fn from(value: SchemaVariantError) -> Self {
        Box::new(value).into()
    }
}

impl From<TransactionsError> for PkgError {
    fn from(value: TransactionsError) -> Self {
        Box::new(value).into()
    }
}

impl From<WorkspaceError> for PkgError {
    fn from(value: WorkspaceError) -> Self {
        Box::new(value).into()
    }
}

impl From<WorkspaceSnapshotError> for PkgError {
    fn from(value: WorkspaceSnapshotError) -> Self {
        Box::new(value).into()
    }
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
            FuncBackendKind::Debug => Self::Debug,
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
            FuncSpecBackendKind::Debug => Self::Debug,
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
            FuncBackendResponseType::Debug => Self::Debug,
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
            FuncSpecBackendResponseType::Debug => Self::Debug,
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
