SELECT parent_attribute_value_id,
       attribute_value_object,
       prop_object,
       func_binding_return_value_object AS object,
       func_with_prototype_context
FROM attribute_value_list_payload_for_read_context_v1($1, $2, $3, $4) AS payload (
                                                                                  parent_attribute_value_id,
                                                                                  attribute_value_object,
                                                                                  prop_object,
                                                                                  func_binding_return_value_object,
                                                                                  func_with_prototype_context
    );
