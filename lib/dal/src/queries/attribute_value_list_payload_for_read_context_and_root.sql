SELECT
    parent_attribute_value_id,
    attribute_value_object,
    prop_object,
    func_binding_return_value_object AS object
FROM
    attribute_value_list_payload_for_read_context_and_root_v1($1, $2, $3, $4) AS payload (
        parent_attribute_value_id,
        attribute_value_object,
        prop_object,
        func_binding_return_value_object
    );
