use serde::Serialize;
use si_events::{
    audit_log::AuditLogKind, ActionKind, ActionPrototypeId, AttributeValueId, ChangeSetId,
    ComponentId, FuncId, InputSocketId, OutputSocketId, PropId, SchemaVariantId, SecretId, UserPk,
};
use strum::EnumDiscriminants;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuditLog {
    /// The title of the [`AuditLog`]. It will likely be combined with the `entity_type` to make a full display name.
    pub title: String,
    /// The identifier of the user. If this is empty, it is the system user.
    pub user_id: Option<UserPk>,
    /// The email of the user.
    pub user_email: Option<String>,
    /// The name of the user.
    pub user_name: Option<String>,
    /// The [kind](AuditLogKing) of the [`AuditLog`] (converted into a string because enum discriminants are not
    /// serializable).
    pub kind: String,
    /// The entity type.
    pub entity_type: String,
    /// The entity name.
    pub entity_name: String,
    /// The timestamp in ISO RFC 3339 format (converted into a string).
    pub timestamp: String,
    /// The identifier of the change set, which will only be empty for actions taken outside of the workspace.
    pub change_set_id: Option<ChangeSetId>,
    /// The name of the change set.
    pub change_set_name: Option<String>,
    /// Serialized version of [`AuditLogDeserializedMetadata`], which is an untagged version of the specific
    /// [`AuditLogKind`].
    pub metadata: serde_json::Value,
}

/// This is an identical copy of latest [`AuditLogKind`], but uses "serde untagged" wrapper.
///
/// Reference: https://serde.rs/enum-representations.html#untagged
///
/// _Note:_ there are multiple uses of renaming to camel case are related to this: https://github.com/serde-rs/serde/issues/1560
#[remain::sorted]
#[derive(Debug, Serialize, EnumDiscriminants)]
#[serde(untagged, rename_all = "camelCase")]
pub enum AuditLogDeserializedMetadata {
    #[serde(rename_all = "camelCase")]
    AbandonChangeSet,
    #[serde(rename_all = "camelCase")]
    AddAction {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
    },
    #[serde(rename_all = "camelCase")]
    ApplyChangeSet,
    #[serde(rename_all = "camelCase")]
    CancelAction {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
    },
    #[serde(rename_all = "camelCase")]
    CreateChangeSet,
    #[serde(rename_all = "camelCase")]
    CreateComponent {
        name: String,
        component_id: ComponentId,
        schema_variant_id: SchemaVariantId,
        schema_variant_name: String,
    },
    #[serde(rename_all = "camelCase")]
    DeleteComponent {
        name: String,
        component_id: ComponentId,
        schema_variant_id: SchemaVariantId,
        schema_variant_name: String,
    },
    #[serde(rename_all = "camelCase")]
    PutActionOnHold {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
    },
    #[serde(rename_all = "camelCase")]
    RunAction {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
        run_status: bool,
    },
    #[serde(rename_all = "camelCase")]
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
    #[serde(rename_all = "camelCase")]
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
    #[serde(rename_all = "camelCase")]
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
    #[serde(rename_all = "camelCase")]
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
    #[serde(rename_all = "camelCase")]
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
}

impl AuditLogDeserializedMetadata {
    pub fn title_and_entity_type(&self) -> (&'static str, &'static str) {
        type Kind = AuditLogDeserializedMetadataDiscriminants;

        match self.into() {
            Kind::AbandonChangeSet => ("Abandoned", "Change Set"),
            Kind::AddAction => ("Enqueued", "Action"),
            Kind::ApplyChangeSet => ("Applied", "Change Set"),
            Kind::CancelAction => ("Removed", "Action"),
            Kind::CreateChangeSet => ("Created", "Change Set"),
            Kind::CreateComponent => ("Created", "Component"),
            Kind::DeleteComponent => ("Deleted", "Component"),
            Kind::PutActionOnHold => ("Paused", "Action"),
            Kind::RunAction => ("Ran", "Action"),
            Kind::UpdateDependentInputSocket => ("Set Dependent", "Input Socket"),
            Kind::UpdateDependentOutputSocket => ("Set Dependent", "Output Socket"),
            Kind::UpdateDependentProperty => ("Set Dependent", "Property"),
            Kind::UpdatePropertyEditorValue => ("Updated Component", "Property"),
            Kind::UpdatePropertyEditorValueForSecret => {
                ("Updated Component", "Property for Secret")
            }
        }
    }
}

impl From<AuditLogKind> for AuditLogDeserializedMetadata {
    fn from(value: AuditLogKind) -> Self {
        // Please keep this in alphabetical order!
        match value {
            AuditLogKind::AbandonChangeSet => Self::AbandonChangeSet,
            AuditLogKind::AddAction {
                prototype_id,
                action_kind,
                func_id,
                func_display_name,
                func_name,
            } => Self::AddAction {
                prototype_id,
                action_kind,
                func_id,
                func_display_name,
                func_name,
            },
            AuditLogKind::ApplyChangeSet => Self::ApplyChangeSet,
            AuditLogKind::CancelAction {
                prototype_id,
                action_kind,
                func_id,
                func_display_name,
                func_name,
            } => Self::CancelAction {
                prototype_id,
                action_kind,
                func_id,
                func_display_name,
                func_name,
            },
            AuditLogKind::CreateChangeSet => Self::CreateChangeSet,
            AuditLogKind::CreateComponent {
                name,
                component_id,
                schema_variant_id,
                schema_variant_name,
            } => Self::CreateComponent {
                name,
                component_id,
                schema_variant_id,
                schema_variant_name,
            },
            AuditLogKind::DeleteComponent {
                name,
                component_id,
                schema_variant_id,
                schema_variant_name,
            } => Self::DeleteComponent {
                name,
                component_id,
                schema_variant_id,
                schema_variant_name,
            },
            AuditLogKind::PutActionOnHold {
                prototype_id,
                action_kind,
                func_id,
                func_display_name,
                func_name,
            } => Self::PutActionOnHold {
                prototype_id,
                action_kind,
                func_id,
                func_display_name,
                func_name,
            },
            AuditLogKind::RunAction {
                prototype_id,
                action_kind,
                func_id,
                func_display_name,
                func_name,
                run_status,
            } => Self::RunAction {
                prototype_id,
                action_kind,
                func_id,
                func_display_name,
                func_name,
                run_status,
            },
            AuditLogKind::UpdateDependentInputSocket {
                input_socket_id,
                input_socket_name,
                attribute_value_id,
                input_attribute_value_ids,
                func_id,
                func_display_name,
                func_name,
                component_id,
                component_name,
                schema_variant_id,
                schema_variant_display_name,
                before_value,
                after_value,
            } => Self::UpdateDependentInputSocket {
                input_socket_id,
                input_socket_name,
                attribute_value_id,
                input_attribute_value_ids,
                func_id,
                func_display_name,
                func_name,
                component_id,
                component_name,
                schema_variant_id,
                schema_variant_display_name,
                before_value,
                after_value,
            },
            AuditLogKind::UpdateDependentOutputSocket {
                output_socket_id,
                output_socket_name,
                attribute_value_id,
                input_attribute_value_ids,
                func_id,
                func_display_name,
                func_name,
                component_id,
                component_name,
                schema_variant_id,
                schema_variant_display_name,
                before_value,
                after_value,
            } => Self::UpdateDependentOutputSocket {
                output_socket_id,
                output_socket_name,
                attribute_value_id,
                input_attribute_value_ids,
                func_id,
                func_display_name,
                func_name,
                component_id,
                component_name,
                schema_variant_id,
                schema_variant_display_name,
                before_value,
                after_value,
            },
            AuditLogKind::UpdateDependentProperty {
                prop_id,
                prop_name,
                attribute_value_id,
                input_attribute_value_ids,
                func_id,
                func_display_name,
                func_name,
                component_id,
                component_name,
                schema_variant_id,
                schema_variant_display_name,
                before_value,
                after_value,
            } => Self::UpdateDependentProperty {
                prop_id,
                prop_name,
                attribute_value_id,
                input_attribute_value_ids,
                func_id,
                func_display_name,
                func_name,
                component_id,
                component_name,
                schema_variant_id,
                schema_variant_display_name,
                before_value,
                after_value,
            },
            AuditLogKind::UpdatePropertyEditorValue {
                component_id,
                component_name,
                schema_variant_id,
                schema_variant_display_name,
                prop_id,
                prop_name,
                attribute_value_id,
                before_value,
                after_value,
            } => Self::UpdatePropertyEditorValue {
                component_id,
                component_name,
                schema_variant_id,
                schema_variant_display_name,
                prop_id,
                prop_name,
                attribute_value_id,
                before_value,
                after_value,
            },
            AuditLogKind::UpdatePropertyEditorValueForSecret {
                component_id,
                component_name,
                schema_variant_id,
                schema_variant_display_name,
                prop_id,
                prop_name,
                attribute_value_id,
                before_secret_name,
                before_secret_id,
                after_secret_name,
                after_secret_id,
            } => Self::UpdatePropertyEditorValueForSecret {
                component_id,
                component_name,
                schema_variant_id,
                schema_variant_display_name,
                prop_id,
                prop_name,
                attribute_value_id,
                before_secret_name,
                before_secret_id,
                after_secret_name,
                after_secret_id,
            },
        }
    }
}
