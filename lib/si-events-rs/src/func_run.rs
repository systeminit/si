use chrono::{DateTime, Utc};
use derive_builder::Builder;
use serde::{Deserialize, Serialize};
use strum::{AsRefStr, Display, EnumIter, EnumString};

use crate::{id, Actor, ChangeSetId, ContentHash, Tenancy};

id!(FuncRunId);
id!(ComponentId);
id!(AttributeValueId);
id!(ActionId);
id!(ActionPrototypeId);

#[derive(AsRefStr, Deserialize, Display, Serialize, Debug, Eq, PartialEq, Clone, Copy)]
pub enum FuncRunState {
    Created,
    Dispatched,
    Running,
    PostProcessing,
    Failure,
    Success,
}

/// Describes the kind of [`Func`](crate::Func).
#[remain::sorted]
#[derive(AsRefStr, Deserialize, Display, Serialize, Debug, Eq, PartialEq, Clone, Copy, Hash)]
pub enum FuncKind {
    Action,
    Attribute,
    Authentication,
    CodeGeneration,
    Intrinsic,
    Qualification,
    SchemaVariantDefinition,
    Unknown,
}

#[remain::sorted]
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
    JsReconciliation,
    JsSchemaVariantDefinition,
    JsValidation,
    Map,
    Object,
    String,
    Unset,
    Validation,
}

#[remain::sorted]
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
    Reconciliation,
    SchemaVariantDefinition,
    String,
    Unset,
    Validation,
    Void,
}

#[remain::sorted]
#[derive(Debug, Copy, Clone, Deserialize, Serialize, PartialEq, Eq, Display)]
pub enum ActionKind {
    /// Create the "outside world" version of the modeled object.
    Create,
    /// Destroy the "outside world" version of the modeled object referenced in the resource.
    Destroy,
    /// This [`Action`][crate::Action] will only ever be manually queued.
    Manual,
    /// Refresh the resource to reflect the current state of the modeled object in the "outside
    /// world".
    Refresh,
    /// Update the version of the modeled object in the "outside world" to match the state of the
    /// model.
    Update,
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
    action_id: Option<ActionId>,
    #[builder(default)]
    action_prototype_id: Option<ActionPrototypeId>,
    #[builder(default)]
    action_kind: Option<ActionKind>,
    #[builder(default)]
    action_display_name: Option<String>,
    #[builder(default)]
    action_originating_change_set_id: Option<ChangeSetId>,
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
    pub fn builder() -> FuncRunBuilder {
        FuncRunBuilder::default()
    }

    pub fn set_result_value_cas_address(&mut self, value: Option<ContentHash>) {
        self.result_value_cas_address = value;
        self.updated_at = Utc::now();
    }

    pub fn set_result_unprocessed_value_cas_address(
        &mut self,
        unprocessed_value: Option<ContentHash>,
    ) {
        self.result_unprocessed_value_cas_address = unprocessed_value;
        self.updated_at = Utc::now();
    }

    pub fn set_state_to_dispatched(&mut self) {
        self.updated_at = Utc::now();
        self.state = FuncRunState::Dispatched;
    }

    pub fn set_state_to_running(&mut self) {
        self.updated_at = Utc::now();
        self.state = FuncRunState::Running;
    }

    pub fn set_state_to_post_processing(&mut self) {
        self.updated_at = Utc::now();
        self.state = FuncRunState::PostProcessing;
    }

    pub fn set_state_to_success(&mut self) {
        self.updated_at = Utc::now();
        self.state = FuncRunState::Success;
    }

    pub fn set_state_to_failure(&mut self) {
        self.updated_at = Utc::now();
        self.state = FuncRunState::Failure;
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

    pub fn component_id(&self) -> Option<ComponentId> {
        self.component_id
    }

    pub fn attribute_value_id(&self) -> Option<AttributeValueId> {
        self.attribute_value_id
    }

    pub fn action_id(&self) -> Option<ActionId> {
        self.action_id
    }

    pub fn action_prototype_id(&self) -> Option<ActionPrototypeId> {
        self.action_prototype_id
    }

    pub fn action_originating_change_set_id(&self) -> Option<ChangeSetId> {
        self.action_originating_change_set_id
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
