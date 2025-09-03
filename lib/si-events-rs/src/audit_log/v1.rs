use serde::{
    Deserialize,
    Serialize,
};
use si_id::{
    ApprovalRequirementDefinitionId,
    EntityId,
    ManagementPrototypeId,
    UserPk,
};
use strum::{
    Display,
    EnumDiscriminants,
};

use crate::{
    ActionKind,
    ActionPrototypeId,
    Actor,
    AttributeValueId,
    AuthenticationMethod,
    ChangeSetId,
    ChangeSetStatus,
    ComponentId,
    FuncArgumentId,
    FuncId,
    FuncKind,
    FuncRunId,
    InputSocketId,
    OutputSocketId,
    PropId,
    SchemaId,
    SchemaVariantId,
    SecretId,
    ViewId,
    WorkspacePk,
    func_run::FuncArgumentKind,
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
    pub authentication_method: Option<AuthenticationMethod>,
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
        component_id: Option<ComponentId>,
    },
    AddApprover {
        approval_requirement_definition_id: ApprovalRequirementDefinitionId,
        entity_name: Option<String>,
        entity_kind: String,
        entity_id: EntityId,
        user_id: UserPk,
    },
    ApplyChangeSet,
    ApproveChangeSetApply {
        from_status: ChangeSetStatus,
    },
    AttachActionFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
        schema_variant_id: Option<SchemaVariantId>,
        component_id: Option<ComponentId>,
        action_kind: Option<ActionKind>,
    },
    AttachAttributeFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
        schema_variant_id: Option<SchemaVariantId>,
        component_id: Option<ComponentId>,
        subject_name: String,
        prop_id: Option<PropId>,
        output_socket_id: Option<OutputSocketId>,
        destination_name: String,
    },
    AttachAuthFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
        schema_variant_id: Option<SchemaVariantId>,
    },
    AttachCodeGenFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
        schema_variant_id: Option<SchemaVariantId>,
        component_id: Option<ComponentId>,
        subject_name: String,
    },
    AttachManagementFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
        schema_variant_id: Option<SchemaVariantId>,
        component_id: Option<ComponentId>,
        subject_name: String,
    },
    AttachQualificationFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
        schema_variant_id: Option<SchemaVariantId>,
        component_id: Option<ComponentId>,
        subject_name: String,
    },
    CancelAction {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
        component_id: Option<ComponentId>,
    },
    ContributeModule {
        version: String,
        schema_id: Option<SchemaId>,
        schema_variant_id: Option<SchemaVariantId>,
        schema_variant_version: Option<String>,
    },
    CreateApprovalRequirementDefinition {
        individual_approvers: Vec<UserPk>,
        approval_requirement_definition_id: ApprovalRequirementDefinitionId,
        entity_name: Option<String>,
        entity_kind: String,
        entity_id: EntityId,
    },
    CreateChangeSet,
    CreateComponent {
        name: String,
        component_id: ComponentId,
        schema_variant_id: SchemaVariantId,
        schema_variant_name: String,
    },
    CreateConnection {
        from_component_id: ComponentId,
        from_component_name: String,
        from_socket_id: OutputSocketId,
        from_socket_name: String,
        to_component_id: ComponentId,
        to_component_name: String,
        to_socket_id: InputSocketId,
        to_socket_name: String,
    },
    CreateFunc {
        func_display_name: Option<String>,
        func_kind: FuncKind,
    },
    CreateFuncArgument {
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
        kind: FuncArgumentKind,
        element_kind: Option<FuncArgumentKind>,
    },
    CreateSchemaVariant {
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
    },
    CreateSecret {
        name: String,
        secret_id: SecretId,
    },
    CreateView {
        view_id: ViewId,
    },
    DeleteApprovalRequirementDefinition {
        approval_requirement_definition_id: ApprovalRequirementDefinitionId,
        entity_name: Option<String>,
        entity_kind: String,
        entity_id: EntityId,
        individual_approvers: Vec<UserPk>,
        // later add approver_groups once implemented for users
    },
    DeleteComponent {
        name: String,
        component_id: ComponentId,
        schema_variant_id: SchemaVariantId,
        schema_variant_name: String,
    },
    DeleteConnection {
        from_component_id: ComponentId,
        from_component_name: String,
        from_socket_id: OutputSocketId,
        from_socket_name: String,
        to_component_id: ComponentId,
        to_component_name: String,
        to_socket_id: InputSocketId,
        to_socket_name: String,
    },
    DeleteFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
        func_kind: FuncKind,
    },
    DeleteFuncArgument {
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
        func_argument_id: FuncArgumentId,
    },
    DeleteSchemaVariant {
        schema_variant_id: SchemaVariantId,
        schema_id: SchemaId,
    },
    DeleteSecret {
        name: String,
        secret_id: SecretId,
    },
    DeleteView {
        view_id: ViewId,
    },
    DetachFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
        schema_variant_id: Option<SchemaVariantId>,
        component_id: Option<ComponentId>,
        subject_name: String,
    },
    EraseComponent {
        name: String,
        component_id: ComponentId,
        schema_variant_id: SchemaVariantId,
        schema_variant_name: String,
    },
    ExecuteFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
    },
    ExportWorkspace {
        id: WorkspacePk,
        name: String,
        version: String,
    },

    GenerateTemplate {
        schema_variant_id: SchemaVariantId,
        management_prototype_id: ManagementPrototypeId,
        func_id: FuncId,
        func_name: String,
        asset_name: String,
    },
    HoldAction {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
        component_id: Option<ComponentId>,
    },

    InstallWorkspace {
        id: WorkspacePk,
        name: String,
        version: String,
    },
    Login,

    ManagementOperationsComplete {
        component_id: ComponentId,
        prototype_id: ManagementPrototypeId,
        func_id: FuncId,
        func_name: String,
        status: String,
        message: Option<String>,
    },

    OrphanComponent {
        component_id: ComponentId,
        previous_parent_id: ComponentId,
        previous_parent_name: String,
    },
    PurgeOpenChangeSets {
        change_set_ids: Vec<ChangeSetId>,
    },
    PutActionOnHold {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
    },
    RegenerateSchemaVariant {
        schema_variant_id: SchemaVariantId,
    },
    RejectChangeSetApply {
        from_status: ChangeSetStatus,
    },
    RemoveApprover {
        approval_requirement_definition_id: ApprovalRequirementDefinitionId,
        entity_name: Option<String>,
        entity_kind: String,
        entity_id: EntityId,
        user_id: UserPk,
    },
    RemoveDefaultSubscriptionSource {
        component_id: ComponentId,
        av_id: AttributeValueId,
        av_identifier: String,
    },
    RenameComponent {
        component_id: ComponentId,
        old_name: String,
        new_name: String,
    },
    ReopenChangeSet {
        from_status: ChangeSetStatus,
    },
    RequestChangeSetApproval {
        from_status: ChangeSetStatus,
    },
    RestoreComponent {
        name: String,
        component_id: ComponentId,
        before_to_delete: bool,
        schema_id: SchemaId,
        schema_name: String,
        schema_variant_id: SchemaVariantId,
        schema_variant_name: String,
    },
    RetryAction {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
        component_id: Option<ComponentId>,
    },
    RunAction {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
        run_status: bool,
    },
    SetAttribute {
        component_id: ComponentId,
        attribute_value_id: AttributeValueId,
        path: String,
        before_value: Option<PropValueSource>,
        after_value: Option<PropValueSource>,
    },
    SetDefaultSubscriptionSource {
        component_id: ComponentId,
        av_id: AttributeValueId,
        av_identifier: String,
    },
    TestFunction {
        func_id: FuncId,
        func_display_name: Option<String>,
        func_run_id: FuncRunId,
    },
    UnlockFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
        schema_variant_id: Option<SchemaVariantId>,
        component_id: Option<ComponentId>,
        subject_name: Option<String>,
    },
    UnlockSchemaVariant {
        schema_variant_id: SchemaVariantId,
        schema_variant_display_name: String,
    },
    UnsetAttribute {
        component_id: ComponentId,
        attribute_value_id: AttributeValueId,
        path: String,
        before_value: Option<PropValueSource>,
    },
    UpdateComponent {
        component_id: ComponentId,
        component_name: String,
    },
    UpdateComponentParent {
        component_id: ComponentId,
        old_parent_id: Option<ComponentId>,
        old_parent_name: Option<String>,
        new_parent_id: ComponentId,
        new_parent_name: String,
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
    UpdateFuncMetadata {
        func_id: FuncId,
        old_display_name: Option<String>,
        new_display_name: Option<String>,
        old_description: Option<String>,
        new_description: Option<String>,
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
    UpdateSchemaVariant {
        old_display_name: String,
        new_display_name: String,
        old_description: String,
        new_description: String,
        old_category: String,
        new_category: String,
        old_link: String,
        new_link: String,
        old_color: String,
        new_color: String,
        old_component_type: String,
        new_component_type: String,
        //todo: what to do about the code?
    },
    UpdateSecret {
        name: String,
        secret_id: SecretId,
    },
    UpdateView {
        view_id: ViewId,
        old_name: String,
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
    WorkspaceIntegration {
        old_slack_webhook_url: String,
        new_slack_webhook_url: String,
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
        component_id: Option<ComponentId>,
    },
    #[serde(rename_all = "camelCase")]
    AddApprover {
        approval_requirement_definition_id: ApprovalRequirementDefinitionId,
        entity_name: Option<String>,
        entity_kind: String,
        entity_id: EntityId,
        user_id: UserPk,
    },
    #[serde(rename_all = "camelCase")]
    ApplyChangeSet,
    #[serde(rename_all = "camelCase")]
    ApproveChangeSetApply { from_status: ChangeSetStatus },
    #[serde(rename_all = "camelCase")]
    AttachActionFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
        schema_variant_id: Option<SchemaVariantId>,
        component_id: Option<ComponentId>,
        action_kind: Option<ActionKind>,
    },
    #[serde(rename_all = "camelCase")]
    AttachAttributeFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
        schema_variant_id: Option<SchemaVariantId>,
        component_id: Option<ComponentId>,
        subject_name: String,
        prop_id: Option<PropId>,
        output_socket_id: Option<OutputSocketId>,
        destination_name: String,
    },
    #[serde(rename_all = "camelCase")]
    AttachAuthFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
        schema_variant_id: Option<SchemaVariantId>,
    },
    #[serde(rename_all = "camelCase")]
    AttachCodeGenFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
        schema_variant_id: Option<SchemaVariantId>,
        component_id: Option<ComponentId>,
        subject_name: String,
    },
    #[serde(rename_all = "camelCase")]
    AttachManagementFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
        schema_variant_id: Option<SchemaVariantId>,
        component_id: Option<ComponentId>,
        subject_name: String,
    },
    #[serde(rename_all = "camelCase")]
    AttachQualificationFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
        schema_variant_id: Option<SchemaVariantId>,
        component_id: Option<ComponentId>,
        subject_name: String,
    },
    #[serde(rename_all = "camelCase")]
    CancelAction {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
        component_id: Option<ComponentId>,
    },
    #[serde(rename_all = "camelCase")]
    ContributeModule {
        version: String,
        schema_id: Option<SchemaId>,
        schema_variant_id: Option<SchemaVariantId>,
        schema_variant_version: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    CreateApprovalRequirementDefinition {
        individual_approvers: Vec<UserPk>,
        approval_requirement_definition_id: ApprovalRequirementDefinitionId,
        entity_name: Option<String>,
        entity_kind: String,
        entity_id: EntityId,
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
    CreateConnection {
        from_component_id: ComponentId,
        from_component_name: String,
        from_socket_id: OutputSocketId,
        from_socket_name: String,
        to_component_id: ComponentId,
        to_component_name: String,
        to_socket_id: InputSocketId,
        to_socket_name: String,
    },
    #[serde(rename_all = "camelCase")]
    CreateFunc {
        func_display_name: Option<String>,
        func_kind: FuncKind,
    },
    #[serde(rename_all = "camelCase")]
    CreateFuncArgument {
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
        kind: FuncArgumentKind,
        element_kind: Option<FuncArgumentKind>,
    },
    #[serde(rename_all = "camelCase")]
    CreateSchemaVariant {
        schema_id: SchemaId,
        schema_variant_id: SchemaVariantId,
    },
    #[serde(rename_all = "camelCase")]
    CreateSecret { name: String, secret_id: SecretId },
    #[serde(rename_all = "camelCase")]
    CreateView { view_id: ViewId },
    #[serde(rename_all = "camelCase")]
    DeleteApprovalRequirementDefinition {
        individual_approvers: Vec<UserPk>,
        approval_requirement_definition_id: ApprovalRequirementDefinitionId,
        entity_name: Option<String>,
        entity_kind: String,
        entity_id: EntityId,
    },
    #[serde(rename_all = "camelCase")]
    DeleteComponent {
        name: String,
        component_id: ComponentId,
        schema_variant_id: SchemaVariantId,
        schema_variant_name: String,
    },
    #[serde(rename_all = "camelCase")]
    DeleteConnection {
        from_component_id: ComponentId,
        from_component_name: String,
        from_socket_id: OutputSocketId,
        from_socket_name: String,
        to_component_id: ComponentId,
        to_component_name: String,
        to_socket_id: InputSocketId,
        to_socket_name: String,
    },
    #[serde(rename_all = "camelCase")]
    DeleteFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
        func_kind: FuncKind,
    },
    #[serde(rename_all = "camelCase")]
    DeleteFuncArgument {
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
        func_argument_id: FuncArgumentId,
    },
    #[serde(rename_all = "camelCase")]
    DeleteSchemaVariant {
        schema_variant_id: SchemaVariantId,
        schema_id: SchemaId,
    },
    #[serde(rename_all = "camelCase")]
    DeleteSecret { name: String, secret_id: SecretId },
    #[serde(rename_all = "camelCase")]
    DeleteView { view_id: ViewId },
    #[serde(rename_all = "camelCase")]
    DetachFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
        schema_variant_id: Option<SchemaVariantId>,
        component_id: Option<ComponentId>,
        subject_name: String,
    },
    #[serde(rename_all = "camelCase")]
    EraseComponent {
        name: String,
        component_id: ComponentId,
        schema_variant_id: SchemaVariantId,
        schema_variant_name: String,
    },
    #[serde(rename_all = "camelCase")]
    ExecuteFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    ExportWorkspace {
        id: WorkspacePk,
        name: String,
        version: String,
    },

    #[serde(rename_all = "camelCase")]
    GenerateTemplate {
        schema_variant_id: SchemaVariantId,
        management_prototype_id: ManagementPrototypeId,
        func_id: FuncId,
        func_name: String,
        asset_name: String,
    },
    #[serde(rename_all = "camelCase")]
    HoldAction {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
        component_id: Option<ComponentId>,
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
    ManagementOperationsComplete {
        component_id: ComponentId,
        prototype_id: ManagementPrototypeId,
        func_id: FuncId,
        func_name: String,
        status: String,
        message: Option<String>,
    },

    #[serde(rename_all = "camelCase")]
    OrphanComponent {
        component_id: ComponentId,
        previous_parent_id: ComponentId,
        previous_parent_name: String,
    },
    #[serde(rename_all = "camelCase")]
    PurgeOpenChangeSets { change_set_ids: Vec<ChangeSetId> },
    #[serde(rename_all = "camelCase")]
    PutActionOnHold {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
    },
    #[serde(rename_all = "camelCase")]
    RegenerateSchemaVariant { schema_variant_id: SchemaVariantId },
    #[serde(rename_all = "camelCase")]
    RejectChangeSetApply { from_status: ChangeSetStatus },
    #[serde(rename_all = "camelCase")]
    RemoveApprover {
        approval_requirement_definition_id: ApprovalRequirementDefinitionId,
        entity_name: Option<String>,
        entity_kind: String,
        entity_id: EntityId,
        user_id: UserPk,
    },
    #[serde(rename_all = "camelCase")]
    RemoveDefaultSubscriptionSource {
        component_id: ComponentId,
        av_id: AttributeValueId,
        av_identifier: String,
    },
    #[serde(rename_all = "camelCase")]
    RenameComponent {
        component_id: ComponentId,
        old_name: String,
        new_name: String,
    },
    #[serde(rename_all = "camelCase")]
    ReopenChangeSet { from_status: ChangeSetStatus },
    #[serde(rename_all = "camelCase")]
    RequestChangeSetApproval { from_status: ChangeSetStatus },
    #[serde(rename_all = "camelCase")]
    RestoreComponent {
        name: String,
        component_id: ComponentId,
        before_to_delete: bool,
        schema_id: SchemaId,
        schema_name: String,
        schema_variant_id: SchemaVariantId,
        schema_variant_name: String,
    },
    #[serde(rename_all = "camelCase")]
    RetryAction {
        prototype_id: ActionPrototypeId,
        action_kind: ActionKind,
        func_id: FuncId,
        func_display_name: Option<String>,
        func_name: String,
        component_id: Option<ComponentId>,
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
    SetAttribute {
        component_id: ComponentId,
        attribute_value_id: AttributeValueId,
        path: String,
        before_value: Option<PropValueSource>,
        after_value: Option<PropValueSource>,
    },
    #[serde(rename_all = "camelCase")]
    SetDefaultSubscriptionSource {
        component_id: ComponentId,
        av_id: AttributeValueId,
        av_identifier: String,
    },
    #[serde(rename_all = "camelCase")]
    TestFunction {
        func_id: FuncId,
        func_display_name: Option<String>,
        func_run_id: FuncRunId,
    },
    #[serde(rename_all = "camelCase")]
    UnlockFunc {
        func_id: FuncId,
        func_display_name: Option<String>,
        schema_variant_id: Option<SchemaVariantId>,
        component_id: Option<ComponentId>,
        subject_name: Option<String>,
    },
    #[serde(rename_all = "camelCase")]
    UnlockSchemaVariant {
        schema_variant_id: SchemaVariantId,
        schema_variant_display_name: String,
    },
    #[serde(rename_all = "camelCase")]
    UnsetAttribute {
        component_id: ComponentId,
        attribute_value_id: AttributeValueId,
        path: String,
        before_value: Option<PropValueSource>,
    },
    #[serde(rename_all = "camelCase")]
    UpdateComponent {
        component_id: ComponentId,
        component_name: String,
    },
    #[serde(rename_all = "camelCase")]
    UpdateComponentParent {
        component_id: ComponentId,
        old_parent_id: Option<ComponentId>,
        old_parent_name: Option<String>,
        new_parent_id: ComponentId,
        new_parent_name: String,
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
    UpdateFuncMetadata {
        func_id: FuncId,
        old_display_name: Option<String>,
        new_display_name: Option<String>,
        old_description: Option<String>,
        new_description: Option<String>,
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
    UpdateSchemaVariant {
        old_display_name: String,
        new_display_name: String,
        old_description: String,
        new_description: String,
        old_category: String,
        new_category: String,
        old_link: String,
        new_link: String,
        old_color: String,
        new_color: String,
        old_component_type: String,
        new_component_type: String,
        //todo: what to do about the code?
    },
    #[serde(rename_all = "camelCase")]
    UpdateSecret { name: String, secret_id: SecretId },
    #[serde(rename_all = "camelCase")]
    UpdateView { view_id: ViewId, old_name: String },
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
    #[serde(rename_all = "camelCase")]
    WorkspaceIntegration {
        old_slack_webhook_url: String,
        new_slack_webhook_url: String,
    },
}

impl AuditLogMetadataV1 {
    pub fn title_and_entity_type(&self) -> (&'static str, Option<&'static str>) {
        // Please keep this in alphabetical order!
        // #[remain::sorted] // NOTE(nick): this is not yet stable
        match self.into() {
            MetadataDiscrim::AbandonChangeSet => ("Abandoned", Some("Change Set")),
            MetadataDiscrim::AddAction => ("Enqueued", Some("Action")),
            MetadataDiscrim::AddApprover => ("User Added", Some("Approval Requirement Definition")),
            MetadataDiscrim::ApplyChangeSet => ("Applied", Some("Change Set")),
            MetadataDiscrim::ApproveChangeSetApply => {
                ("Approved Request to Apply", Some("Change Set"))
            }
            MetadataDiscrim::AttachActionFunc => ("Attached", Some("Action Function")),
            MetadataDiscrim::AttachAttributeFunc => ("Attached", Some("Attribute Function")),
            MetadataDiscrim::AttachAuthFunc => ("Attached", Some("Authentication Function")),
            MetadataDiscrim::AttachCodeGenFunc => ("Attached", Some("Code Generation Function")),
            MetadataDiscrim::AttachManagementFunc => ("Attached", Some("Management Function")),
            MetadataDiscrim::AttachQualificationFunc => {
                ("Attached", Some("Qualification Function"))
            }
            MetadataDiscrim::CancelAction => ("Removed", Some("Action")),
            MetadataDiscrim::ContributeModule => ("Contributed", Some("Module")),
            MetadataDiscrim::CreateApprovalRequirementDefinition => {
                ("Created", Some("Approval Requirement Definition"))
            }
            MetadataDiscrim::CreateChangeSet => ("Created", Some("Change Set")),
            MetadataDiscrim::CreateComponent => ("Created", Some("Component")),
            MetadataDiscrim::CreateConnection => ("Created", Some("Connection")),
            MetadataDiscrim::CreateFunc => ("Created", Some("Function")),
            MetadataDiscrim::CreateFuncArgument => ("Created", Some("Function Argument")),
            MetadataDiscrim::CreateSchemaVariant => ("Created", Some("Schema Variant")),
            MetadataDiscrim::CreateSecret => ("Created", Some("Secret")),
            MetadataDiscrim::CreateView => ("Created", Some("View")),
            MetadataDiscrim::DeleteApprovalRequirementDefinition => {
                ("Deleted", Some("Approval Requirement Definition"))
            }
            MetadataDiscrim::DeleteComponent => ("Deleted", Some("Component")),
            MetadataDiscrim::DeleteConnection => ("Deleted", Some("Connection")),
            MetadataDiscrim::DeleteFunc => ("Deleted", Some("Function")),
            MetadataDiscrim::DeleteFuncArgument => ("Deleted", Some("Function Argument")),
            MetadataDiscrim::DeleteSchemaVariant => ("Deleted", Some("Schema Variant")),
            MetadataDiscrim::DeleteSecret => ("Deleted", Some("Secret")),
            MetadataDiscrim::DeleteView => ("Deleted", Some("View")),
            MetadataDiscrim::DetachFunc => ("Detached", Some("Function")),
            MetadataDiscrim::EraseComponent => ("Erased", Some("Component")),
            MetadataDiscrim::ExecuteFunc => ("Executed", Some("Function")),
            MetadataDiscrim::ExportWorkspace => ("Exported", Some("Workspace")),
            MetadataDiscrim::InstallWorkspace => ("Installed", Some("Workspace")),
            MetadataDiscrim::GenerateTemplate => ("Generated", Some("Template")),
            MetadataDiscrim::HoldAction => ("Held", Some("Action")),
            MetadataDiscrim::Login => ("Authenticated", None),
            MetadataDiscrim::ManagementOperationsComplete => {
                ("Executed", Some("Management Operations"))
            }
            MetadataDiscrim::OrphanComponent => ("Orphaned", Some("Component")),
            MetadataDiscrim::PurgeOpenChangeSets => ("Purged Open", Some("Change Sets")),
            MetadataDiscrim::PutActionOnHold => ("Paused", Some("Action")),
            MetadataDiscrim::RegenerateSchemaVariant => ("Regenerated", Some("Schema Variant")),
            MetadataDiscrim::RejectChangeSetApply => {
                ("Rejected Request to Apply", Some("Change Set"))
            }
            MetadataDiscrim::RemoveApprover => {
                ("User removed", Some("Approval Requirement Definition"))
            }
            MetadataDiscrim::RemoveDefaultSubscriptionSource => {
                ("Removed Default", Some("Subscription Source"))
            }
            MetadataDiscrim::RenameComponent => ("Renamed", Some("Component")),
            MetadataDiscrim::ReopenChangeSet => ("Reopened", Some("Change Set")),
            MetadataDiscrim::RequestChangeSetApproval => ("Requested to Apply", Some("Change Set")),
            MetadataDiscrim::RestoreComponent => ("Restored", Some("Component")),
            MetadataDiscrim::RetryAction => ("Retried", Some("Action")),
            MetadataDiscrim::RunAction => ("Ran", Some("Action")),
            MetadataDiscrim::SetAttribute => ("Set", Some("Attribute")),
            MetadataDiscrim::SetDefaultSubscriptionSource => {
                ("Set Default", Some("Subscription Source"))
            }
            MetadataDiscrim::TestFunction => ("Tested", Some("Function")),
            MetadataDiscrim::UnlockFunc => ("Unlocked", Some("Function")),
            MetadataDiscrim::UnlockSchemaVariant => ("Unlocked", Some("Schema Variant")),
            MetadataDiscrim::UnsetAttribute => ("Unset", Some("Attribute")),
            MetadataDiscrim::UpdateComponent => ("Updated", Some("Component")),
            MetadataDiscrim::UpdateComponentParent => ("Updated Parent", Some("Component")),
            MetadataDiscrim::UpdateDependentInputSocket => ("Set Dependent", Some("Input Socket")),
            MetadataDiscrim::UpdateDependentOutputSocket => {
                ("Set Dependent", Some("Output Socket"))
            }
            MetadataDiscrim::UpdateDependentProperty => ("Set Dependent", Some("Property")),
            MetadataDiscrim::UpdateFuncMetadata => ("Updated Func", Some("Metadata")),
            MetadataDiscrim::UpdatePropertyEditorValue => ("Updated Component", Some("Property")),
            MetadataDiscrim::UpdatePropertyEditorValueForSecret => {
                ("Updated Component", Some("Property for Secret"))
            }
            MetadataDiscrim::UpdateSecret => ("Updated", Some("Secret")),
            MetadataDiscrim::UpdateSchemaVariant => ("Updated", Some("Schema Variant")),
            MetadataDiscrim::UpdateView => ("Updated", Some("View")),
            MetadataDiscrim::UpgradeComponent => ("Upgraded", Some("Component")),
            MetadataDiscrim::WithdrawRequestForChangeSetApply => {
                ("Withdrew Request to Apply", Some("Change Set"))
            }
            MetadataDiscrim::WorkspaceIntegration => ("Workspace Integration Updated", None),
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
                component_id,
            } => Self::AddAction {
                prototype_id,
                action_kind,
                func_id,
                func_display_name,
                func_name,
                component_id,
            },
            Kind::AddApprover {
                approval_requirement_definition_id,
                entity_name,
                user_id,
                entity_kind,
                entity_id,
            } => Self::AddApprover {
                approval_requirement_definition_id,
                entity_name,
                user_id,
                entity_kind,
                entity_id,
            },
            Kind::ApplyChangeSet => Self::ApplyChangeSet,
            Kind::ApproveChangeSetApply { from_status } => {
                Self::ApproveChangeSetApply { from_status }
            }
            Kind::AttachActionFunc {
                func_id,
                func_display_name,
                schema_variant_id,
                component_id,
                action_kind,
            } => Self::AttachActionFunc {
                func_id,
                func_display_name,
                schema_variant_id,
                component_id,
                action_kind,
            },
            Kind::AttachAttributeFunc {
                func_id,
                func_display_name,
                schema_variant_id,
                component_id,
                subject_name,
                prop_id,
                output_socket_id,
                destination_name,
            } => Self::AttachAttributeFunc {
                func_id,
                func_display_name,
                schema_variant_id,
                component_id,
                subject_name,
                prop_id,
                output_socket_id,
                destination_name,
            },
            Kind::AttachAuthFunc {
                func_id,
                func_display_name,
                schema_variant_id,
            } => Self::AttachAuthFunc {
                func_id,
                func_display_name,
                schema_variant_id,
            },
            Kind::AttachCodeGenFunc {
                func_id,
                func_display_name,
                schema_variant_id,
                component_id,
                subject_name,
            } => Self::AttachCodeGenFunc {
                func_id,
                func_display_name,
                schema_variant_id,
                component_id,
                subject_name,
            },
            Kind::AttachManagementFunc {
                func_id,
                func_display_name,
                schema_variant_id,
                component_id,
                subject_name,
            } => Self::AttachManagementFunc {
                func_id,
                func_display_name,
                schema_variant_id,
                component_id,
                subject_name,
            },
            Kind::AttachQualificationFunc {
                func_id,
                func_display_name,
                schema_variant_id,
                component_id,
                subject_name,
            } => Self::AttachQualificationFunc {
                func_id,
                func_display_name,
                schema_variant_id,
                component_id,
                subject_name,
            },
            Kind::CancelAction {
                prototype_id,
                action_kind,
                func_id,
                func_display_name,
                func_name,
                component_id,
            } => Self::CancelAction {
                prototype_id,
                action_kind,
                func_id,
                func_display_name,
                func_name,
                component_id,
            },
            Kind::ContributeModule {
                version,
                schema_id,
                schema_variant_id,
                schema_variant_version,
            } => Self::ContributeModule {
                version,
                schema_id,
                schema_variant_id,
                schema_variant_version,
            },
            Kind::CreateApprovalRequirementDefinition {
                individual_approvers: approvers,
                approval_requirement_definition_id,
                entity_name,
                entity_kind,
                entity_id,
            } => Self::CreateApprovalRequirementDefinition {
                individual_approvers: approvers,
                approval_requirement_definition_id,
                entity_name,
                entity_kind,
                entity_id,
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
            Kind::CreateConnection {
                from_component_id,
                from_component_name,
                from_socket_id,
                from_socket_name,
                to_component_id,
                to_component_name,
                to_socket_id,
                to_socket_name,
            } => Self::CreateConnection {
                from_component_id,
                from_component_name,
                from_socket_id,
                from_socket_name,
                to_component_id,
                to_component_name,
                to_socket_id,
                to_socket_name,
            },
            Kind::CreateFunc {
                func_display_name,
                func_kind,
            } => Self::CreateFunc {
                func_display_name,
                func_kind,
            },
            Kind::CreateFuncArgument {
                func_id,
                func_display_name,
                func_name,
                kind,
                element_kind,
            } => Self::CreateFuncArgument {
                func_id,
                func_display_name,
                func_name,
                kind,
                element_kind,
            },
            Kind::CreateSchemaVariant {
                schema_id,
                schema_variant_id,
            } => Self::CreateSchemaVariant {
                schema_id,
                schema_variant_id,
            },
            Kind::CreateSecret { name, secret_id } => Self::CreateSecret { name, secret_id },
            Kind::CreateView { view_id } => Self::CreateView { view_id },
            Kind::DeleteApprovalRequirementDefinition {
                individual_approvers,
                approval_requirement_definition_id,
                entity_name,
                entity_kind,
                entity_id,
            } => Self::DeleteApprovalRequirementDefinition {
                approval_requirement_definition_id,
                individual_approvers,
                entity_name,
                entity_kind,
                entity_id,
            },
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
            Kind::DeleteConnection {
                from_component_id,
                from_component_name,
                from_socket_id,
                from_socket_name,
                to_component_id,
                to_component_name,
                to_socket_id,
                to_socket_name,
            } => Self::DeleteConnection {
                from_component_id,
                from_component_name,
                from_socket_id,
                from_socket_name,
                to_component_id,
                to_component_name,
                to_socket_id,
                to_socket_name,
            },
            Kind::DeleteFunc {
                func_id,
                func_display_name,
                func_kind,
            } => Self::DeleteFunc {
                func_id,
                func_display_name,
                func_kind,
            },
            Kind::DeleteFuncArgument {
                func_id,
                func_display_name,
                func_name,
                func_argument_id,
            } => Self::DeleteFuncArgument {
                func_id,
                func_display_name,
                func_name,
                func_argument_id,
            },
            Kind::DeleteSchemaVariant {
                schema_variant_id,
                schema_id,
            } => Self::DeleteSchemaVariant {
                schema_variant_id,
                schema_id,
            },
            Kind::DeleteSecret { name, secret_id } => Self::DeleteSecret { name, secret_id },
            Kind::DeleteView { view_id } => Self::DeleteView { view_id },
            Kind::DetachFunc {
                func_id,
                func_display_name,
                schema_variant_id,
                component_id,
                subject_name,
            } => Self::DetachFunc {
                func_id,
                func_display_name,
                schema_variant_id,
                component_id,
                subject_name,
            },
            Kind::EraseComponent {
                name,
                component_id,
                schema_variant_id,
                schema_variant_name,
            } => Self::EraseComponent {
                name,
                component_id,
                schema_variant_id,
                schema_variant_name,
            },
            Kind::ExecuteFunc {
                func_id,
                func_display_name,
            } => Self::ExecuteFunc {
                func_id,
                func_display_name,
            },
            Kind::ExportWorkspace { id, name, version } => {
                Self::ExportWorkspace { id, name, version }
            }
            Kind::GenerateTemplate {
                schema_variant_id,
                management_prototype_id,
                func_id,
                func_name,
                asset_name,
            } => Self::GenerateTemplate {
                schema_variant_id,
                management_prototype_id,
                func_id,
                func_name,
                asset_name,
            },
            Kind::HoldAction {
                prototype_id,
                action_kind,
                func_id,
                func_display_name,
                func_name,
                component_id,
            } => Self::HoldAction {
                prototype_id,
                action_kind,
                func_id,
                func_display_name,
                func_name,
                component_id,
            },
            Kind::InstallWorkspace { id, name, version } => {
                Self::InstallWorkspace { id, name, version }
            }
            Kind::Login => Self::Login,
            Kind::ManagementOperationsComplete {
                component_id,
                prototype_id,
                func_id,
                func_name,
                status,
                message,
            } => Self::ManagementOperationsComplete {
                component_id,
                prototype_id,
                func_id,
                func_name,
                status,
                message,
            },
            Kind::OrphanComponent {
                component_id,
                previous_parent_id,
                previous_parent_name,
            } => Self::OrphanComponent {
                component_id,
                previous_parent_id,
                previous_parent_name,
            },
            Kind::PurgeOpenChangeSets { change_set_ids } => {
                Self::PurgeOpenChangeSets { change_set_ids }
            }
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
            Kind::RegenerateSchemaVariant { schema_variant_id } => {
                Self::RegenerateSchemaVariant { schema_variant_id }
            }
            Kind::RejectChangeSetApply { from_status } => {
                Self::RejectChangeSetApply { from_status }
            }
            Kind::RenameComponent {
                component_id,
                old_name,
                new_name,
            } => Self::RenameComponent {
                component_id,
                old_name,
                new_name,
            },
            Kind::RemoveApprover {
                approval_requirement_definition_id,
                entity_name,
                entity_kind,
                entity_id,
                user_id,
            } => Self::RemoveApprover {
                approval_requirement_definition_id,
                entity_name,
                entity_kind,
                entity_id,
                user_id,
            },
            Kind::RemoveDefaultSubscriptionSource {
                component_id,
                av_id,
                av_identifier,
            } => Self::RemoveDefaultSubscriptionSource {
                component_id,
                av_id,
                av_identifier,
            },
            Kind::ReopenChangeSet { from_status } => Self::ReopenChangeSet { from_status },
            Kind::RestoreComponent {
                name,
                component_id,
                before_to_delete,
                schema_id,
                schema_name,
                schema_variant_id,
                schema_variant_name,
            } => Self::RestoreComponent {
                name,
                component_id,
                before_to_delete,
                schema_id,
                schema_name,
                schema_variant_id,
                schema_variant_name,
            },
            Kind::RequestChangeSetApproval { from_status } => {
                Self::RequestChangeSetApproval { from_status }
            }
            Kind::RetryAction {
                prototype_id,
                action_kind,
                func_id,
                func_display_name,
                func_name,
                component_id,
            } => Self::RetryAction {
                prototype_id,
                action_kind,
                func_id,
                func_display_name,
                func_name,
                component_id,
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
            Kind::SetAttribute {
                component_id,
                attribute_value_id,
                path,
                before_value,
                after_value,
            } => Self::SetAttribute {
                component_id,
                attribute_value_id,
                path,
                before_value,
                after_value,
            },
            Kind::SetDefaultSubscriptionSource {
                component_id,
                av_id,
                av_identifier,
            } => Self::SetDefaultSubscriptionSource {
                component_id,
                av_id,
                av_identifier,
            },
            Kind::TestFunction {
                func_id,
                func_display_name,
                func_run_id,
            } => Self::TestFunction {
                func_id,
                func_display_name,
                func_run_id,
            },
            Kind::UnlockFunc {
                func_id,
                func_display_name,
                schema_variant_id,
                component_id,
                subject_name,
            } => Self::UnlockFunc {
                func_id,
                func_display_name,
                schema_variant_id,
                component_id,
                subject_name,
            },
            Kind::UnlockSchemaVariant {
                schema_variant_id,
                schema_variant_display_name,
            } => Self::UnlockSchemaVariant {
                schema_variant_id,
                schema_variant_display_name,
            },
            Kind::UnsetAttribute {
                component_id,
                attribute_value_id,
                path,
                before_value,
            } => Self::UnsetAttribute {
                component_id,
                attribute_value_id,
                path,
                before_value,
            },
            Kind::UpdateComponent {
                component_id,
                component_name,
            } => Self::UpdateComponent {
                component_id,
                component_name,
            },
            Kind::UpdateComponentParent {
                component_id,
                old_parent_id,
                old_parent_name,
                new_parent_id,
                new_parent_name,
            } => Self::UpdateComponentParent {
                component_id,
                old_parent_id,
                old_parent_name,
                new_parent_id,
                new_parent_name,
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
            Kind::UpdateFuncMetadata {
                func_id,
                old_display_name,
                new_display_name,
                old_description,
                new_description,
            } => Self::UpdateFuncMetadata {
                func_id,
                old_display_name,
                new_display_name,
                old_description,
                new_description,
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
            Kind::UpdateSchemaVariant {
                old_display_name,
                new_display_name,
                old_description,
                new_description,
                old_category,
                new_category,
                old_link,
                new_link,
                old_color,
                new_color,
                old_component_type,
                new_component_type,
            } => Self::UpdateSchemaVariant {
                old_display_name,
                new_display_name,
                old_description,
                new_description,
                old_category,
                new_category,
                old_link,
                new_link,
                old_color,
                new_color,
                old_component_type,
                new_component_type,
            },
            Kind::UpdateSecret { name, secret_id } => Self::UpdateSecret { name, secret_id },
            Kind::UpdateView { view_id, old_name } => Self::UpdateView { view_id, old_name },
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
            Kind::WorkspaceIntegration {
                old_slack_webhook_url,
                new_slack_webhook_url,
            } => Self::WorkspaceIntegration {
                old_slack_webhook_url,
                new_slack_webhook_url,
            },
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[allow(dead_code)]
pub enum PropValueSource {
    Value(serde_json::Value),
    Subscription {
        value: Option<serde_json::Value>,
        source_component_id: ComponentId,
        source_path: String,
    },
    None,
}
