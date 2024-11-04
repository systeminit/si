use serde::{Deserialize, Serialize};
use strum::Display;

use crate::{
    ActionKind, ActionPrototypeId, Actor, AttributeValueId, ChangeSetId, ComponentId, FuncId,
    PropId, SchemaVariantId, SecretId,
};

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct AuditLogV2 {
    pub actor: Actor,
    pub kind: AuditLogKindV2,
    pub timestamp: String,
    pub change_set_id: Option<ChangeSetId>,
}

// NOTE(nick): this intentionally does not use "remain::sorted".
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq, Display)]
pub enum AuditLogKindV2 {
    CreateComponent {
        name: String,
        component_id: ComponentId,
        schema_variant_id: SchemaVariantId,
        schema_variant_name: String,
    },
    DeleteComponent {
        name: String,
        component_id: ComponentId,
        schema_variant_id: SchemaVariantId,
        schema_variant_name: String,
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
    ApplyChangeset,
    AddAction {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
    },
    PutActionOnHold {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
    },
    CancelAction {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
    },
    CreateChangeset,
    AbandonChangeset,
    ActionRun {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
        run_status: bool,
    },
}
