use si_events::ulid::Ulid;
use super::*;
use super::super::content_address::ContentAddressDiscriminants;

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef)]
pub struct Module<'a>(pub(super) GraphyNode<'a>);

impl<'a> GraphyNodeType<'a> for Module<'a> {
    type Id = Ulid;
    type Weight = ContentNodeWeight;
    fn node_kind() -> NodeWeightDiscriminants { NodeWeightDiscriminants::Content }
    fn construct(node: GraphyNode<'a>) -> Self { Self(node) }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        Self::content_weight_as(weight)
    }
}
impl<'a> GraphyContentNodeType<'a> for Module<'a> {
    fn content_kind() -> ContentAddressDiscriminants { ContentAddressDiscriminants::Module }
}
impl<'a> GraphyCategoryNodeType<'a> for Module<'a> {
    fn category_kind() -> CategoryNodeKind { CategoryNodeKind::Module }
}

impl<'a> Module<'a> {
}

impl<'a> TryFrom<GraphyNode<'a>> for Module<'a> {
    type Error = GraphyError;
    fn try_from(node: GraphyNode<'a>) -> Result<Self, Self::Error> {
        let result = Self(node);
        result.weight()?;
        Ok(result)
    }
}
