use super::{
    Action, AnyCategory, Category, Component, DependentValueRoot, Func, GraphyCategoryElement,
    GraphyContentNode, GraphyError, GraphyNode, GraphyNodeRef, GraphyResult, Module, Schema,
    Secret,
};
use crate::{
    workspace_snapshot::{
        content_address::ContentAddressDiscriminants,
        node_weight::{ContentNodeWeight, NodeWeight},
    },
    EdgeWeightKindDiscriminants,
};
use si_events::ulid::Ulid;

///
/// The root node of the graph. There can be only one.
///
/// Everything must be reachable from here.
///
#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef, derive_more::Deref)]
pub struct Root<'a>(GraphyNodeRef<'a>);

impl<'a> GraphyNode<'a> for Root<'a> {
    type Id = Ulid;
    type Weight = ContentNodeWeight;
    fn as_node(node: impl Into<GraphyNodeRef<'a>> + Copy) -> Self {
        Self(node.into())
    }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        Self::content_weight_as(weight)
    }
}

impl<'a> GraphyContentNode<'a> for Root<'a> {
    fn content_kind() -> ContentAddressDiscriminants {
        ContentAddressDiscriminants::Root
    }
}

impl<'a> Root<'a> {
    //
    // Children
    //
    pub fn categories(self) -> impl Iterator<Item = AnyCategory<'a>> {
        self.targets(EdgeWeightKindDiscriminants::Use)
    }

    pub fn components(self) -> GraphyResult<Category<'a, Component<'a>>> {
        self.category()
    }
    pub fn schemas(self) -> GraphyResult<Category<'a, Schema<'a>>> {
        self.category()
    }
    pub fn funcs(self) -> GraphyResult<Category<'a, Func<'a>>> {
        self.category()
    }
    pub fn modules(self) -> GraphyResult<Category<'a, Module<'a>>> {
        self.category()
    }
    pub fn actions(self) -> GraphyResult<Category<'a, Action<'a>>> {
        self.category()
    }
    pub fn dependent_value_roots(self) -> GraphyResult<Category<'a, DependentValueRoot<'a>>> {
        self.category()
    }
    pub fn secrets(self) -> GraphyResult<Category<'a, Secret<'a>>> {
        self.category()
    }

    pub fn category<T: GraphyCategoryElement<'a>>(self) -> GraphyResult<Category<'a, T>> {
        for category in self.categories() {
            if let Ok(category) = Category::try_as_node(category) {
                return Ok(category);
            }
        }
        Err(GraphyError::CategoryNotFound(T::category_kind()))
    }
}
