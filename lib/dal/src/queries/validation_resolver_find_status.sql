SELECT DISTINCT ON (validation_resolvers.id) validation_resolvers.id,
                                             validation_resolvers.visibility_change_set_pk,
                                             validation_resolvers.visibility_deleted_at,
                                             attribute_values.id                       as attribute_value_id,
                                             row_to_json(validation_prototypes.*)      as validation_prototype_json,
                                             row_to_json(func_binding_return_values.*) as object

FROM attribute_values
         LEFT JOIN validation_resolvers
                   ON validation_resolvers.attribute_value_id = attribute_values.id
                       AND
                      validation_resolvers.func_binding_return_value_id = attribute_values.func_binding_return_value_id
                       AND in_tenancy_and_visible_v1($1, $2, validation_resolvers)
         LEFT JOIN validation_prototypes
                   ON validation_prototypes.id = validation_resolvers.validation_prototype_id
                       AND in_tenancy_and_visible_v1($1, $2, validation_prototypes)
         LEFT JOIN func_bindings
                   ON func_bindings.id = validation_resolvers.func_binding_id
                       AND in_tenancy_and_visible_v1($1, $2, func_bindings)
         LEFT JOIN func_binding_return_value_belongs_to_func_binding
                   ON func_binding_return_value_belongs_to_func_binding.belongs_to_id = func_bindings.id
                       AND in_tenancy_and_visible_v1($1, $2, func_binding_return_value_belongs_to_func_binding)
         LEFT JOIN func_binding_return_values
                   ON func_binding_return_values.id = func_binding_return_value_belongs_to_func_binding.object_id
                       AND in_tenancy_and_visible_v1($1, $2, func_binding_return_values)

WHERE in_tenancy_and_visible_v1($1, $2, attribute_values)
  AND attribute_values.attribute_context_prop_id IN (WITH RECURSIVE recursive_props AS
                                                                        (SELECT left_object_id AS prop_id
                                                                         FROM prop_many_to_many_schema_variants
                                                                         WHERE right_object_id = $4
                                                                         UNION ALL
                                                                         SELECT pbp.object_id AS prop_id
                                                                         FROM prop_belongs_to_prop AS pbp
                                                                                  JOIN recursive_props ON pbp.belongs_to_id = recursive_props.prop_id)
                                                     SELECT prop_id
                                                     FROM recursive_props)
  AND exact_attribute_read_context_v1($3, attribute_values.attribute_context_prop_id,
                                      attribute_values.attribute_context_internal_provider_id,
                                      attribute_values.attribute_context_external_provider_id,
                                      attribute_values.attribute_context_schema_id,
                                      attribute_values.attribute_context_schema_variant_id,
                                      attribute_values.attribute_context_component_id,
                                      attribute_values.attribute_context_system_id)

ORDER BY validation_resolvers.id,
         visibility_change_set_pk DESC,
         visibility_deleted_at DESC NULLS FIRST;
