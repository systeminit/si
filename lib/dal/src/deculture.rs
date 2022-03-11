pub mod attribute;
pub mod component;
pub mod index_map;
pub mod input_socket;

pub use attribute::{
    context::{
        AttributeContext, AttributeContextBuilderError, AttributeContextError, AttributeReadContext,
    },
    prototype::{
        AttributePrototype, AttributePrototypeError, AttributePrototypeId, AttributePrototypeResult,
    },
    value::{
        AttributeValue, AttributeValueError, AttributeValueId, AttributeValuePayload,
        AttributeValueResult,
    },
};
pub use index_map::IndexMap;
