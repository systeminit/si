use serde::{Deserialize, Serialize};
use si_pkg::{FuncSpecBackendKind, FuncSpecBackendResponseType, SiPkgError, SpecError};
use std::collections::HashMap;
use thiserror::Error;

use crate::attribute::prototype::argument::AttributePrototypeArgumentError;
use crate::attribute::prototype::AttributePrototypeError;
use crate::schema::variant::SchemaVariantError;
use crate::{
    change_set_pointer::ChangeSetPointerError,
    func::{argument::FuncArgumentError, FuncError},
    installed_pkg::InstalledPkgError,
    prop::PropError,
    socket::input::InputSocketError,
    socket::output::OutputSocketError,
    workspace_snapshot::WorkspaceSnapshotError,
    ActionPrototypeError, ChangeSetPk, FuncBackendKind, FuncBackendResponseType, OutputSocketId,
    SchemaError, SchemaVariantId,
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
    // #[error(transparent)]
    // AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("attribute function for context {0:?} has key {1} but is not setting a prop value")]
    AttributeFuncForKeyMissingProp(import::AttrFuncContext, String),
    #[error("attribute function for prop {0} has a key {1} but prop kind is {2} not a map)")]
    AttributeFuncForKeySetOnWrongKind(PropId, String, PropKind),
    #[error(transparent)]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error("attrbute prototype argument error: {0}")]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    // #[error(transparent)]
    // AttributeValue(#[from] AttributeValueError),
    // #[error("parent prop could not be found with path: {0}")]
    // AttributeValueParentPropNotFound(String),
    // #[error("parent value could not be found for prop path: {0} and key {1:?}, index {2:?}")]
    // AttributeValueParentValueNotFound(String, Option<String>, Option<i64>),
    // #[error("attribute value is a proxy but there is no value to proxy")]
    // AttributeValueSetToProxyButNoProxyFound,
    // #[error("encountered an attribute value with a key or index but no parent")]
    // AttributeValueWithKeyOrIndexButNoParent,
    // #[error(transparent)]
    // ChangeSet(#[from] ChangeSetError),
    // #[error("change set {0} not found")]
    // ChangeSetNotFound(ChangeSetPk),
    #[error(transparent)]
    ChangeSetPointer(#[from] ChangeSetPointerError),
    #[error(transparent)]
    ConnectionAnnotation(#[from] ConnectionAnnotationError),
    #[error("expected data on an SiPkg node, but none found: {0}")]
    DataNotFound(String),
    // #[error(transparent)]
    // Edge(#[from] EdgeError),
    // #[error("edge refers to component not in export: {0}")]
    // EdgeRefersToMissingComponent(ComponentId),
    #[error(transparent)]
    Func(#[from] FuncError),
    #[error(transparent)]
    FuncArgument(#[from] FuncArgumentError),
    #[error("func argument for {0} not found with name {1}")]
    FuncArgumentNotFoundByName(FuncId, String),
    #[error("func {0} could not be found by name")]
    FuncNotFoundByName(String),
    // #[error(transparent)]
    // FuncBinding(#[from] FuncBindingError),
    // #[error(transparent)]
    // FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    // #[error(transparent)]
    // FuncExecution(#[from] crate::func::execution::FuncExecutionError),
    // #[error("Installed func id {0} does not exist")]
    // InstalledFuncMissing(FuncId),
    #[error("input socket error: {0}")]
    InputSocket(#[from] InputSocketError),
    #[error(transparent)]
    InstalledPkg(#[from] InstalledPkgError),
    // #[error("Installed schema variant definition {0} does not exist")]
    // InstalledSchemaVariantDefinitionMissing(SchemaVariantDefinitionId),
    // #[error("Installed schema variant {0} does not exist")]
    // InstalledSchemaVariantMissing(SchemaVariantId),
    // #[error("Leaf Function {0} has invalid argument {1}")]
    // InvalidLeafArgument(FuncId, String),
    // #[error("Missing Func {1} for AttributePrototype {0}")]
    // MissingAttributePrototypeFunc(AttributePrototypeId, FuncId),
    // #[error("Missing value for context {0:?}")]
    // MissingAttributeValueForContext(AttributeReadContext),
    // #[error("Missing a func map for changeset {0}")]
    // MissingChangeSetFuncMap(ChangeSetPk),
    // #[error("Missing component {0} for edge from {1} to {2}")]
    // MissingComponentForEdge(String, String, String),
    // #[error("Func {0} missing from exported funcs")]
    // MissingExportedFunc(FuncId),
    #[error("Cannot find FuncArgument {0} for Func {1}")]
    MissingFuncArgument(String, FuncId),
    // #[error("Cannot find FuncArgument {0}")]
    // MissingFuncArgumentById(FuncArgumentId),
    #[error("Package asked for a function with the unique id {0} but none could be found")]
    MissingFuncUniqueId(String),
    #[error("Cannot find InputSocket for name: {0}")]
    MissingInputSocketName(String),
    // #[error("Intrinsic function {0} not found")]
    // MissingIntrinsicFunc(String),
    // #[error("Intrinsic function (0) argument {1} not found")]
    // MissingIntrinsicFuncArgument(String, String),
    // #[error("Cannot find item prop for installed map prop {0}")]
    // MissingItemPropForMapProp(PropId),
    // #[error("Cannot find installed prop {0}")]
    // MissingProp(PropId),
    // #[error("Cannot find root prop for variant {0}")]
    // MissingRootProp(SchemaVariantId),
    // #[error("Cannot find schema_variant_definition {0}")]
    // MissingSchemaVariantDefinition(SchemaVariantId),
    // #[error("Cannot find socket with name {0} for edge kind {1}")]
    // MissingSocketName(String, SocketEdgeKind),
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
    // #[error("prop spec structure is invalid: {0}")]
    // PropSpecChildrenInvalid(String),
    // #[error(transparent)]
    // PropTree(#[from] PropTreeError),
    // #[error("prop tree structure is invalid: {0}")]
    // PropTreeInvalid(String),
    #[error("schema error: {0}")]
    Schema(#[from] SchemaError),
    #[error("schema variant error: {0}")]
    SchemaVariant(#[from] SchemaVariantError),
    // #[error(transparent)]
    // SchemaVariantDefinition(#[from] SchemaVariantDefinitionError),
    // #[error("schema variant not found: {0}")]
    // SchemaVariantNotFound(SchemaVariantId),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    // #[error(transparent)]
    // Socket(#[from] SocketError),
    // #[error(transparent)]
    // StandardModel(#[from] StandardModelError),
    // #[error("standard model relationship {0} missing belongs_to for {1} with id {2}")]
    // StandardModelMissingBelongsTo(&'static str, &'static str, String),
    // #[error("standard model relationship {0} found multiple belongs_to for {1} with id {2}")]
    // StandardModelMultipleBelongsTo(&'static str, &'static str, String),
    // #[error(transparent)]
    // UlidDecode(#[from] ulid::DecodeError),
    // #[error(transparent)]
    // UrlParse(#[from] ParseError),
    // #[error(transparent)]
    // Workspace(#[from] WorkspaceError),
    // #[error("Cannot find default change set \"{0}\" in workspace backup")]
    // WorkspaceBackupNoDefaultChangeSet(String),
    // #[error("Workspace backup missing workspace name")]
    // WorkspaceNameNotInBackup,
    // #[error("Workspace not found: {0}")]
    // WorkspaceNotFound(WorkspacePk),
    // #[error("Workspace backup missing workspace pk")]
    // WorkspacePkNotInBackup,
    #[error(transparent)]
    WorkspaceSnaphot(#[from] WorkspaceSnapshotError),
}

impl PkgError {
    // fn prop_tree_invalid(message: impl Into<String>) -> Self {
    //     Self::PropTreeInvalid(message.into())
    // }

    // fn prop_spec_children_invalid(message: impl Into<String>) -> Self {
    //     Self::PropSpecChildrenInvalid(message.into())
    // }
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
/// thing is asked for in a specific change set, and not found, the NONE change set will be
/// checked.
pub struct ChangeSetThingMap<Key, Thing>(HashMap<ChangeSetPk, HashMap<Key, Thing>>);

impl<Key, Thing> ChangeSetThingMap<Key, Thing>
where
    Key: Eq + PartialEq + std::hash::Hash,
{
    pub fn new() -> Self {
        let head_thing_map = HashMap::new();

        let mut change_set_map: HashMap<ChangeSetPk, HashMap<Key, Thing>> = HashMap::new();
        change_set_map.insert(ChangeSetPk::NONE, head_thing_map);

        Self(change_set_map)
    }

    pub fn get(&self, change_set_pk: Option<ChangeSetPk>, key: &Key) -> Option<&Thing> {
        match self.0.get(&change_set_pk.unwrap_or(ChangeSetPk::NONE)) {
            Some(change_set_map) => change_set_map.get(key).or_else(|| {
                self.0
                    .get(&ChangeSetPk::NONE)
                    .and_then(|things| things.get(key))
            }),
            None => self
                .0
                .get(&ChangeSetPk::NONE)
                .and_then(|things| things.get(key)),
        }
    }

    pub fn insert(
        &mut self,
        change_set_pk: Option<ChangeSetPk>,
        key: Key,
        thing: Thing,
    ) -> Option<Thing> {
        self.0
            .entry(change_set_pk.unwrap_or(ChangeSetPk::NONE))
            .or_default()
            .insert(key, thing)
    }
}

impl<Key, Thing> Default for ChangeSetThingMap<Key, Thing>
where
    Key: Eq + PartialEq + std::hash::Hash,
{
    fn default() -> Self {
        Self::new()
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
