use si_id::{
    AttributePrototypeId,
    AttributeValueId,
};

use crate::attribute::value::AttributeValueResult;

pub trait AttributeValueExt {
    fn component_prototype_id(
        &self,
        id: AttributeValueId,
    ) -> AttributeValueResult<Option<AttributePrototypeId>>;
}
