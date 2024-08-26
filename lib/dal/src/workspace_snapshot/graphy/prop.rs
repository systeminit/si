use si_events::ulid::Ulid;

use super::{
    AnyAttributePrototype, GraphyError, GraphyNode, GraphyNodeRef, GraphyResult, Ordering,
    GraphyItertools as _, SchemaVariant,
};
use crate::{
    workspace_snapshot::node_weight::{NodeWeight, PropNodeWeight},
    EdgeWeightKind, EdgeWeightKindDiscriminants, PropId,
};

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct Prop<'a>(GraphyNodeRef<'a>);

impl<'a> GraphyNode<'a> for Prop<'a> {
    type Id = PropId;
    type Weight = PropNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(node.into())
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::Prop(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a> Prop<'a> {
    /// Child props in order
    pub fn children(self) -> GraphyResult<impl Iterator<Item = GraphyResult<Prop<'a>>>> {
        let optional_children = match self.ordering_node()? {
            Some(ordering) => Some(ordering.children()?),
            // If there are Contains children, an ordering is presently required
            None => match self.unordered_children().next() {
                Some(_) => return Err(GraphyError::MissingOrdering(self.index)),
                None => None,
            },
        };
        Ok(optional_children.into_iter().flatten())
    }

    /// Default value for this prop
    pub fn default_value(self) -> GraphyResult<AnyAttributePrototype<'a>> {
        self.targets(EdgeWeightKindDiscriminants::Prototype)
            .single()
    }

    /// Child props in no particular order
    pub fn unordered_children(self) -> impl Iterator<Item = Prop<'a>> {
        self.targets(EdgeWeightKindDiscriminants::Use)
    }

    /// Order of child props
    pub fn ordering_node(self) -> GraphyResult<Option<Ordering<'a, Self>>> {
        self.targets(EdgeWeightKind::Ordering).optional()
    }

    //
    // Backreferences
    //
    pub fn parent(self) -> GraphyResult<PropParent<'a>> {
        self.sources(EdgeWeightKindDiscriminants::Use).single()
    }
    pub fn parent_prop(self) -> GraphyResult<Option<Prop<'a>>> {
        self.sources(EdgeWeightKindDiscriminants::Use).optional()
    }
    pub fn parent_ordering(self) -> GraphyResult<Option<Ordering<'a, Self>>> {
        self.sources(EdgeWeightKind::Ordering).optional()
    }
}

#[derive(Copy, Clone)]
pub enum PropParent<'a> {
    Prop(Prop<'a>),
    SchemaVariant(SchemaVariant<'a>),
}

impl<'a> GraphyNode<'a> for PropParent<'a> {
    type Id = Ulid;
    type Weight = NodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        GraphyNode::try_as_node(node)
            .map(PropParent::Prop)
            .unwrap_or_else(|_| PropParent::SchemaVariant(GraphyNode::as_node(node)))
    }
    fn try_as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> GraphyResult<Self> {
        GraphyNode::try_as_node(node)
            .map(PropParent::Prop)
            .or_else(|_| GraphyNode::try_as_node(node).map(PropParent::SchemaVariant))
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        Ok(weight)
    }
}

impl<'a> From<PropParent<'a>> for GraphyNodeRef<'a> {
    fn from(node: PropParent<'a>) -> Self {
        match node {
            PropParent::Prop(node) => node.into(),
            PropParent::SchemaVariant(node) => node.into(),
        }
    }
}

impl<'a> AsRef<GraphyNodeRef<'a>> for PropParent<'a> {
    fn as_ref(&self) -> &GraphyNodeRef<'a> {
        match self {
            PropParent::Prop(node) => node.as_ref(),
            PropParent::SchemaVariant(node) => node.as_ref(),
        }
    }
}
