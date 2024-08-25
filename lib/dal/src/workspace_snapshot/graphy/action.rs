
use crate::action::ActionId;
use super::*;
use super::super::node_weight::{category_node_weight::CategoryNodeKind, ActionNodeWeight, NodeWeight};

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef)]
pub struct Action<'a>(pub(super) GraphyNode<'a>);

impl<'a> GraphyNodeType<'a> for Action<'a> {
    type Id = ActionId;
    type Weight = ActionNodeWeight;
    fn node_kind() -> NodeWeightDiscriminants { NodeWeightDiscriminants::Action }
    fn construct(node: GraphyNode<'a>) -> Self { Self(node) }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::Action(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}
impl<'a> GraphyCategoryNodeType<'a> for Action<'a> {
    fn category_kind() -> CategoryNodeKind { CategoryNodeKind::Action }
}

impl<'a> Action<'a> {
}
