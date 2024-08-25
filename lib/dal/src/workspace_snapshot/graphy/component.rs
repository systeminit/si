
use crate::ComponentId;
use super::*;
use super::super::node_weight::{category_node_weight::CategoryNodeKind, ComponentNodeWeight, NodeWeight};

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef)]
pub struct Component<'a>(pub(super) GraphyNode<'a>);

impl<'a> GraphyNodeType<'a> for Component<'a> {
    type Id = ComponentId;
    type Weight = ComponentNodeWeight;
    fn node_kind() -> NodeWeightDiscriminants { NodeWeightDiscriminants::Component }
    fn construct(node: GraphyNode<'a>) -> Self { Self(node) }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::Component(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a> GraphyCategoryNodeType<'a> for Component<'a> {
    fn category_kind() -> CategoryNodeKind { CategoryNodeKind::Component }
}

impl<'a> Component<'a> {
}

impl<'a> TryFrom<GraphyNode<'a>> for Component<'a> {
    type Error = GraphyError;
    fn try_from(node: GraphyNode<'a>) -> Result<Self, Self::Error> {
        let result = Self(node);
        result.weight()?;
        Ok(result)
    }
}
