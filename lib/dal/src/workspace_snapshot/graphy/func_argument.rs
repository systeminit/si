use super::{GraphyError, GraphyNode, GraphyNodeRef, GraphyResult};
use crate::{
    func::argument::FuncArgumentId,
    workspace_snapshot::node_weight::{FuncArgumentNodeWeight, NodeWeight},
};

///
/// Argument definition for a function.
///
#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct FuncArgument<'a>(GraphyNodeRef<'a>);

impl<'a> FuncArgument<'a> {}

impl<'a> GraphyNode<'a> for FuncArgument<'a> {
    type Id = FuncArgumentId;
    type Weight = FuncArgumentNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(node.into())
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::FuncArgument(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}
