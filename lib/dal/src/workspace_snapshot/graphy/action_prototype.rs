use super::{
    Func, GraphyError, GraphyNode, GraphyNodeRef, GraphyResult, GraphyItertools as _, SchemaVariant,
};
use crate::{
    workspace_snapshot::node_weight::{ActionPrototypeNodeWeight, NodeWeight},
    ActionPrototypeId, EdgeWeightKind, EdgeWeightKindDiscriminants,
};

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct ActionPrototype<'a>(GraphyNodeRef<'a>);

impl<'a> GraphyNode<'a> for ActionPrototype<'a> {
    type Id = ActionPrototypeId;
    type Weight = ActionPrototypeNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(node.into())
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::ActionPrototype(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a> ActionPrototype<'a> {
    //
    // Properties
    //
    pub fn function(self) -> GraphyResult<Func<'a>> {
        self.targets(EdgeWeightKindDiscriminants::Use).single()
    }

    //
    // Backreferences
    //
    pub fn parent(self) -> GraphyResult<SchemaVariant<'a>> {
        self.sources(EdgeWeightKind::ActionPrototype).single()
    }
}
