use crate::{ComponentId, SchemaId, SchemaVariantId, SystemId};

/// Represents a context for a prototype that has the component_id as the max specificity.
/// Useful for using PrototypeContext objects generically.
/// Read only, but the mutation methods could be ported into it as well.
pub trait PrototypeContext {
    fn component_id(&self) -> ComponentId;
    fn schema_id(&self) -> SchemaId;
    fn schema_variant_id(&self) -> SchemaVariantId;
    fn system_id(&self) -> SystemId;
}

pub trait GetContext<T>
where
    T: PrototypeContext,
{
    fn context(&self) -> T;
}
