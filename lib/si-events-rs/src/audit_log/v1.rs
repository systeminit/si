use serde::{Deserialize, Serialize};
use strum::{Display, EnumDiscriminants};

use crate::{
    ActionKind, ActionPrototypeId, Actor, AttributeValueId, ChangeSetId, ChangeSetStatus,
    ComponentId, FuncId, InputSocketId, OutputSocketId, PropId, SchemaId, SchemaVariantId,
    SecretId, WorkspacePk,
};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct AuditLogV1 {
    pub actor: Actor,
    pub kind: AuditLogKindV1,
    pub entity_name: String,
    pub timestamp: String,
    pub change_set_id: Option<ChangeSetId>,
}

#[remain::sorted]
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Display, EnumDiscriminants)]
pub enum AuditLogKindV1 {
    AbandonChangeSet {
        from_status: ChangeSetStatus,
    },
    AddAction {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
    },
    ApplyChangeSet,
    ApproveChangeSetApply {
        from_status: ChangeSetStatus,
    },
    CancelAction {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
    },
    CreateChangeSet,
    CreateComponent {
        name: String,
        component_id: ComponentId,
        schema_variant_id: SchemaVariantId,
        schema_variant_name: String,
    },
    CreateSecret {
        name: String,
        secret_id: SecretId,
    },
    DeleteComponent {
        name: String,
        component_id: ComponentId,
        schema_variant_id: SchemaVariantId,
        schema_variant_name: String,
    },
    DeleteSecret {
        name: String,
        secret_id: SecretId,
    },
    ExportWorkspace {
        id: WorkspacePk,
        name: String,
        version: String,
    },
    InstallWorkspace {
        id: WorkspacePk,
        name: String,
        version: String,
    },
    Login,
    PutActionOnHold {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
    },
    RejectChangeSetApply {
        from_status: ChangeSetStatus,
    },

    ReopenChangeSet {
        from_status: ChangeSetStatus,
    },
    RequestChangeSetApproval {
        from_status: ChangeSetStatus,
    },
    RetryAction {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
    },
    RunAction {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
        run_status: bool,
    },
    UpdateDependentInputSocket {
        input_socket_id: InputSocketId,
        input_socket_name: String,
        attribute_value_id: AttributeValueId,
        input_attribute_value_ids: Vec<AttributeValueId>,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
        component_id: ComponentId,
        component_name: String,
        schema_variant_id: SchemaVariantId,
        schema_variant_display_name: String,
        before_value: Option<serde_json::Value>,
        after_value: Option<serde_json::Value>,
    },
    UpdateDependentOutputSocket {
        output_socket_id: OutputSocketId,
        output_socket_name: String,
        attribute_value_id: AttributeValueId,
        input_attribute_value_ids: Vec<AttributeValueId>,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
        component_id: ComponentId,
        component_name: String,
        schema_variant_id: SchemaVariantId,
        schema_variant_display_name: String,
        before_value: Option<serde_json::Value>,
        after_value: Option<serde_json::Value>,
    },
    UpdateDependentProperty {
        prop_id: PropId,
        prop_name: String,
        attribute_value_id: AttributeValueId,
        input_attribute_value_ids: Vec<AttributeValueId>,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
        component_id: ComponentId,
        component_name: String,
        schema_variant_id: SchemaVariantId,
        schema_variant_display_name: String,
        before_value: Option<serde_json::Value>,
        after_value: Option<serde_json::Value>,
    },
    UpdatePropertyEditorValue {
        component_id: ComponentId,
        component_name: String,
        schema_variant_id: SchemaVariantId,
        schema_variant_display_name: String,
        prop_id: PropId,
        prop_name: String,
        attribute_value_id: AttributeValueId,
        before_value: Option<serde_json::Value>,
        after_value: Option<serde_json::Value>,
    },
    UpdatePropertyEditorValueForSecret {
        component_id: ComponentId,
        component_name: String,
        schema_variant_id: SchemaVariantId,
        schema_variant_display_name: String,
        prop_id: PropId,
        prop_name: String,
        attribute_value_id: AttributeValueId,
        before_secret_name: Option<String>,
        before_secret_id: Option<SecretId>,
        after_secret_name: Option<String>,
        after_secret_id: Option<SecretId>,
    },
    UpdateSecret {
        name: String,
        secret_id: SecretId,
    },
    UpgradeComponent {
        name: String,
        component_id: ComponentId,
        schema_id: SchemaId,
        new_schema_variant_id: SchemaVariantId,
        new_schema_variant_name: String,
        old_schema_variant_id: SchemaVariantId,
        old_schema_variant_name: String,
    },
    WithdrawRequestForChangeSetApply {
        from_status: ChangeSetStatus,
    },
}
