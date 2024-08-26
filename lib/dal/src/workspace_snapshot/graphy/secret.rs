use super::{GraphyCategoryElement, GraphyError, GraphyNode, GraphyNodeRef, GraphyResult};
use crate::workspace_snapshot::node_weight::{
    category_node_weight::CategoryNodeKind, secret_node_weight::SecretNodeWeight, NodeWeight,
};
use si_events::ulid::Ulid;

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct Secret<'a>(GraphyNodeRef<'a>);

impl<'a> GraphyNode<'a> for Secret<'a> {
    type Id = Ulid;
    type Weight = SecretNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(node.into())
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::Secret(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a> GraphyCategoryElement<'a> for Secret<'a> {
    fn category_kind() -> CategoryNodeKind {
        CategoryNodeKind::Secret
    }
}

impl<'a> Secret<'a> {}
