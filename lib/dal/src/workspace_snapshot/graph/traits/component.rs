use si_id::{
    AttributeValueId,
    ComponentId,
};

use crate::component::ComponentResult;

pub trait ComponentExt {
    fn root_attribute_value(&self, component_id: ComponentId) -> ComponentResult<AttributeValueId>;

    fn external_source_count(&self, component_id: ComponentId) -> ComponentResult<usize>;
}
