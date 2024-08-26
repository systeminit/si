use super::{
    ActionPrototype, Component, GraphyCategoryElement, GraphyError, GraphyNode, GraphyNodeRef,
    GraphyResult, GraphyItertools as _,
};
use crate::{
    action::ActionId,
    workspace_snapshot::node_weight::{
        category_node_weight::CategoryNodeKind, ActionNodeWeight, NodeWeight,
    },
    EdgeWeightKindDiscriminants,
};

///
/// Action to be taken. This represents intent to create/delete/refresh/update a resource.
///
#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct Action<'a>(GraphyNodeRef<'a>);

///
/// Action to be taken. This represents intent to create/delete/refresh/update a resource.
///
impl<'a> Action<'a> {
    /// Component to take action on. Will be used as the argument to the action function.
    pub fn component(self) -> GraphyResult<Component<'a>> {
        self.targets(EdgeWeightKindDiscriminants::Use).single()
    }

    /// Definition of the action
    pub fn action_prototype(self) -> GraphyResult<ActionPrototype<'a>> {
        self.targets(EdgeWeightKindDiscriminants::Use).single()
    }
}

impl<'a> GraphyNode<'a> for Action<'a> {
    type Id = ActionId;
    type Weight = ActionNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(node.into())
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::Action(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a> GraphyCategoryElement<'a> for Action<'a> {
    fn category_kind() -> CategoryNodeKind {
        CategoryNodeKind::Action
    }
}
