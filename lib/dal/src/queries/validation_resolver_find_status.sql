SELECT
    validation_resolvers.id,
    validation_resolvers.visibility_change_set_pk,
    validation_resolvers.visibility_deleted_at,
    attribute_values.id                            as attribute_value_id,
    row_to_json(validation_prototypes.*)           as validation_prototype_json,
    row_to_json(func_binding_return_values.*)      as object
FROM attribute_values_v1($1, $2) as attribute_values
LEFT JOIN validation_resolvers_v1($1, $2) as validation_resolvers
    ON validation_resolvers.attribute_value_id = attribute_values.id
        AND validation_resolvers.attribute_value_func_binding_return_value_id = attribute_values.func_binding_return_value_id
LEFT JOIN validation_prototypes_v1($1, $2) as validation_prototypes
        ON validation_prototypes.id = validation_resolvers.validation_prototype_id
LEFT JOIN func_bindings_v1($1, $2) as func_bindings
        ON func_bindings.id = validation_resolvers.validation_func_binding_id
LEFT JOIN func_binding_return_value_belongs_to_func_binding_v1($1, $2) as fbrvbtfb
        ON fbrvbtfb.belongs_to_id = func_bindings.id
LEFT JOIN func_binding_return_values_v1($1, $2) as func_binding_return_values
        ON func_binding_return_values.id = fbrvbtfb.object_id
WHERE
    attribute_values.attribute_context_prop_id IN (
        WITH RECURSIVE recursive_props AS (
            SELECT left_object_id AS prop_id
            FROM prop_many_to_many_schema_variants_v1($1, $2) AS prop_many_to_many_schema_variants
            WHERE right_object_id = $4
            UNION ALL
            SELECT pbp.object_id AS prop_id
            FROM prop_belongs_to_prop_v1($1, $2) AS pbp
            JOIN recursive_props ON pbp.belongs_to_id = recursive_props.prop_id
        )
        SELECT prop_id
        FROM recursive_props
    )
    AND exact_attribute_read_context_v1($3, attribute_values)
