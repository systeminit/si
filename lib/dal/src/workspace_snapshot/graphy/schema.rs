use crate::workspace_snapshot::node_weight::{NodeWeight, PropNodeWeight};
use crate::{EdgeWeightKind, EdgeWeightKindDiscriminants, PropId, SchemaId, SchemaVariantId};
use super::*;
use super::super::{content_address::ContentAddressDiscriminants, node_weight::{category_node_weight::CategoryNodeKind, ContentNodeWeight}};

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef)]
pub struct Schema<'a>(pub(super) GraphyNode<'a>);

impl<'a> GraphyNodeType<'a> for Schema<'a> {
    type Id = SchemaId;
    type Weight = ContentNodeWeight;
    fn node_kind() -> NodeWeightDiscriminants { NodeWeightDiscriminants::Content }
    fn construct(node: GraphyNode<'a>) -> Self { Self(node) }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        Self::content_weight_as(weight)
    }
}

impl<'a> GraphyContentNodeType<'a> for Schema<'a> {
    fn content_kind() -> ContentAddressDiscriminants { ContentAddressDiscriminants::Schema }
}

impl<'a> GraphyCategoryNodeType<'a> for Schema<'a> {
    fn category_kind() -> CategoryNodeKind { CategoryNodeKind::Schema }
}

impl<'a> Schema<'a> {
    //
    // Children
    //
    pub fn variants(self) -> impl Iterator<Item = SchemaVariant<'a>> {
        self.0.target_nodes(EdgeWeightKindDiscriminants::Use).map(SchemaVariant)
    }

    //
    // Properties
    //
    pub fn authentication_prototypes(self) -> impl Iterator<Item = Func<'a>> {
        self.0.target_nodes(EdgeWeightKindDiscriminants::AuthenticationPrototype).map(Func)
    }

    //
    // Backreferences
    //
    pub fn category(self) -> GraphyResult<Category<'a, Module<'a>>> {
        self.0.matching_source(EdgeWeightKindDiscriminants::Use)
    }
    // TODO is this an alternate parent or can it be in both here and the category?
    pub fn module(self) -> GraphyResult<Option<Module<'a>>> {
        self.0.matching_source_opt(EdgeWeightKindDiscriminants::Use)
    }
}

impl<'a> TryFrom<GraphyNode<'a>> for Schema<'a> {
    type Error = GraphyError;
    fn try_from(node: GraphyNode<'a>) -> Result<Self, Self::Error> {
        let result = Self(node);
        result.weight()?;
        Ok(result)
    }
}

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef)]
pub struct SchemaVariant<'a>(pub(super) GraphyNode<'a>);

impl<'a> GraphyNodeType<'a> for SchemaVariant<'a> {
    type Id = SchemaVariantId;
    type Weight = ContentNodeWeight;
    fn node_kind() -> NodeWeightDiscriminants { NodeWeightDiscriminants::Content }
    fn construct(node: GraphyNode<'a>) -> Self { Self(node) }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        Self::content_weight_as(weight)
    }
}

impl<'a> GraphyContentNodeType<'a> for SchemaVariant<'a> {
    fn content_kind() -> ContentAddressDiscriminants { ContentAddressDiscriminants::SchemaVariant }
}

impl<'a> SchemaVariant<'a> {
    //
    // Backreferences
    //
    pub fn schema(self) -> GraphyResult<Schema<'a>> {
        self.0.matching_source(EdgeWeightKindDiscriminants::Use)
    }
    pub fn components(self) -> impl Iterator<Item = Component<'a>> {
        self.0.matching_sources(EdgeWeightKindDiscriminants::Use)
    }
    // TODO is this an alternate parent or can it be in both here and the category?
    pub fn module(self) -> GraphyResult<Option<Module<'a>>> {
        self.0.matching_source_opt(EdgeWeightKindDiscriminants::Use)
    }
}

impl<'a> TryFrom<GraphyNode<'a>> for SchemaVariant<'a> {
    type Error = GraphyError;
    fn try_from(node: GraphyNode<'a>) -> Result<Self, Self::Error> {
        let result = Self(node);
        result.weight()?;
        Ok(result)
    }
}

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef)]
pub struct Prop<'a>(pub(super) GraphyNode<'a>);

pub enum PropParent<'a> {
    Prop(Prop<'a>),
    SchemaVariant(SchemaVariant<'a>),
}

impl<'a> GraphyNodeType<'a> for Prop<'a> {
    type Id = PropId;
    type Weight = PropNodeWeight;
    fn node_kind() -> NodeWeightDiscriminants { NodeWeightDiscriminants::Prop }
    fn construct(node: GraphyNode<'a>) -> Self { Self(node) }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::Prop(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a> Prop<'a> {
    //
    // Children
    //
    pub fn children(self) -> GraphyResult<OptionIter<impl Iterator<Item = Prop<'a>>>> {
        let ordering = self.ordering()?;
        if ordering.is_none() && self.0.outgoing_edges(EdgeWeightKindDiscriminants::Use).next().is_some() {
            return Err(GraphyError::MissingOrdering(self.0.index))
        }
        Ok(OptionIter(ordering.map(Ordering::children)))
    }

    pub fn unordered_children(self) -> impl Iterator<Item = Prop<'a>> {
        self.0
            .all_outgoing_edges()
            .filter_map(|e| match e.weight().kind() {
                // TODO when do we look at default, anyway?
                EdgeWeightKind::Use { .. } => Some(Prop(self.0.graph.node(e.target()))),
                _ => None,
            })
    }

    pub fn ordering(self) -> GraphyResult<Option<Ordering<'a, Self, Self>>> {
        Ok(self.0.target_node_opt(EdgeWeightKindDiscriminants::Ordering)?.map(Ordering::construct))
    }

    pub fn prototype(self) -> GraphyResult<AttributePrototype<'a>> {
        Ok(AttributePrototype(self.0.target_node(EdgeWeightKindDiscriminants::Prototype)?))
    }

    //
    // Backreferences
    //
    pub fn parent(self) -> GraphyResult<PropParent<'a>> {
        let parent = self.0.source_node(EdgeWeightKindDiscriminants::Use)?;
        match Prop::try_from(parent) {
            Ok(parent) => Ok(PropParent::Prop(parent)),
            Err(_) => Ok(PropParent::SchemaVariant(SchemaVariant(self.0))),
        }
    }

    pub fn parent_prop(self) -> GraphyResult<Option<Prop<'a>>> {
        Ok(self.0.source_node(EdgeWeightKindDiscriminants::Use)?.try_into().ok())
    }
}

impl<'a> TryFrom<GraphyNode<'a>> for Prop<'a> {
    type Error = GraphyError;
    fn try_from(node: GraphyNode<'a>) -> Result<Self, Self::Error> {
        let result = Self(node);
        result.weight()?;
        Ok(result)
    }
}

pub struct OptionIter<I: Iterator>(pub Option<I>);

impl<I: Iterator> Iterator for OptionIter<I> {
    type Item = I::Item;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.as_mut().and_then(I::next)
    }
}

