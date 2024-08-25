use si_events::ulid::Ulid;
use crate::EdgeWeightKindDiscriminants;
use super::*;
use super::super::content_address::ContentAddressDiscriminants;

#[derive(Copy, Clone, derive_more::Into, derive_more::AsRef)]
pub struct Root<'a>(pub(super) GraphyNode<'a>);

impl<'a> GraphyNodeType<'a> for Root<'a> {
    type Id = Ulid;
    type Weight = ContentNodeWeight;
    fn node_kind() -> NodeWeightDiscriminants { NodeWeightDiscriminants::Content }
    fn construct(node: GraphyNode<'a>) -> Self { Self(node) }
    fn weight_as(weight: &NodeWeight) -> GraphyResult<&Self::Weight> {
        Self::content_weight_as(weight)
    }
}

impl<'a> GraphyContentNodeType<'a> for Root<'a> {
    fn content_kind() -> ContentAddressDiscriminants { ContentAddressDiscriminants::Root }
}

impl<'a> Root<'a> {
    //
    // Children
    //
    pub fn categories(self) -> impl Iterator<Item = AnyCategory<'a>> {
        self.0.target_nodes(EdgeWeightKindDiscriminants::Use).map(AnyCategory)
    }

    pub fn components(self) -> GraphyResult<impl Iterator<Item = Component<'a>>> {
        Ok(self.component_category()?.all())
    }
    pub fn schemas(self) -> GraphyResult<impl Iterator<Item = Schema<'a>>> {
        Ok(self.schema_category()?.all())
    }
    pub fn funcs(self) -> GraphyResult<impl Iterator<Item = Func<'a>>> {
        Ok(self.func_category()?.all())
    }
    pub fn modules(self) -> GraphyResult<impl Iterator<Item = Module<'a>>> {
        Ok(self.module_category()?.all())
    }
    pub fn actions(self) -> GraphyResult<impl Iterator<Item = Action<'a>>> {
        Ok(self.action_category()?.all())
    }
    pub fn dependent_value_roots(self) -> GraphyResult<impl Iterator<Item = DependentValueRoot<'a>>> {
        Ok(self.dependent_value_root_category()?.all())
    }
    pub fn secrets(self) -> GraphyResult<impl Iterator<Item = Secret<'a>>> {
        Ok(self.secret_category()?.all())
    }

    pub fn component_category(self) -> GraphyResult<Category<'a, Component<'a>>> {
        self.category()
    }
    pub fn schema_category(self) -> GraphyResult<Category<'a, Schema<'a>>> {
        self.category()
    }
    pub fn func_category(self) -> GraphyResult<Category<'a, Func<'a>>> {
        self.category()
    }
    pub fn module_category(self) -> GraphyResult<Category<'a, Module<'a>>> {
        self.category()
    }
    pub fn action_category(self) -> GraphyResult<Category<'a, Action<'a>>> {
        self.category()
    }
    pub fn dependent_value_root_category(self) -> GraphyResult<Category<'a, DependentValueRoot<'a>>> {
        self.category()
    }
    pub fn secret_category(self) -> GraphyResult<Category<'a, Secret<'a>>> {
        self.category()
    }

    pub fn category<T: GraphyCategoryNodeType<'a>>(self) -> GraphyResult<Category<'a, T>> {
        for category in self.categories() {
            if let Ok(category) = category.try_into() {
                return Ok(category);
            }
        }
        Err(GraphyError::CategoryNotFound(T::category_kind()))
    }
}
