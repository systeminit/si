use serde::{Deserialize, Serialize};

use std::default::Default;

use crate::attribute_resolver::UNSET_ID_VALUE;
use crate::{ComponentId, PropId, SchemaId, SchemaVariantId, SystemId};

// Prop and Component IDs might both be set, unset, or a combination of the two. In fact, by
// default, [`AttributeResolverContext`] is created with _all_ IDs as [`UNSET_ID_VALUE`].
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq, Eq)]
pub struct AttributeResolverContext {
    prop_id: PropId,
    component_id: ComponentId,
    schema_id: SchemaId,
    schema_variant_id: SchemaVariantId,
    system_id: SystemId,
}

impl Default for AttributeResolverContext {
    fn default() -> Self {
        Self::new()
    }
}

impl AttributeResolverContext {
    pub fn new() -> Self {
        AttributeResolverContext {
            prop_id: UNSET_ID_VALUE.into(),
            component_id: UNSET_ID_VALUE.into(),
            schema_id: UNSET_ID_VALUE.into(),
            schema_variant_id: UNSET_ID_VALUE.into(),
            system_id: UNSET_ID_VALUE.into(),
        }
    }

    pub fn prop_id(&self) -> PropId {
        self.prop_id
    }

    pub fn set_prop_id(&mut self, prop_id: PropId) {
        self.prop_id = prop_id;
    }

    pub fn component_id(&self) -> ComponentId {
        self.component_id
    }

    pub fn set_component_id(&mut self, component_id: ComponentId) {
        self.component_id = component_id;
    }

    pub fn schema_id(&self) -> SchemaId {
        self.schema_id
    }

    pub fn set_schema_id(&mut self, schema_id: SchemaId) {
        self.schema_id = schema_id;
    }

    pub fn schema_variant_id(&self) -> SchemaVariantId {
        self.schema_variant_id
    }

    pub fn set_schema_variant_id(&mut self, schema_variant_id: SchemaVariantId) {
        self.schema_variant_id = schema_variant_id;
    }

    pub fn system_id(&self) -> SystemId {
        self.system_id
    }

    pub fn set_system_id(&mut self, system_id: SystemId) {
        self.system_id = system_id;
    }
}