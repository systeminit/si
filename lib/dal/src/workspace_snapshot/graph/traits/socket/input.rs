use anyhow::Result;
use si_events::ContentHash;

use crate::{
    workspace_snapshot::{graph::LineageId, node_weight::InputSocketNodeWeight},
    AttributeValueId, ComponentId, InputSocketId, SchemaVariantId, SocketArity,
};

pub trait InputSocketExt {
    fn new_input_socket(
        &mut self,
        schema_variant_id: SchemaVariantId,
        input_socket_id: InputSocketId,
        lineage_id: LineageId,
        arity: SocketArity,
        content_hash: ContentHash,
    ) -> Result<InputSocketNodeWeight>;

    fn get_input_socket(&self, input_socket_id: InputSocketId) -> Result<InputSocketNodeWeight>;

    fn list_input_sockets_for_schema_variant(
        &self,
        schema_variant_id: SchemaVariantId,
    ) -> Result<Vec<InputSocketNodeWeight>>;

    fn all_attribute_value_ids_everywhere_for_input_socket_id(
        &self,
        input_socket_id: InputSocketId,
    ) -> Result<Vec<AttributeValueId>>;

    fn component_attribute_value_id_for_input_socket_id(
        &self,
        input_socket_id: InputSocketId,
        component_id: ComponentId,
    ) -> Result<AttributeValueId>;

    fn input_socket_id_find_for_attribute_value_id(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> Result<Option<InputSocketId>>;
}
