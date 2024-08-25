use crate::workspace_snapshot::node_weight::{NodeWeight, PropNodeWeight};
use crate::{AttributePrototypeId, EdgeWeightKind, EdgeWeightKindDiscriminants, PropId, SchemaId, SchemaVariantId};
use super::*;
use super::super::{content_address::ContentAddressDiscriminants, node_weight::{category_node_weight::CategoryNodeKind, ContentNodeWeight}};

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef)]
pub struct AttributeValue<'a>(pub(super) GraphyNode<'a>);

pub enum AttributeValueParent<'a> {
    AttributeValue(AttributeValue<'a>),
    SchemaVariant(SchemaVariant<'a>),
}

impl<'a> GraphyNodeType<'a> for AttributeValue<'a> {
    type Id = AttributeValueId;
    type Weight = AttributeValueNodeWeight;
    fn node_kind() -> NodeWeightDiscriminants { NodeWeightDiscriminants::AttributeValue }
    fn construct(node: GraphyNode<'a>) -> Self { Self(node) }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        match weight {
            NodeWeight::AttributeValue(weight) => Ok(weight),
            weight => Err(GraphyError::WrongNodeType(weight.into())),
        }
    }
}

impl<'a> AttributeValue<'a> {
    //
    // Children
    //
    pub fn children(self) -> GraphyResult<OptionIter<impl Iterator<Item = AttributeValue<'a>>>> {
        let ordering = self.ordering()?;
        if ordering.is_none() && self.0.outgoing_edges(EdgeWeightKindDiscriminants::Contain).next().is_some() {
            return Err(GraphyError::MissingOrdering(self.0.index))
        }
        Ok(OptionIter(ordering.map(Ordering::children)))
    }

    pub fn unordered_children_with_keys(self) -> impl Iterator<Item = (Option<&'a String>, AttributeValue<'a>)> {
        self.0
            .all_outgoing_edges()
            .filter_map(|e| match e.weight().kind() {
                EdgeWeightKind::Contain(key) => Some((
                    key.as_ref(),
                    AttributeValue(self.0.graph.node(e.target()))
                )),
                _ => None,
            })
    }

    pub fn ordering(self) -> GraphyResult<Option<Ordering<'a, Self, Self>>> {
        Ok(self.0.target_node_opt(EdgeWeightKindDiscriminants::Ordering)?.map(Ordering::construct))
    }

    //
    // Backreferences
    //
    pub fn parent(self) -> GraphyResult<AttributeValueParent<'a>> {
        let parent = self.0.source_node(EdgeWeightKindDiscriminants::Use)?;
        match AttributeValue::try_from(parent) {
            Ok(parent) => Ok(AttributeValueParent::AttributeValue(parent)),
            Err(_) => Ok(AttributeValueParent::SchemaVariant(SchemaVariant(self.0))),
        }
    }

    pub fn parent_prop(self) -> GraphyResult<Option<AttributeValue<'a>>> {
        Ok(self.0.source_node(EdgeWeightKindDiscriminants::Use)?.try_into().ok())
    }
}

impl<'a> TryFrom<GraphyNode<'a>> for AttributeValue<'a> {
    type Error = GraphyError;
    fn try_from(node: GraphyNode<'a>) -> Result<Self, Self::Error> {
        let result = Self(node);
        result.weight()?;
        Ok(result)
    }
}

