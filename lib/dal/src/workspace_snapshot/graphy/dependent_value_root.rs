use super::{GraphyCategoryElement, GraphyError, GraphyNode, GraphyNodeRef, GraphyResult};
use crate::workspace_snapshot::node_weight::{
    category_node_weight::CategoryNodeKind, DependentValueRootNodeWeight, NodeWeight,
};
use si_events::ulid::Ulid;

///
/// Dependent value that is "dirty"--has been updated.
///
/// JS functions will be run to ensure any downstream values are changed.
///
#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct DependentValueRoot<'a>(GraphyNodeRef<'a>);

impl<'a> DependentValueRoot<'a> {}

impl<'a> GraphyNode<'a> for DependentValueRoot<'a> {
    type Id = Ulid;
    type Weight = DependentValueRootNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(node.into())
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::DependentValueRoot(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a> GraphyCategoryElement<'a> for DependentValueRoot<'a> {
    fn category_kind() -> CategoryNodeKind {
        CategoryNodeKind::DependentValueRoots
    }
}
