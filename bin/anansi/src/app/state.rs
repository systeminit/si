use dal::{
    ChangeSetId,
    PropKind,
    Ulid,
    WorkspaceSnapshotGraph,
    action::{
        ActionState,
        prototype::ActionKind,
    },
    attribute::path::AttributePath,
    func::FuncKind,
    workspace_snapshot::{
        edge_weight::{
            EdgeWeight,
            EdgeWeightKind,
        },
        graph::NodeIndex,
        node_weight::{
            NodeWeight,
            category_node_weight::CategoryNodeKind,
        },
    },
};
use ratatui::layout::Rect;
use si_events::ContentHash;

use super::helpers::SnapshotStats;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusPanel {
    NodeList,
    NodeDetails,
    EdgePanel,
    EditHistory,
}

/// Represents which modal dialog is currently active
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActiveModal {
    /// The node edit modal
    EditNode,
    /// Confirmation dialog for deleting a node
    DeleteNodeConfirm,
    /// Confirmation dialog for deleting an edge
    DeleteEdgeConfirm,
    /// Modal for adding a new edge
    AddEdge,
    /// Confirmation dialog for saving the graph
    SaveConfirm,
    /// Modal displaying snapshot statistics
    Stats,
}

/// Represents the selected edge weight kind in the add edge modal
#[derive(Debug, Clone, PartialEq)]
pub enum SelectedEdgeWeightKind {
    Action,
    ActionPrototype,
    AuthenticationPrototype,
    Contain { key: Option<String> },
    DeprecatedFrameContains,
    Ordering,
    Ordinal,
    Prop,
    Prototype { key: Option<String> },
    PrototypeArgument,
    PrototypeArgumentValue,
    Proxy,
    Root,
    Socket,
    SocketValue,
    Use { is_default: bool },
    ValidationOutput,
    ManagementPrototype,
    Represents,
    Manages,
    DiagramObject,
    ApprovalRequirementDefinition,
    ValueSubscription { path: String },
    DefaultSubscriptionSource,
    Reason,
    LeafPrototype,
}

impl SelectedEdgeWeightKind {
    /// All available edge weight kinds for selection
    pub const ALL: &'static [&'static str] = &[
        "Action",
        "ActionPrototype",
        "AuthenticationPrototype",
        "Contain",
        "DeprecatedFrameContains",
        "Ordering",
        "Ordinal",
        "Prop",
        "Prototype",
        "PrototypeArgument",
        "PrototypeArgumentValue",
        "Proxy",
        "Root",
        "Socket",
        "SocketValue",
        "Use",
        "ValidationOutput",
        "ManagementPrototype",
        "Represents",
        "Manages",
        "DiagramObject",
        "ApprovalRequirementDefinition",
        "ValueSubscription",
        "DefaultSubscriptionSource",
        "Reason",
        "LeafPrototype",
    ];

    /// Create from index and optional parameters
    pub fn from_index(index: usize, key: Option<String>, is_default: bool, path: String) -> Self {
        match index {
            0 => Self::Action,
            1 => Self::ActionPrototype,
            2 => Self::AuthenticationPrototype,
            3 => Self::Contain { key },
            4 => Self::DeprecatedFrameContains,
            5 => Self::Ordering,
            6 => Self::Ordinal,
            7 => Self::Prop,
            8 => Self::Prototype { key },
            9 => Self::PrototypeArgument,
            10 => Self::PrototypeArgumentValue,
            11 => Self::Proxy,
            12 => Self::Root,
            13 => Self::Socket,
            14 => Self::SocketValue,
            15 => Self::Use { is_default },
            16 => Self::ValidationOutput,
            17 => Self::ManagementPrototype,
            18 => Self::Represents,
            19 => Self::Manages,
            20 => Self::DiagramObject,
            21 => Self::ApprovalRequirementDefinition,
            22 => Self::ValueSubscription { path },
            23 => Self::DefaultSubscriptionSource,
            24 => Self::Reason,
            25 => Self::LeafPrototype,
            _ => Self::Action,
        }
    }

    /// Convert to EdgeWeightKind
    pub fn to_edge_weight_kind(&self) -> EdgeWeightKind {
        match self {
            Self::Action => EdgeWeightKind::Action,
            Self::ActionPrototype => EdgeWeightKind::ActionPrototype,
            Self::AuthenticationPrototype => EdgeWeightKind::AuthenticationPrototype,
            Self::Contain { key } => EdgeWeightKind::Contain(key.clone()),
            Self::DeprecatedFrameContains => EdgeWeightKind::DeprecatedFrameContains,
            Self::Ordering => EdgeWeightKind::Ordering,
            Self::Ordinal => EdgeWeightKind::Ordinal,
            Self::Prop => EdgeWeightKind::Prop,
            Self::Prototype { key } => EdgeWeightKind::Prototype(key.clone()),
            Self::PrototypeArgument => EdgeWeightKind::PrototypeArgument,
            Self::PrototypeArgumentValue => EdgeWeightKind::PrototypeArgumentValue,
            Self::Proxy => EdgeWeightKind::Proxy,
            Self::Root => EdgeWeightKind::Root,
            Self::Socket => EdgeWeightKind::Socket,
            Self::SocketValue => EdgeWeightKind::SocketValue,
            Self::Use { is_default } => EdgeWeightKind::Use {
                is_default: *is_default,
            },
            Self::ValidationOutput => EdgeWeightKind::ValidationOutput,
            Self::ManagementPrototype => EdgeWeightKind::ManagementPrototype,
            Self::Represents => EdgeWeightKind::Represents,
            Self::Manages => EdgeWeightKind::Manages,
            Self::DiagramObject => EdgeWeightKind::DiagramObject,
            Self::ApprovalRequirementDefinition => EdgeWeightKind::ApprovalRequirementDefinition,
            Self::ValueSubscription { path } => {
                EdgeWeightKind::ValueSubscription(AttributePath::JsonPointer(path.clone()))
            }
            Self::DefaultSubscriptionSource => EdgeWeightKind::DefaultSubscriptionSource,
            Self::Reason => EdgeWeightKind::Reason,
            Self::LeafPrototype => EdgeWeightKind::LeafPrototype,
        }
    }

    /// Returns true if this kind needs a key parameter
    pub fn needs_key(&self) -> bool {
        matches!(self, Self::Contain { .. } | Self::Prototype { .. })
    }

    /// Returns true if this kind needs the is_default parameter
    pub fn needs_is_default(&self) -> bool {
        matches!(self, Self::Use { .. })
    }

    /// Returns true if this kind needs a path parameter
    pub fn needs_path(&self) -> bool {
        matches!(self, Self::ValueSubscription { .. })
    }
}

/// Which field is currently focused in the add edge modal
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum AddEdgeField {
    #[default]
    SourceNodeId,
    TargetNodeId,
    EdgeKind,
    /// Optional key field for Contain/Prototype edges
    Key,
    /// is_default field for Use edges
    IsDefault,
    /// Path field for ValueSubscription edges
    Path,
}

/// State for the add edge modal
#[derive(Debug, Clone, Default)]
pub struct AddEdgeState {
    /// Source node ID input
    pub source_node_id: String,
    /// Target node ID input
    pub target_node_id: String,
    /// Selected edge weight kind index
    pub edge_kind_index: usize,
    /// Optional key for Contain/Prototype edges
    pub key: String,
    /// is_default for Use edges
    pub is_default: bool,
    /// Path for ValueSubscription edges
    pub path: String,
    /// Currently focused field
    pub focused_field: AddEdgeField,
    /// Whether currently editing a text field
    pub editing: bool,
    /// Autocomplete suggestions for source (indices into node_list)
    pub source_suggestions: Vec<usize>,
    /// Currently selected source suggestion index
    pub source_suggestion_index: usize,
    /// Autocomplete suggestions for target (indices into node_list)
    pub target_suggestions: Vec<usize>,
    /// Currently selected target suggestion index
    pub target_suggestion_index: usize,
    /// Whether the source node ID is valid (exists in node list)
    pub source_valid: bool,
    /// Whether the target node ID is valid (exists in node list)
    pub target_valid: bool,
    /// Error message to display
    pub error_message: Option<String>,
}

/// Represents an editable field on a node weight
#[derive(Debug, Clone)]
pub struct EditableField {
    pub name: String,
    pub value: String,
    pub field_type: EditableFieldType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EditableFieldType {
    String,
    Bool,
    /// Enum with a list of possible values and the currently selected index
    Enum {
        options: Vec<String>,
        selected_index: usize,
    },
}

/// Typed representation of an edit operation that can be re-executed
#[derive(Debug, Clone)]
pub enum GraphEdit {
    // Common edit that applies to multiple node types
    ContentHash {
        node_weight_id: Ulid,
        old: ContentHash,
        new: ContentHash,
    },

    // Prop edits
    PropName {
        node_weight_id: Ulid,
        old: String,
        new: String,
    },
    PropKind {
        node_weight_id: Ulid,
        old: PropKind,
        new: PropKind,
    },
    PropCanBeUsedAsPrototypeArg {
        node_weight_id: Ulid,
        old: bool,
        new: bool,
    },

    // Func edits
    FuncName {
        node_weight_id: Ulid,
        old: String,
        new: String,
    },
    FuncKind {
        node_weight_id: Ulid,
        old: FuncKind,
        new: FuncKind,
    },

    // FuncArgument edits
    FuncArgumentName {
        node_weight_id: Ulid,
        old: String,
        new: String,
    },

    // ActionPrototype edits
    ActionPrototypeName {
        node_weight_id: Ulid,
        old: String,
        new: String,
    },
    ActionPrototypeDescription {
        node_weight_id: Ulid,
        old: Option<String>,
        new: Option<String>,
    },
    ActionPrototypeKind {
        node_weight_id: Ulid,
        old: ActionKind,
        new: ActionKind,
    },

    // Component edits
    ComponentToDelete {
        node_weight_id: Ulid,
        old: bool,
        new: bool,
    },

    // Action edits
    ActionState {
        node_weight_id: Ulid,
        old: ActionState,
        new: ActionState,
    },
    ActionOriginatingChangeSetId {
        node_weight_id: Ulid,
        old: ChangeSetId,
        new: ChangeSetId,
    },

    // Category edits
    CategoryKind {
        node_weight_id: Ulid,
        old: CategoryNodeKind,
        new: CategoryNodeKind,
    },

    // DependentValueRoot edits
    DependentValueRootValueId {
        node_weight_id: Ulid,
        old: Ulid,
        new: Ulid,
    },

    // FinishedDependentValueRoot edits
    FinishedDependentValueRootValueId {
        node_weight_id: Ulid,
        old: Ulid,
        new: Ulid,
    },

    // Content edits
    ContentToDelete {
        node_weight_id: Ulid,
        old: bool,
        new: bool,
    },

    // Ordering edits
    OrderingOrder {
        node_weight_id: Ulid,
        old: Vec<Ulid>,
        new: Vec<Ulid>,
    },

    // Node deletion (stores the deleted node for undo)
    DeleteNode {
        node_weight_id: Ulid,
        deleted_weight: Box<NodeWeight>,
    },

    // Node addition (stores the added node for undo)
    AddNode {
        node_weight_id: Ulid,
        added_weight: Box<NodeWeight>,
    },

    // Edge deletion (stores the deleted edge for undo)
    DeleteEdge {
        source_node_id: Ulid,
        target_node_id: Ulid,
        edge_weight: EdgeWeight,
    },

    // Edge addition (stores the added edge for undo)
    AddEdge {
        source_node_id: Ulid,
        target_node_id: Ulid,
        edge_weight: EdgeWeight,
    },
}

impl GraphEdit {
    /// Returns the field name for display purposes
    pub fn field_name(&self) -> &'static str {
        match self {
            GraphEdit::ContentHash { .. } => "Content Hash",
            GraphEdit::PropName { .. } => "Name",
            GraphEdit::PropKind { .. } => "Kind",
            GraphEdit::PropCanBeUsedAsPrototypeArg { .. } => "Can be used as prototype arg",
            GraphEdit::FuncName { .. } => "Name",
            GraphEdit::FuncKind { .. } => "Kind",
            GraphEdit::FuncArgumentName { .. } => "Name",
            GraphEdit::ActionPrototypeName { .. } => "Name",
            GraphEdit::ActionPrototypeDescription { .. } => "Description",
            GraphEdit::ActionPrototypeKind { .. } => "Kind",
            GraphEdit::ComponentToDelete { .. } => "To Delete",
            GraphEdit::ActionState { .. } => "State",
            GraphEdit::ActionOriginatingChangeSetId { .. } => "Originating Change Set ID",
            GraphEdit::CategoryKind { .. } => "Kind",
            GraphEdit::DependentValueRootValueId { .. } => "Value ID",
            GraphEdit::FinishedDependentValueRootValueId { .. } => "Value ID",
            GraphEdit::ContentToDelete { .. } => "To Delete",
            GraphEdit::OrderingOrder { .. } => "Order",
            GraphEdit::DeleteNode { .. } => "Node",
            GraphEdit::AddNode { .. } => "Node",
            GraphEdit::DeleteEdge { .. } => "Edge",
            GraphEdit::AddEdge { .. } => "Edge",
        }
    }

    /// Returns a reversed edit (swaps old and new values) for undo functionality
    pub fn reverse(&self) -> Self {
        match self {
            GraphEdit::ContentHash {
                node_weight_id,
                old,
                new,
            } => GraphEdit::ContentHash {
                node_weight_id: *node_weight_id,
                old: *new,
                new: *old,
            },
            GraphEdit::PropName {
                node_weight_id,
                old,
                new,
            } => GraphEdit::PropName {
                node_weight_id: *node_weight_id,
                old: new.clone(),
                new: old.clone(),
            },
            GraphEdit::PropKind {
                node_weight_id,
                old,
                new,
            } => GraphEdit::PropKind {
                node_weight_id: *node_weight_id,
                old: *new,
                new: *old,
            },
            GraphEdit::PropCanBeUsedAsPrototypeArg {
                node_weight_id,
                old,
                new,
            } => GraphEdit::PropCanBeUsedAsPrototypeArg {
                node_weight_id: *node_weight_id,
                old: *new,
                new: *old,
            },
            GraphEdit::FuncName {
                node_weight_id,
                old,
                new,
            } => GraphEdit::FuncName {
                node_weight_id: *node_weight_id,
                old: new.clone(),
                new: old.clone(),
            },
            GraphEdit::FuncKind {
                node_weight_id,
                old,
                new,
            } => GraphEdit::FuncKind {
                node_weight_id: *node_weight_id,
                old: *new,
                new: *old,
            },
            GraphEdit::FuncArgumentName {
                node_weight_id,
                old,
                new,
            } => GraphEdit::FuncArgumentName {
                node_weight_id: *node_weight_id,
                old: new.clone(),
                new: old.clone(),
            },
            GraphEdit::ActionPrototypeName {
                node_weight_id,
                old,
                new,
            } => GraphEdit::ActionPrototypeName {
                node_weight_id: *node_weight_id,
                old: new.clone(),
                new: old.clone(),
            },
            GraphEdit::ActionPrototypeDescription {
                node_weight_id,
                old,
                new,
            } => GraphEdit::ActionPrototypeDescription {
                node_weight_id: *node_weight_id,
                old: new.clone(),
                new: old.clone(),
            },
            GraphEdit::ActionPrototypeKind {
                node_weight_id,
                old,
                new,
            } => GraphEdit::ActionPrototypeKind {
                node_weight_id: *node_weight_id,
                old: *new,
                new: *old,
            },
            GraphEdit::ComponentToDelete {
                node_weight_id,
                old,
                new,
            } => GraphEdit::ComponentToDelete {
                node_weight_id: *node_weight_id,
                old: *new,
                new: *old,
            },
            GraphEdit::ActionState {
                node_weight_id,
                old,
                new,
            } => GraphEdit::ActionState {
                node_weight_id: *node_weight_id,
                old: *new,
                new: *old,
            },
            GraphEdit::ActionOriginatingChangeSetId {
                node_weight_id,
                old,
                new,
            } => GraphEdit::ActionOriginatingChangeSetId {
                node_weight_id: *node_weight_id,
                old: *new,
                new: *old,
            },
            GraphEdit::CategoryKind {
                node_weight_id,
                old,
                new,
            } => GraphEdit::CategoryKind {
                node_weight_id: *node_weight_id,
                old: *new,
                new: *old,
            },
            GraphEdit::DependentValueRootValueId {
                node_weight_id,
                old,
                new,
            } => GraphEdit::DependentValueRootValueId {
                node_weight_id: *node_weight_id,
                old: *new,
                new: *old,
            },
            GraphEdit::FinishedDependentValueRootValueId {
                node_weight_id,
                old,
                new,
            } => GraphEdit::FinishedDependentValueRootValueId {
                node_weight_id: *node_weight_id,
                old: *new,
                new: *old,
            },
            GraphEdit::ContentToDelete {
                node_weight_id,
                old,
                new,
            } => GraphEdit::ContentToDelete {
                node_weight_id: *node_weight_id,
                old: *new,
                new: *old,
            },
            GraphEdit::OrderingOrder {
                node_weight_id,
                old,
                new,
            } => GraphEdit::OrderingOrder {
                node_weight_id: *node_weight_id,
                old: new.clone(),
                new: old.clone(),
            },
            // DeleteNode reverses to AddNode - undoing deletion restores the node
            GraphEdit::DeleteNode {
                node_weight_id,
                deleted_weight,
            } => GraphEdit::AddNode {
                node_weight_id: *node_weight_id,
                added_weight: deleted_weight.clone(),
            },
            // AddNode reverses to DeleteNode - undoing addition removes the node
            GraphEdit::AddNode {
                node_weight_id,
                added_weight,
            } => GraphEdit::DeleteNode {
                node_weight_id: *node_weight_id,
                deleted_weight: added_weight.clone(),
            },
            // DeleteEdge reverses to AddEdge - undoing deletion restores the edge
            GraphEdit::DeleteEdge {
                source_node_id,
                target_node_id,
                edge_weight,
            } => GraphEdit::AddEdge {
                source_node_id: *source_node_id,
                target_node_id: *target_node_id,
                edge_weight: edge_weight.clone(),
            },
            // AddEdge reverses to DeleteEdge - undoing addition removes the edge
            GraphEdit::AddEdge {
                source_node_id,
                target_node_id,
                edge_weight,
            } => GraphEdit::DeleteEdge {
                source_node_id: *source_node_id,
                target_node_id: *target_node_id,
                edge_weight: edge_weight.clone(),
            },
        }
    }

    /// Returns formatted old and new values for display
    pub fn display_values(&self) -> (String, String) {
        match self {
            GraphEdit::ContentHash { old, new, .. } => (old.to_string(), new.to_string()),
            GraphEdit::PropName { old, new, .. } => (old.clone(), new.clone()),
            GraphEdit::PropKind { old, new, .. } => (format!("{old:?}"), format!("{new:?}")),
            GraphEdit::PropCanBeUsedAsPrototypeArg { old, new, .. } => {
                (old.to_string(), new.to_string())
            }
            GraphEdit::FuncName { old, new, .. } => (old.clone(), new.clone()),
            GraphEdit::FuncKind { old, new, .. } => (format!("{old:?}"), format!("{new:?}")),
            GraphEdit::FuncArgumentName { old, new, .. } => (old.clone(), new.clone()),
            GraphEdit::ActionPrototypeName { old, new, .. } => (old.clone(), new.clone()),
            GraphEdit::ActionPrototypeDescription { old, new, .. } => (
                old.clone().unwrap_or_default(),
                new.clone().unwrap_or_default(),
            ),
            GraphEdit::ActionPrototypeKind { old, new, .. } => {
                (format!("{old:?}"), format!("{new:?}"))
            }
            GraphEdit::ComponentToDelete { old, new, .. } => (old.to_string(), new.to_string()),
            GraphEdit::ActionState { old, new, .. } => (format!("{old:?}"), format!("{new:?}")),
            GraphEdit::ActionOriginatingChangeSetId { old, new, .. } => {
                (old.to_string(), new.to_string())
            }
            GraphEdit::CategoryKind { old, new, .. } => (format!("{old:?}"), format!("{new:?}")),
            GraphEdit::DependentValueRootValueId { old, new, .. } => {
                (old.to_string(), new.to_string())
            }
            GraphEdit::FinishedDependentValueRootValueId { old, new, .. } => {
                (old.to_string(), new.to_string())
            }
            GraphEdit::ContentToDelete { old, new, .. } => (old.to_string(), new.to_string()),
            GraphEdit::OrderingOrder { old, new, .. } => {
                let old_str = old
                    .iter()
                    .map(|u| u.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                let new_str = new
                    .iter()
                    .map(|u| u.to_string())
                    .collect::<Vec<_>>()
                    .join(",");
                (old_str, new_str)
            }
            GraphEdit::DeleteNode { deleted_weight, .. } => {
                // Get the node weight kind
                let node_kind = match &**deleted_weight {
                    NodeWeight::Prop(_) => "Prop",
                    NodeWeight::Func(_) => "Func",
                    NodeWeight::FuncArgument(_) => "FuncArgument",
                    NodeWeight::ActionPrototype(_) => "ActionPrototype",
                    NodeWeight::Component(_) => "Component",
                    NodeWeight::Action(_) => "Action",
                    NodeWeight::Category(_) => "Category",
                    NodeWeight::AttributePrototypeArgument(_) => "AttributePrototypeArgument",
                    NodeWeight::AttributeValue(_) => "AttributeValue",
                    NodeWeight::DependentValueRoot(_) => "DependentValueRoot",
                    NodeWeight::FinishedDependentValueRoot(_) => "FinishedDependentValueRoot",
                    NodeWeight::Content(_) => "Content",
                    NodeWeight::Secret(_) => "Secret",
                    NodeWeight::Ordering(_) => "Ordering",
                    NodeWeight::InputSocket(_) => "InputSocket",
                    NodeWeight::SchemaVariant(_) => "SchemaVariant",
                    NodeWeight::ManagementPrototype(_) => "ManagementPrototype",
                    NodeWeight::Geometry(_) => "Geometry",
                    NodeWeight::View(_) => "View",
                    NodeWeight::DiagramObject(_) => "DiagramObject",
                    NodeWeight::ApprovalRequirementDefinition(_) => "ApprovalRequirementDefinition",
                    NodeWeight::Reason(_) => "Reason",
                    NodeWeight::LeafPrototype(_) => "LeafPrototype",
                };

                // Extract the name if available
                let name = match &**deleted_weight {
                    NodeWeight::Prop(p) => Some(p.name().to_string()),
                    NodeWeight::Func(f) => Some(f.name().to_string()),
                    NodeWeight::FuncArgument(fa) => Some(fa.name().to_string()),
                    NodeWeight::ActionPrototype(ap) => Some(ap.name().to_string()),
                    NodeWeight::Category(c) => Some(format!("{:?}", c.kind())),
                    _ => None,
                };

                let old_value = if let Some(name) = name {
                    format!("{} '{}' ({})", node_kind, name, deleted_weight.id())
                } else {
                    format!("{} ({})", node_kind, deleted_weight.id())
                };

                (old_value, "(deleted)".to_string())
            }
            GraphEdit::AddNode { added_weight, .. } => {
                // Get the node weight kind
                let node_kind = match &**added_weight {
                    NodeWeight::Prop(_) => "Prop",
                    NodeWeight::Func(_) => "Func",
                    NodeWeight::FuncArgument(_) => "FuncArgument",
                    NodeWeight::ActionPrototype(_) => "ActionPrototype",
                    NodeWeight::Component(_) => "Component",
                    NodeWeight::Action(_) => "Action",
                    NodeWeight::Category(_) => "Category",
                    NodeWeight::AttributePrototypeArgument(_) => "AttributePrototypeArgument",
                    NodeWeight::AttributeValue(_) => "AttributeValue",
                    NodeWeight::DependentValueRoot(_) => "DependentValueRoot",
                    NodeWeight::FinishedDependentValueRoot(_) => "FinishedDependentValueRoot",
                    NodeWeight::Content(_) => "Content",
                    NodeWeight::Secret(_) => "Secret",
                    NodeWeight::Ordering(_) => "Ordering",
                    NodeWeight::InputSocket(_) => "InputSocket",
                    NodeWeight::SchemaVariant(_) => "SchemaVariant",
                    NodeWeight::ManagementPrototype(_) => "ManagementPrototype",
                    NodeWeight::Geometry(_) => "Geometry",
                    NodeWeight::View(_) => "View",
                    NodeWeight::DiagramObject(_) => "DiagramObject",
                    NodeWeight::ApprovalRequirementDefinition(_) => "ApprovalRequirementDefinition",
                    NodeWeight::Reason(_) => "Reason",
                    NodeWeight::LeafPrototype(_) => "LeafPrototype",
                };

                // Extract the name if available
                let name = match &**added_weight {
                    NodeWeight::Prop(p) => Some(p.name().to_string()),
                    NodeWeight::Func(f) => Some(f.name().to_string()),
                    NodeWeight::FuncArgument(fa) => Some(fa.name().to_string()),
                    NodeWeight::ActionPrototype(ap) => Some(ap.name().to_string()),
                    NodeWeight::Category(c) => Some(format!("{:?}", c.kind())),
                    _ => None,
                };

                let new_value = if let Some(name) = name {
                    format!("{} '{}' ({})", node_kind, name, added_weight.id())
                } else {
                    format!("{} ({})", node_kind, added_weight.id())
                };

                ("(added)".to_string(), new_value)
            }
            GraphEdit::DeleteEdge {
                source_node_id,
                target_node_id,
                edge_weight,
            } => (
                format!("{:?}", edge_weight.kind()),
                format!("{source_node_id} → {target_node_id}"),
            ),
            GraphEdit::AddEdge {
                source_node_id,
                target_node_id,
                edge_weight,
            } => (
                format!("{:?}", edge_weight.kind()),
                format!("{source_node_id} → {target_node_id}"),
            ),
        }
    }

    /// Returns the node weight kind this edit applies to
    pub fn node_kind(&self) -> &'static str {
        match self {
            GraphEdit::ContentHash { .. } => "Multiple",
            GraphEdit::PropName { .. }
            | GraphEdit::PropKind { .. }
            | GraphEdit::PropCanBeUsedAsPrototypeArg { .. } => "Prop",
            GraphEdit::FuncName { .. } | GraphEdit::FuncKind { .. } => "Func",
            GraphEdit::FuncArgumentName { .. } => "FuncArgument",
            GraphEdit::ActionPrototypeName { .. }
            | GraphEdit::ActionPrototypeDescription { .. }
            | GraphEdit::ActionPrototypeKind { .. } => "ActionPrototype",
            GraphEdit::ComponentToDelete { .. } => "Component",
            GraphEdit::ActionState { .. } | GraphEdit::ActionOriginatingChangeSetId { .. } => {
                "Action"
            }
            GraphEdit::CategoryKind { .. } => "Category",
            GraphEdit::DependentValueRootValueId { .. } => "DependentValueRoot",
            GraphEdit::FinishedDependentValueRootValueId { .. } => "FinishedDependentValueRoot",
            GraphEdit::ContentToDelete { .. } => "Content",
            GraphEdit::OrderingOrder { .. } => "Ordering",
            GraphEdit::DeleteNode { .. } => "Node",
            GraphEdit::AddNode { .. } => "Node",
            GraphEdit::DeleteEdge { .. } => "Edge",
            GraphEdit::AddEdge { .. } => "Edge",
        }
    }

    /// Returns the operation type for display
    pub fn operation_type(&self) -> &'static str {
        match self {
            GraphEdit::DeleteNode { .. } | GraphEdit::DeleteEdge { .. } => "Delete",
            GraphEdit::AddNode { .. } | GraphEdit::AddEdge { .. } => "Add",
            _ => "Edit",
        }
    }
}

/// Represents an edit that has been made to a node weight
#[derive(Debug, Clone)]
pub struct PendingEdit {
    pub node_id: Ulid,
    pub edit: GraphEdit,
}

/// State for the node list panel
#[derive(Debug, Clone)]
pub struct NodeListState {
    /// Currently selected node index in the list
    pub selected_index: usize,
    /// Scroll offset for the node list
    pub scroll_offset: usize,
    /// Cached list of all nodes for display
    pub node_list: Vec<NodeListItem>,
    /// Filtered list of nodes based on current filter
    pub filtered_node_list: Vec<NodeListItem>,
    /// Current filter text
    pub filter_text: String,
    /// Whether we're in filter input mode
    pub filter_mode: bool,
}

impl NodeListState {
    pub fn new(node_list: Vec<NodeListItem>) -> Self {
        let filtered_node_list = node_list.clone();
        Self {
            selected_index: 0,
            scroll_offset: 0,
            node_list,
            filtered_node_list,
            filter_text: String::new(),
            filter_mode: false,
        }
    }

    pub fn update_filter(&mut self) {
        if self.filter_text.is_empty() {
            self.filtered_node_list = self.node_list.clone();
        } else {
            let filter_lower = self.filter_text.to_lowercase();
            self.filtered_node_list = self
                .node_list
                .iter()
                .filter(|item| {
                    // Filter by node weight kind, ID, or name
                    item.node_weight_kind.to_lowercase().contains(&filter_lower)
                        || item
                            .node_id
                            .to_string()
                            .to_lowercase()
                            .contains(&filter_lower)
                        || item
                            .name
                            .as_ref()
                            .is_some_and(|n| n.to_lowercase().contains(&filter_lower))
                })
                .cloned()
                .collect();
        }

        // Reset selection if out of bounds
        if self.selected_index >= self.filtered_node_list.len()
            && !self.filtered_node_list.is_empty()
        {
            self.selected_index = self.filtered_node_list.len() - 1;
        }

        // Reset scroll offset if out of bounds
        if self.scroll_offset >= self.filtered_node_list.len() {
            self.scroll_offset = 0;
        }
    }
}

/// State for the node details panel
#[derive(Debug, Clone, Default)]
pub struct NodeDetailsState {
    /// Scroll offset for the node details panel
    pub scroll_offset: u16,
}

/// State for the edge panel
#[derive(Debug, Clone, Default)]
pub struct EdgePanelState {
    /// Currently selected edge index within the edge panel
    pub selected_edge: usize,
    /// Scroll offset for the edge list
    pub scroll_offset: usize,
}

/// State for the edit history panel
#[derive(Debug, Clone, Default)]
pub struct EditHistoryState {
    /// Scroll offset for the edit history panel
    pub scroll_offset: u16,
}

/// State for the edit node modal
#[derive(Debug, Clone, Default)]
pub struct EditModalState {
    /// Index of the currently selected field in the edit modal
    pub field_index: usize,
    /// List of editable fields for the current node
    pub editable_fields: Vec<EditableField>,
    /// Current edit input buffer (when editing a field)
    pub input: String,
    /// Whether we're currently editing a field (typing mode)
    pub editing: bool,
}

pub struct SaveModalState {
    /// The filename/path being edited
    pub filename: String,
    /// Whether we're editing the filename
    pub editing: bool,
}

impl Default for SaveModalState {
    fn default() -> Self {
        Self {
            filename: String::from("workspace_snapshot.modified.bin"),
            editing: true,
        }
    }
}

pub struct AppState {
    /// The current working copy of the graph
    pub working_graph: WorkspaceSnapshotGraph,

    /// Track whether the graph has been modified
    pub is_dirty: bool,

    /// The original graph loaded from file (for comparison)
    pub original_graph: WorkspaceSnapshotGraph,

    /// Whether the application should quit
    pub should_quit: bool,

    /// Current panel that has focus
    pub focus: FocusPanel,

    /// Currently active modal dialog (if any)
    pub active_modal: Option<ActiveModal>,

    /// Stack of edits for undo functionality
    pub pending_edits: Vec<PendingEdit>,

    /// State for the node list panel
    pub node_list: NodeListState,

    /// State for the node details panel
    pub details: NodeDetailsState,

    /// State for the edge panel
    pub edge_panel: EdgePanelState,

    /// State for the edit history panel
    pub edit_history: EditHistoryState,

    /// State for the edit modal
    pub edit_modal: EditModalState,

    /// State for the add edge modal
    pub add_edge_modal: AddEdgeState,

    /// State for the save modal
    pub save_modal: SaveModalState,

    /// Path to save the snapshot file
    pub save_path: Option<std::path::PathBuf>,

    /// Temporary success message to display (e.g., after save)
    pub success_message: Option<String>,

    /// Snapshot statistics (byte sizes and counts per node/edge kind)
    pub stats: SnapshotStats,

    /// Scroll offset for the stats modal
    pub stats_scroll: u16,

    /// Current frame size (updated before each event handling)
    pub frame_size: Rect,
}

#[derive(Debug, Clone)]
pub struct NodeListItem {
    pub index: NodeIndex,
    pub node_id: Ulid,
    pub node_weight_kind: String,
    pub name: Option<String>,
}

impl AppState {
    pub fn new(
        graph: WorkspaceSnapshotGraph,
        node_list: Vec<NodeListItem>,
        save_path: Option<std::path::PathBuf>,
        stats: SnapshotStats,
    ) -> Self {
        Self {
            working_graph: graph.clone(),
            is_dirty: false,
            original_graph: graph,
            should_quit: false,
            focus: FocusPanel::NodeList,
            active_modal: None,
            pending_edits: Vec::new(),
            node_list: NodeListState::new(node_list),
            details: NodeDetailsState::default(),
            edge_panel: EdgePanelState::default(),
            edit_history: EditHistoryState::default(),
            edit_modal: EditModalState::default(),
            add_edge_modal: AddEdgeState::default(),
            save_modal: SaveModalState::default(),
            save_path,
            success_message: None,
            stats,
            stats_scroll: 0,
            frame_size: Rect::default(),
        }
    }

    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            FocusPanel::NodeList => FocusPanel::NodeDetails,
            FocusPanel::NodeDetails => FocusPanel::EdgePanel,
            FocusPanel::EdgePanel => FocusPanel::EditHistory,
            FocusPanel::EditHistory => FocusPanel::NodeList,
        };
    }

    /// Mark the working graph as dirty (modified from original)
    pub fn mark_dirty(&mut self) {
        self.is_dirty = true;
    }
}
