use serde::{
    Deserialize,
    Serialize,
};
use si_id::WorkspacePk;

use crate::{
    action::action_diff_list::ActionDiffView,
    component::{
        ComponentDiffStatus,
        component_diff::AttributeDiff,
    },
};

/// A comprehensive review optimized for Luminork change set view.
///
/// This aggregates and pre-processes data from ComponentList, ComponentDiff,
/// ActionDiffList, and ErasedComponents to provide a complete change set overview.
///
/// NOTE: This is NOT a traditional MV - it's built on-demand via the API endpoint
/// and is never part of the incremental MV index.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LuminorkChangeSetReview {
    /// The workspace ID (used as the MV ID)
    pub id: WorkspacePk,
    /// Components that have been added, modified, or removed in this change set
    pub components: Vec<ComponentReview>,
}

/// A component with all its change information pre-aggregated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentReview {
    /// Basic component information from ComponentInList
    #[serde(flatten)]
    pub component: crate::component::ComponentInList,

    /// Attribute diffs organized as separate trees per top-level attribute
    /// Only includes "interesting" diffs (filtered)
    pub attribute_diff_trees: Vec<AttributeDiffTree>,

    /// Action diffs for this component
    pub action_diffs: Vec<ActionDiffView>,

    /// Corrected diff status (accounts for action diffs and filtered attributes)
    pub corrected_diff_status: ComponentDiffStatus,
}

/// A flattened attribute diff with its full path
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AttributeDiffTree {
    /// The full path to this attribute (e.g., "/domain/Region")
    pub path: String,

    /// The diff for this attribute
    pub diff: AttributeDiff,
}
