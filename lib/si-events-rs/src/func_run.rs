use chrono::{
    DateTime,
    Utc,
};
use derive_builder::Builder;
use serde::{
    Deserialize,
    Serialize,
};
use si_id::{
    ActionId,
    ActionPrototypeId,
};
pub use si_id::{
    AttributePrototypeArgumentId,
    AttributePrototypeId,
    AttributeValueId,
    ComponentId,
    FuncRunId,
    ManagementPrototypeId,
    ViewId,
};
use strum::{
    AsRefStr,
    Display,
    EnumIter,
    EnumString,
};

use crate::{
    ActionKind,
    ActionResultState,
    Actor,
    ChangeSetId,
    ContentHash,
    FuncId,
    Tenancy,
    WorkspacePk,
};

#[derive(AsRefStr, Deserialize, Display, Serialize, Debug, Eq, PartialEq, Clone, Copy)]
pub enum FuncRunState {
    Created,
    Dispatched,
    // NOTE(nick,fletcher): renamed from "cancelled" to "killed", but needed backwards compatibility.
    #[serde(alias = "cancelled")]
    Killed,
    Running,
    PostProcessing,
    Failure,
    Success,
}

/// Describes the kind of [`Func`](crate::Func).
#[derive(
    AsRefStr, Deserialize, Display, Serialize, Debug, Eq, PartialEq, Clone, Copy, Hash, EnumIter,
)]
pub enum FuncKind {
    Action,
    Attribute,
    Authentication,
    CodeGeneration,
    Intrinsic,
    Qualification,
    SchemaVariantDefinition,
    Unknown,
    Management,
    Debug,
}

/// Describes the kind of [`FuncArgument`](crate::FuncArgument).
#[derive(AsRefStr, Deserialize, Display, Serialize, Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum FuncArgumentKind {
    Any,
    Array,
    Boolean,
    Integer,
    Json,
    Map,
    Object,
    String,
    Float,
}

// NOTE(nick,zack): do not add "remain::sorted" for postcard de/ser. We need the order to be
// retained.
#[derive(
    Deserialize,
    Serialize,
    Debug,
    Display,
    AsRefStr,
    PartialEq,
    Eq,
    EnumIter,
    EnumString,
    Clone,
    Copy,
)]
pub enum FuncBackendKind {
    Array,
    Boolean,
    /// Comparison between two JSON values
    Diff,
    /// Mathematical identity of the [`Func`](crate::Func)'s arguments.
    Identity,
    Integer,
    JsAction,
    JsAttribute,
    JsAuthentication,
    Json,
    // NOTE(nick): this has been deprecated. Not adding serde deprecated tag in case it affects the type.
    JsReconciliation,
    JsSchemaVariantDefinition,
    JsValidation,
    Map,
    Object,
    String,
    Unset,
    Validation,
    Management,
    ResourcePayloadToValue,
    NormalizeToArray,
    Float,
    Debug,
}

// NOTE(nick,zack): do not add "remain::sorted" for postcard de/ser. We need the order to be
// retained.
#[derive(
    Deserialize,
    Serialize,
    Debug,
    Display,
    AsRefStr,
    PartialEq,
    Eq,
    EnumIter,
    EnumString,
    Clone,
    Copy,
)]
pub enum FuncBackendResponseType {
    Action,
    Array,
    Boolean,
    CodeGeneration,
    /// Mathematical identity of the [`Func`](crate::Func)'s arguments.
    Identity,
    Integer,
    Json,
    Map,
    Object,
    Qualification,
    // NOTE(nick): this has been deprecated. Not adding serde deprecated tag in case it affects the type.
    Reconciliation,
    SchemaVariantDefinition,
    String,
    Unset,
    Validation,
    Void,
    Management,
    Float,
    Debug,
}

#[derive(Debug, Clone, Builder, Serialize, Deserialize)]
pub struct FuncRun {
    #[builder(default = "FuncRunId::new()")]
    id: FuncRunId,
    #[builder(default = "FuncRunState::Created")]
    state: FuncRunState,
    actor: Actor,
    tenancy: Tenancy,
    component_id: Option<ComponentId>,
    attribute_value_id: Option<AttributeValueId>,
    #[builder(default)]
    component_name: Option<String>,
    #[builder(default)]
    schema_name: Option<String>,
    #[builder(default)]
    action_or_func_id: Option<si_id::ulid_upstream::Ulid>,
    #[builder(default)]
    prototype_id: Option<si_id::ulid_upstream::Ulid>,
    #[builder(default)]
    action_kind: Option<ActionKind>,
    #[builder(default)]
    action_display_name: Option<String>,
    #[builder(default)]
    action_originating_change_set_id: Option<ChangeSetId>,
    #[builder(default)]
    action_originating_change_set_name: Option<String>,
    #[builder(default)]
    action_result_state: Option<ActionResultState>,
    backend_kind: FuncBackendKind,
    backend_response_type: FuncBackendResponseType,
    function_name: String,
    #[builder(default)]
    function_display_name: Option<String>,
    function_kind: FuncKind,
    #[builder(default)]
    function_description: Option<String>,
    #[builder(default)]
    function_link: Option<String>,
    function_args_cas_address: ContentHash,
    function_code_cas_address: ContentHash,
    #[builder(default)]
    result_value_cas_address: Option<ContentHash>,
    #[builder(default)]
    result_unprocessed_value_cas_address: Option<ContentHash>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

impl FuncRun {
    /// NOTE: We are also using this to record the success or failure state of a
    /// management function. Extending the ActionResult state should be fine as
    /// long as the three existing states remain (and we can't remove them or
    /// we'll break postcard serialization).
    pub fn set_action_result_state(&mut self, value: Option<ActionResultState>) {
        self.action_result_state = value;
        self.updated_at = Utc::now();
    }

    pub fn set_success(
        &mut self,
        unprocessed_value: Option<ContentHash>,
        value: Option<ContentHash>,
    ) {
        self.result_unprocessed_value_cas_address = unprocessed_value;
        self.result_value_cas_address = value;
        self.state = FuncRunState::Success;
        self.updated_at = Utc::now();
    }

    pub fn set_state(&mut self, state: FuncRunState) {
        self.updated_at = Utc::now();
        self.state = state;
    }

    pub fn id(&self) -> FuncRunId {
        self.id
    }

    pub fn state(&self) -> FuncRunState {
        self.state
    }

    pub fn actor(&self) -> &Actor {
        &self.actor
    }

    pub fn tenancy(&self) -> &Tenancy {
        &self.tenancy
    }

    pub fn workspace_pk(&self) -> WorkspacePk {
        self.tenancy.workspace_pk
    }

    pub fn change_set_id(&self) -> ChangeSetId {
        self.tenancy.change_set_id
    }

    pub fn component_id(&self) -> Option<ComponentId> {
        self.component_id
    }

    pub fn component_name(&self) -> Option<&str> {
        self.component_name.as_deref()
    }

    pub fn attribute_value_id(&self) -> Option<AttributeValueId> {
        self.attribute_value_id
    }

    pub fn action_id(&self) -> Option<ActionId> {
        self.action_or_func_id.map(Into::into)
    }

    pub fn func_id(&self) -> Option<FuncId> {
        self.action_or_func_id.map(Into::into)
    }

    pub fn action_display_name(&self) -> Option<&str> {
        self.action_display_name.as_deref()
    }

    pub fn schema_name(&self) -> Option<&str> {
        self.schema_name.as_deref()
    }

    pub fn action_kind(&self) -> Option<ActionKind> {
        self.action_kind
    }

    pub fn action_result_state(&self) -> Option<ActionResultState> {
        self.action_result_state
    }

    /// The action prototype id of this action run, *if* this is an action run.
    /// If this is not an action run, this might actually be another prototype
    /// id
    pub fn action_prototype_id(&self) -> Option<ActionPrototypeId> {
        self.prototype_id.map(Into::into)
    }

    /// The management prototype id of this action run, *if* this is a mangement
    /// run.  If this is not management run, this might actually be another
    /// prototype id
    pub fn management_prototype_id(&self) -> Option<ManagementPrototypeId> {
        self.prototype_id.map(Into::into)
    }

    pub fn action_originating_change_set_id(&self) -> Option<ChangeSetId> {
        self.action_originating_change_set_id
    }

    pub fn action_originating_change_set_name(&self) -> Option<&str> {
        self.action_originating_change_set_name.as_deref()
    }

    pub fn backend_kind(&self) -> FuncBackendKind {
        self.backend_kind
    }

    pub fn backend_response_type(&self) -> FuncBackendResponseType {
        self.backend_response_type
    }

    pub fn function_name(&self) -> &str {
        &self.function_name
    }

    pub fn function_kind(&self) -> FuncKind {
        self.function_kind
    }

    pub fn function_args_cas_address(&self) -> ContentHash {
        self.function_args_cas_address
    }

    pub fn function_code_cas_address(&self) -> ContentHash {
        self.function_code_cas_address
    }

    pub fn result_value_cas_address(&self) -> Option<ContentHash> {
        self.result_value_cas_address
    }

    pub fn result_unprocessed_value_cas_address(&self) -> Option<ContentHash> {
        self.result_unprocessed_value_cas_address
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }

    pub fn updated_at(&self) -> DateTime<Utc> {
        self.updated_at
    }

    pub fn function_display_name(&self) -> Option<&str> {
        self.function_display_name.as_deref()
    }

    pub fn function_description(&self) -> Option<&str> {
        self.function_description.as_deref()
    }

    pub fn function_link(&self) -> Option<&str> {
        self.function_link.as_deref()
    }
}

#[derive(Debug)]
pub struct FuncRunValue {
    func_run_id: FuncRunId,
    unprocessed_value: Option<serde_json::Value>,
    value: Option<serde_json::Value>,
}

impl FuncRunValue {
    pub fn new(
        func_run_id: FuncRunId,
        unprocessed_value: Option<serde_json::Value>,
        value: Option<serde_json::Value>,
    ) -> Self {
        FuncRunValue {
            func_run_id,
            unprocessed_value,
            value,
        }
    }

    pub fn func_run_id(&self) -> FuncRunId {
        self.func_run_id
    }

    pub fn unprocessed_value(&self) -> Option<&serde_json::Value> {
        self.unprocessed_value.as_ref()
    }

    pub fn value(&self) -> Option<&serde_json::Value> {
        self.value.as_ref()
    }

    pub fn set_processed_value(&mut self, value: Option<serde_json::Value>) {
        self.value = value;
    }

    pub fn take_unprocessed_value(&mut self) -> Option<serde_json::Value> {
        self.unprocessed_value.take()
    }

    pub fn take_value(&mut self) -> Option<serde_json::Value> {
        self.value.take()
    }
}
