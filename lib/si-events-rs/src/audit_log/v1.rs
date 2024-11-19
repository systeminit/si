use serde::{Deserialize, Serialize};
use strum::{Display, EnumDiscriminants};

use crate::{
    ActionKind, ActionPrototypeId, Actor, AttributeValueId, ChangeSetId, ChangeSetStatus,
    ComponentId, FuncId, InputSocketId, OutputSocketId, PropId, SchemaId, SchemaVariantId,
    SecretId, WorkspacePk,
};

type MetadataDiscrim = AuditLogMetadataV1Discriminants;
type Kind = AuditLogKindV1;
type Metadata = AuditLogMetadataV1;

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

/// This is an identical copy of latest [`AuditLogKind`], but uses "serde untagged" wrapper. This is used for inserting
/// and reading from the "metadata" column in the table as well as for additional columns.
///
/// Reference: https://serde.rs/enum-representations.html#untagged
///
/// _Note:_ there are multiple uses of renaming to camel case are related to this: https://github.com/serde-rs/serde/issues/1560
#[remain::sorted]
#[derive(Debug, Deserialize, Serialize, EnumDiscriminants)]
#[serde(untagged, rename_all = "camelCase")]
pub enum AuditLogMetadataV1 {
    #[serde(rename_all = "camelCase")]
    AbandonChangeSet { from_status: ChangeSetStatus },
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
    ApproveChangeSetApply { from_status: ChangeSetStatus },
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
    CreateSecret { name: String, secret_id: SecretId },
    #[serde(rename_all = "camelCase")]
    DeleteComponent {
        name: String,
        component_id: ComponentId,
        schema_variant_id: SchemaVariantId,
        schema_variant_name: String,
    },
    #[serde(rename_all = "camelCase")]
    DeleteSecret { name: String, secret_id: SecretId },
    #[serde(rename_all = "camelCase")]
    ExportWorkspace {
        id: WorkspacePk,
        name: String,
        version: String,
    },
    #[serde(rename_all = "camelCase")]
    InstallWorkspace {
        id: WorkspacePk,
        name: String,
        version: String,
    },
    #[serde(rename_all = "camelCase")]
    Login,
    #[serde(rename_all = "camelCase")]
    PutActionOnHold {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
    },
    #[serde(rename_all = "camelCase")]
    RejectChangeSetApply { from_status: ChangeSetStatus },
    #[serde(rename_all = "camelCase")]
    ReopenChangeSet { from_status: ChangeSetStatus },
    #[serde(rename_all = "camelCase")]
    RequestChangeSetApproval { from_status: ChangeSetStatus },
    #[serde(rename_all = "camelCase")]
    RetryAction {
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
    #[serde(rename_all = "camelCase")]
    UpdateSecret { name: String, secret_id: SecretId },
    #[serde(rename_all = "camelCase")]
    UpgradeComponent {
        name: String,
        component_id: ComponentId,
        schema_id: SchemaId,
        new_schema_variant_id: SchemaVariantId,
        new_schema_variant_name: String,
        old_schema_variant_id: SchemaVariantId,
        old_schema_variant_name: String,
    },
    #[serde(rename_all = "camelCase")]
    WithdrawRequestForChangeSetApply { from_status: ChangeSetStatus },
}

impl AuditLogMetadataV1 {
    pub fn title_and_entity_type(&self) -> (&'static str, Option<&'static str>) {
        // Please keep this in alphabetical order!
        // #[remain::sorted] // NOTE(nick): this is not yet stable
        match self.into() {
            MetadataDiscrim::AbandonChangeSet => ("Abandoned", Some("Change Set")),
            MetadataDiscrim::AddAction => ("Enqueued", Some("Action")),
            MetadataDiscrim::ApplyChangeSet => ("Applied", Some("Change Set")),
            MetadataDiscrim::ApproveChangeSetApply => {
                ("Approved Request to Apply", Some("Change Set"))
            }
            MetadataDiscrim::CancelAction => ("Removed", Some("Action")),
            MetadataDiscrim::CreateChangeSet => ("Created", Some("Change Set")),
            MetadataDiscrim::CreateComponent => ("Created", Some("Component")),
            MetadataDiscrim::CreateSecret => ("Created", Some("Secret")),
            MetadataDiscrim::DeleteComponent => ("Deleted", Some("Component")),
            MetadataDiscrim::DeleteSecret => ("Deleted", Some("Secret")),
            MetadataDiscrim::ExportWorkspace => ("Exported", Some("Workspace")),
            MetadataDiscrim::InstallWorkspace => ("Installed", Some("Workspace")),
            MetadataDiscrim::Login => ("Authenticated", None),
            MetadataDiscrim::PutActionOnHold => ("Paused", Some("Action")),
            MetadataDiscrim::RejectChangeSetApply => {
                ("Rejected Request to Apply", Some("Change Set"))
            }
            MetadataDiscrim::ReopenChangeSet => ("Reopened", Some("Change Set")),
            MetadataDiscrim::RequestChangeSetApproval => ("Requested to Apply", Some("Change Set")),
            MetadataDiscrim::RetryAction => ("Retried", Some("Action")),
            MetadataDiscrim::RunAction => ("Ran", Some("Action")),
            MetadataDiscrim::UpdateDependentInputSocket => ("Set Dependent", Some("Input Socket")),
            MetadataDiscrim::UpdateDependentOutputSocket => {
                ("Set Dependent", Some("Output Socket"))
            }
            MetadataDiscrim::UpdateDependentProperty => ("Set Dependent", Some("Property")),
            MetadataDiscrim::UpdatePropertyEditorValue => ("Updated Component", Some("Property")),
            MetadataDiscrim::UpdatePropertyEditorValueForSecret => {
                ("Updated Component", Some("Property for Secret"))
            }
            MetadataDiscrim::UpdateSecret => ("Updated", Some("Secret")),
            MetadataDiscrim::UpgradeComponent => ("Upgraded", Some("Component")),
            MetadataDiscrim::WithdrawRequestForChangeSetApply => {
                ("Withdrew Request to Apply", Some("Change Set"))
            }
        }
    }
}

impl From<Kind> for Metadata {
    fn from(value: Kind) -> Self {
        // Please keep this in alphabetical order!
        // #[remain::sorted] // NOTE(nick): this is not yet stable
        match value {
            Kind::AbandonChangeSet { from_status } => Self::AbandonChangeSet { from_status },
            Kind::AddAction {
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
            Kind::ApplyChangeSet => Self::ApplyChangeSet,
            Kind::ApproveChangeSetApply { from_status } => {
                Self::ApproveChangeSetApply { from_status }
            }
            Kind::CancelAction {
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
            Kind::CreateChangeSet => Self::CreateChangeSet,
            Kind::CreateComponent {
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
            Kind::CreateSecret { name, secret_id } => Self::CreateSecret { name, secret_id },
            Kind::DeleteComponent {
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
            Kind::DeleteSecret { name, secret_id } => Self::DeleteSecret { name, secret_id },
            Kind::ExportWorkspace { id, name, version } => {
                Self::ExportWorkspace { id, name, version }
            }
            Kind::InstallWorkspace { id, name, version } => {
                Self::InstallWorkspace { id, name, version }
            }
            Kind::Login => Self::Login,
            Kind::PutActionOnHold {
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
            Kind::RejectChangeSetApply { from_status } => {
                Self::RejectChangeSetApply { from_status }
            }
            Kind::ReopenChangeSet { from_status } => Self::ReopenChangeSet { from_status },
            Kind::RequestChangeSetApproval { from_status } => {
                Self::RequestChangeSetApproval { from_status }
            }
            Kind::RetryAction {
                prototype_id,
                action_kind,
                func_id,
                func_display_name,
                func_name,
            } => Self::RetryAction {
                prototype_id,
                action_kind,
                func_id,
                func_display_name,
                func_name,
            },
            Kind::RunAction {
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
            Kind::UpdateDependentInputSocket {
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
            Kind::UpdateDependentOutputSocket {
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
            Kind::UpdateDependentProperty {
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
            Kind::UpdatePropertyEditorValue {
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
            Kind::UpdatePropertyEditorValueForSecret {
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
            Kind::UpdateSecret { name, secret_id } => Self::UpdateSecret { name, secret_id },
            Kind::UpgradeComponent {
                name,
                component_id,
                schema_id,
                new_schema_variant_id,
                new_schema_variant_name,
                old_schema_variant_id,
                old_schema_variant_name,
            } => Self::UpgradeComponent {
                name,
                component_id,
                schema_id,
                new_schema_variant_id,
                new_schema_variant_name,
                old_schema_variant_id,
                old_schema_variant_name,
            },
            Kind::WithdrawRequestForChangeSetApply { from_status } => {
                Self::WithdrawRequestForChangeSetApply { from_status }
            }
        }
    }
}
