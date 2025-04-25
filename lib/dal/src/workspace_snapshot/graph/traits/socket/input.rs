use crate::{
    AttributeValueId,
    ComponentId,
    InputSocketId,
    SchemaVariantId,
    workspace_snapshot::{
        graph::WorkspaceSnapshotGraphResult,
        node_weight::InputSocketNodeWeight,
    },
};

pub trait InputSocketExt {
    fn get_input_socket(
        &self,
        input_socket_id: InputSocketId,
    ) -> WorkspaceSnapshotGraphResult<InputSocketNodeWeight>;

    fn list_input_sockets_for_schema_variant(
        &self,
        schema_variant_id: SchemaVariantId,
    ) -> WorkspaceSnapshotGraphResult<Vec<InputSocketNodeWeight>>;

    fn all_attribute_value_ids_everywhere_for_input_socket_id(
        &self,
        input_socket_id: InputSocketId,
    ) -> WorkspaceSnapshotGraphResult<Vec<AttributeValueId>>;

    fn component_attribute_value_id_for_input_socket_id(
        &self,
        input_socket_id: InputSocketId,
        component_id: ComponentId,
    ) -> WorkspaceSnapshotGraphResult<AttributeValueId>;

    fn input_socket_id_find_for_attribute_value_id(
        &self,
        attribute_value_id: AttributeValueId,
    ) -> WorkspaceSnapshotGraphResult<Option<InputSocketId>>;
}
