SELECT DISTINCT ON (validation_resolvers.id) validation_resolvers.id,
                                             validation_resolvers.visibility_change_set_pk,

                                             attribute_values.id                       as attribute_value_id,
                                             row_to_json(validation_prototypes.*)      as validation_prototype_json,
                                             row_to_json(func_binding_return_values.*) as object
FROM attribute_values
         LEFT JOIN validation_resolvers
                   ON validation_resolvers.attribute_value_id = attribute_values.id
                       AND
                      validation_resolvers.func_binding_return_value_id = attribute_values.func_binding_return_value_id
                       AND in_tenancy_v1($1, validation_resolvers.tenancy_universal,
                                         validation_resolvers.tenancy_billing_account_ids,
                                         validation_resolvers.tenancy_organization_ids,
                                         validation_resolvers.tenancy_workspace_ids)
                       AND is_visible_v1($2, validation_resolvers.visibility_change_set_pk,
                                         validation_resolvers.visibility_deleted_at)
         LEFT JOIN validation_prototypes ON validation_prototypes.id = validation_resolvers.validation_prototype_id
         LEFT JOIN func_bindings ON func_bindings.id = validation_resolvers.func_binding_id
         LEFT JOIN func_binding_return_value_belongs_to_func_binding
                   ON func_binding_return_value_belongs_to_func_binding.belongs_to_id = func_bindings.id
         LEFT JOIN func_binding_return_values
                   ON func_binding_return_values.id = func_binding_return_value_belongs_to_func_binding.object_id
WHERE in_tenancy_v1($1, attribute_values.tenancy_universal, attribute_values.tenancy_billing_account_ids,
                    attribute_values.tenancy_organization_ids, attribute_values.tenancy_workspace_ids)
  AND is_visible_v1($2, attribute_values.visibility_change_set_pk, attribute_values.visibility_deleted_at)
  AND in_attribute_context_v1($3, attribute_values.attribute_context_prop_id,
                              attribute_values.attribute_context_internal_provider_id,
                              attribute_values.attribute_context_external_provider_id,
                              attribute_values.attribute_context_schema_id,
                              attribute_values.attribute_context_schema_variant_id,
                              attribute_values.attribute_context_component_id,
                              attribute_values.attribute_context_system_id)
  AND attribute_values.attribute_context_prop_id IN (WITH RECURSIVE recursive_props AS (SELECT left_object_id AS prop_id
                                                                                        FROM prop_many_to_many_schema_variants
                                                                                        WHERE right_object_id = $4
                                                                                        UNION ALL
                                                                                        SELECT pbp.object_id AS prop_id
                                                                                        FROM prop_belongs_to_prop AS pbp
                                                                                                 JOIN recursive_props ON pbp.belongs_to_id = recursive_props.prop_id)
                                                     SELECT prop_id
                                                     FROM recursive_props)
ORDER BY validation_resolvers.id,
         visibility_change_set_pk DESC;
