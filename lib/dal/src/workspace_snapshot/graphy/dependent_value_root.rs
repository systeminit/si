use si_events::ulid::Ulid;
use super::*;
use super::super::node_weight::{category_node_weight::CategoryNodeKind, DependentValueRootNodeWeight, NodeWeight};

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef)]
pub struct DependentValueRoot<'a>(pub(super) GraphyNode<'a>);

impl<'a> GraphyNodeType<'a> for DependentValueRoot<'a> {
    type Id = Ulid;
    type Weight = DependentValueRootNodeWeight;
    fn node_kind() -> NodeWeightDiscriminants { NodeWeightDiscriminants::DependentValueRoot }
    fn construct(node: GraphyNode<'a>) -> Self { Self(node) }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::DependentValueRoot(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a> GraphyCategoryNodeType<'a> for DependentValueRoot<'a> {
    fn category_kind() -> CategoryNodeKind { CategoryNodeKind::DependentValueRoots }
}

impl<'a> DependentValueRoot<'a> {
}
