use si_events::ulid::Ulid;
use super::*;
use super::super::node_weight::{category_node_weight::CategoryNodeKind, secret_node_weight::SecretNodeWeight, NodeWeight};

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef)]
pub struct Secret<'a>(pub(super) GraphyNode<'a>);

impl<'a> GraphyNodeType<'a> for Secret<'a> {
    type Id = Ulid;
    type Weight = SecretNodeWeight;
    fn node_kind() -> NodeWeightDiscriminants { NodeWeightDiscriminants::Secret }
    fn construct(node: GraphyNode<'a>) -> Self { Self(node) }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::Secret(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a> GraphyCategoryNodeType<'a> for Secret<'a> {
    fn category_kind() -> CategoryNodeKind { CategoryNodeKind::Secret }
}

impl<'a> Secret<'a> {
}
