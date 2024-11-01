use serde::Serialize;
use si_events::{
    audit_log::AuditLogKind, AttributeValueId, ChangeSetId, ComponentId, PropId, SchemaVariantId,
    SecretId, UserPk,
};

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AuditLog {
    pub user_id: Option<UserPk>,
    pub user_email: Option<String>,
    pub user_name: Option<String>,
    // NOTE(nick): enum discriminants are not deserializable, so this is a string.
    pub kind: String,
    pub timestamp: String,
    pub change_set_id: Option<ChangeSetId>,
    /// Serialized version of [`AuditLogDeserializedMetadata`].
    pub metadata: serde_json::Value,
}

/// This is an identical copy of latest [`AuditLogKind`], but uses "serde untagged" wrapper.
///
/// Reference: https://serde.rs/enum-representations.html#untagged
///
/// _Notes:_
///   1) this does not use [`remain::sorted`] in order to match the aforementioned type
///   2) multiple uses of renaming to camel case are related to this: https://github.com/serde-rs/serde/issues/1560
#[derive(Debug, Serialize)]
#[serde(untagged, rename_all = "camelCase")]
pub enum AuditLogDeserializedMetadata {
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

impl From<AuditLogKind> for AuditLogDeserializedMetadata {
    fn from(value: AuditLogKind) -> Self {
        match value {
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
