use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::ParseError;

pub use export::{get_component_type, PkgExporter};
pub use import::{
    attach_resource_payload_to_value, import_pkg, import_pkg_from_pkg, ImportAttributeSkip,
    ImportEdgeSkip, ImportOptions, ImportSkips,
};
use si_pkg::{FuncSpecBackendKind, FuncSpecBackendResponseType, SiPkgError, SpecError};

use crate::authentication_prototype::AuthenticationPrototypeError;
use crate::{
    component::view::debug::ComponentDebugViewError,
    func::{
        argument::{FuncArgumentError, FuncArgumentId},
        binding::FuncBindingError,
    },
    installed_pkg::InstalledPkgError,
    prop_tree::PropTreeError,
    schema::variant::definition::{SchemaVariantDefinitionError, SchemaVariantDefinitionId},
    socket::{SocketEdgeKind, SocketError},
    ActionPrototypeError, AttributeContextBuilderError, AttributePrototypeArgumentError,
    AttributePrototypeArgumentId, AttributePrototypeError, AttributePrototypeId,
    AttributeReadContext, AttributeValueError, ChangeSetError, ChangeSetPk, ComponentError,
    ComponentId, DalContext, EdgeError, ExternalProviderError, ExternalProviderId, FuncBackendKind,
    FuncBackendResponseType, FuncBindingReturnValueError, FuncError, FuncId, InternalProviderError,
    InternalProviderId, NodeError, PropError, PropId, PropKind, SchemaError, SchemaId,
    SchemaVariantError, SchemaVariantId, StandardModelError, UserPk, WorkspaceError, WorkspacePk,
    WsEvent, WsEventResult, WsPayload,
};

mod export;
mod import;

#[remain::sorted]
#[derive(Debug, Error)]
pub enum PkgError {
    #[error("Action creation error: {0}")]
    Action(#[from] ActionPrototypeError),
    #[error(transparent)]
    AttributeContextBuilder(#[from] AttributeContextBuilderError),
    #[error("attribute function for context {0:?} has key {1} but is not setting a prop value")]
    AttributeFuncForKeyMissingProp(AttributeReadContext, String),
    #[error("attribute function for prop {0} has a key {1} but prop kind is {2} not a map)")]
    AttributeFuncForKeySetOnWrongKind(PropId, String, PropKind),
    #[error(transparent)]
    AttributePrototype(#[from] AttributePrototypeError),
    #[error(transparent)]
    AttributePrototypeArgument(#[from] AttributePrototypeArgumentError),
    #[error("Missing ExternalProvider {1} for AttributePrototypeArgument {1}")]
    AttributePrototypeArgumentMissingExternalProvider(
        AttributePrototypeArgumentId,
        ExternalProviderId,
    ),
    #[error("AttributePrototypeArgument {0} missing FuncArgument {1}")]
    AttributePrototypeArgumentMissingFuncArgument(AttributePrototypeArgumentId, FuncArgumentId),
    #[error("Missing InternalProvider {1} for AttributePrototypeArgument {1}")]
    AttributePrototypeArgumentMissingInternalProvider(
        AttributePrototypeArgumentId,
        InternalProviderId,
    ),
    #[error(transparent)]
    AttributeValue(#[from] AttributeValueError),
    #[error("parent prop could not be found with path: {0}")]
    AttributeValueParentPropNotFound(String),
    #[error("parent value could not be found for prop path: {0} and key {1:?}, index {2:?}")]
    AttributeValueParentValueNotFound(String, Option<String>, Option<i64>),
    #[error("attribute value is a proxy but there is no value to proxy")]
    AttributeValueSetToProxyButNoProxyFound,
    #[error("encountered an attribute value with a key or index but no parent")]
    AttributeValueWithKeyOrIndexButNoParent,
    #[error("Auth func creation error: {0}")]
    AuthFunc(#[from] AuthenticationPrototypeError),
    #[error(transparent)]
    ChangeSet(#[from] ChangeSetError),
    #[error("change set {0} not found")]
    ChangeSetNotFound(ChangeSetPk),
    #[error(transparent)]
    Component(#[from] ComponentError),
    #[error(transparent)]
    ComponentDebugView(#[from] ComponentDebugViewError),
    #[error("component import can only happen during a workspace import")]
    ComponentImportWithoutChangeSet,
    #[error("could not find schema {0} for package component {1}")]
    ComponentMissingBuiltinSchema(String, String),
    #[error("could not find schema {0} with variant {1} for package component {2}")]
    ComponentMissingBuiltinSchemaVariant(String, String, String),
    #[error("component has no node: {0}")]
    ComponentMissingNode(ComponentId),
    #[error("could not find schema variant {0} for package component {1}")]
    ComponentMissingSchemaVariant(String, String),
    #[error("could not update find schema {0} with variant {1} for package component {2}")]
    ComponentMissingUpdateSchemaVariant(String, String, String),
    #[error("component spec has no position")]
    ComponentSpecMissingPosition,
    #[error("map item prop {0} has both custom key prototypes and custom prop only prototype")]
    ConflictingMapKeyPrototypes(PropId),
    #[error("expected data on an SiPkg node, but none found: {0}")]
    DataNotFound(String),
    #[error(transparent)]
    Edge(#[from] EdgeError),
    #[error("edge refers to component not in export: {0}")]
    EdgeRefersToMissingComponent(ComponentId),
    #[error("Cannot find Socket for explicit InternalProvider {0}")]
    ExplicitInternalProviderMissingSocket(InternalProviderId),
    #[error(transparent)]
    ExternalProvider(#[from] ExternalProviderError),
    #[error("Cannot find Socket for ExternalProvider {0}")]
    ExternalProviderMissingSocket(ExternalProviderId),
    #[error(transparent)]
    Func(#[from] FuncError),
    #[error(transparent)]
    FuncArgument(#[from] FuncArgumentError),
    #[error(transparent)]
    FuncBinding(#[from] FuncBindingError),
    #[error(transparent)]
    FuncBindingReturnValue(#[from] FuncBindingReturnValueError),
    #[error(transparent)]
    FuncExecution(#[from] crate::func::execution::FuncExecutionError),
    #[error("Installed func id {0} does not exist")]
    InstalledFuncMissing(FuncId),
    #[error(transparent)]
    InstalledPkg(#[from] InstalledPkgError),
    #[error("Installed schema id {0} does not exist")]
    InstalledSchemaMissing(SchemaId),
    #[error("Installed schema variant definition {0} does not exist")]
    InstalledSchemaVariantDefinitionMissing(SchemaVariantDefinitionId),
    #[error("Installed schema variant {0} does not exist")]
    InstalledSchemaVariantMissing(SchemaVariantId),
    #[error(transparent)]
    InternalProvider(#[from] InternalProviderError),
    #[error("Missing Prop {1} for InternalProvider {1}")]
    InternalProviderMissingProp(InternalProviderId, PropId),
    #[error("Leaf Function {0} has invalid argument {1}")]
    InvalidLeafArgument(FuncId, String),
    #[error("json pointer {1} not found in {0:?}")]
    JsonPointerNotFound(serde_json::Value, String),
    #[error("json value is not an object: {0:?}")]
    JsonValueIsNotAnObject(serde_json::Value),
    #[error("Missing AttributePrototype {0} for explicit InternalProvider {1}")]
    MissingAttributePrototypeForInputSocket(AttributePrototypeId, InternalProviderId),
    #[error("Missing AttributePrototype {0} for ExternalProvider {1}")]
    MissingAttributePrototypeForOutputSocket(AttributePrototypeId, ExternalProviderId),
    #[error("Missing Func {1} for AttributePrototype {0}")]
    MissingAttributePrototypeFunc(AttributePrototypeId, FuncId),
    #[error("Missing value for context {0:?}")]
    MissingAttributeValueForContext(AttributeReadContext),
    #[error("Missing a func map for changeset {0}")]
    MissingChangeSetFuncMap(ChangeSetPk),
    #[error("Missing component {0} for edge from {1} to {2}")]
    MissingComponentForEdge(String, String, String),
    #[error("Func {0} missing from exported funcs")]
    MissingExportedFunc(FuncId),
    #[error("Cannot find FuncArgument {0} for Func {1}")]
    MissingFuncArgument(String, FuncId),
    #[error("Cannot find FuncArgument {0}")]
    MissingFuncArgumentById(FuncArgumentId),
    #[error("Package asked for a function with the unique id {0} but none could be found")]
    MissingFuncUniqueId(String),
    #[error("Cannot find InternalProvider for Prop {0}")]
    MissingInternalProviderForProp(PropId),
    #[error("Cannot find InternalProvider for Socket named {0}")]
    MissingInternalProviderForSocketName(String),
    #[error("Intrinsic function {0} not found")]
    MissingIntrinsicFunc(String),
    #[error("Intrinsic function (0) argument {1} not found")]
    MissingIntrinsicFuncArgument(String, String),
    #[error("Cannot find item prop for installed map prop {0}")]
    MissingItemPropForMapProp(PropId),
    #[error("Cannot find installed prop {0}")]
    MissingProp(PropId),
    #[error("Cannot find root prop for variant {0}")]
    MissingRootProp(SchemaVariantId),
    #[error("Cannot find schema_variant_definition {0}")]
    MissingSchemaVariantDefinition(SchemaVariantId),
    #[error("Cannot find socket with name {0} for edge kind {1}")]
    MissingSocketName(String, SocketEdgeKind),
    #[error("Unique id missing for node in workspace backup: {0}")]
    MissingUniqueIdForNode(String),
    #[error(transparent)]
    Node(#[from] NodeError),
    #[error("Package with that hash already installed: {0}")]
    PackageAlreadyInstalled(String),
    #[error(transparent)]
    Pkg(#[from] SiPkgError),
    #[error(transparent)]
    PkgSpec(#[from] SpecError),
    #[error(transparent)]
    Prop(#[from] PropError),
    #[error("prop spec structure is invalid: {0}")]
    PropSpecChildrenInvalid(String),
    #[error(transparent)]
    PropTree(#[from] PropTreeError),
    #[error("prop tree structure is invalid: {0}")]
    PropTreeInvalid(String),
    #[error(transparent)]
    Schema(#[from] SchemaError),
    #[error(transparent)]
    SchemaVariant(#[from] SchemaVariantError),
    #[error(transparent)]
    SchemaVariantDefinition(#[from] SchemaVariantDefinitionError),
    #[error("schema variant not found: {0}")]
    SchemaVariantNotFound(SchemaVariantId),
    #[error("json serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),
    #[error(transparent)]
    Socket(#[from] SocketError),
    #[error(transparent)]
    StandardModel(#[from] StandardModelError),
    #[error("standard model relationship {0} missing belongs_to for {1} with id {2}")]
    StandardModelMissingBelongsTo(&'static str, &'static str, String),
    #[error("standard model relationship {0} found multiple belongs_to for {1} with id {2}")]
    StandardModelMultipleBelongsTo(&'static str, &'static str, String),
    #[error(transparent)]
    UlidDecode(#[from] ulid::DecodeError),
    #[error("unable to export component: {0}")]
    UnableToExportComponent(ComponentId),
    #[error(transparent)]
    UrlParse(#[from] ParseError),
    #[error(transparent)]
    Workspace(#[from] WorkspaceError),
    #[error("Cannot find default change set \"{0}\" in workspace backup")]
    WorkspaceBackupNoDefaultChangeSet(String),
    #[error("Workspace backup missing workspace name")]
    WorkspaceNameNotInBackup,
    #[error("Workspace not found: {0}")]
    WorkspaceNotFound(WorkspacePk),
    #[error("Workspace backup missing workspace pk")]
    WorkspacePkNotInBackup,
}

impl PkgError {
    fn prop_tree_invalid(message: impl Into<String>) -> Self {
        Self::PropTreeInvalid(message.into())
    }

    fn prop_spec_children_invalid(message: impl Into<String>) -> Self {
        Self::PropSpecChildrenInvalid(message.into())
    }
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
#[derive(Debug)]
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

    pub fn get(&self, change_set_pk: ChangeSetPk, key: &Key) -> Option<&Thing> {
        match self.0.get(&change_set_pk) {
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

    pub fn insert(&mut self, change_set_pk: ChangeSetPk, key: Key, thing: Thing) -> Option<Thing> {
        self.0.entry(change_set_pk).or_default().insert(key, thing)
    }

    pub fn remove(&mut self, change_set_pk: ChangeSetPk, key: &Key) -> Option<Thing> {
        self.0.entry(change_set_pk).or_default().remove(key)
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

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceImportPayload {
    workspace_pk: Option<WorkspacePk>,
    user_pk: Option<UserPk>,
}

#[derive(Clone, Deserialize, Serialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceExportPayload {
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
        schema_variant_ids: Vec<SchemaVariantId>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::ModuleImported(ModuleImportedPayload { schema_variant_ids }),
        )
        .await
    }

    pub async fn workspace_imported(
        ctx: &DalContext,
        workspace_pk: Option<WorkspacePk>,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::WorkspaceImported(WorkspaceImportPayload {
                workspace_pk,
                user_pk,
            }),
        )
        .await
    }

    pub async fn workspace_exported(
        ctx: &DalContext,
        workspace_pk: Option<WorkspacePk>,
        user_pk: Option<UserPk>,
    ) -> WsEventResult<Self> {
        WsEvent::new(
            ctx,
            WsPayload::WorkspaceExported(WorkspaceExportPayload {
                workspace_pk,
                user_pk,
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
