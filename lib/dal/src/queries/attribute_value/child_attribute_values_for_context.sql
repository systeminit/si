SELECT DISTINCT ON (av.attribute_context_prop_id, COALESCE(av.key, '')) row_to_json(av.*) AS object
FROM attribute_values_v1($1, $2) AS av
         INNER JOIN attribute_value_belongs_to_attribute_value_v1($1, $2) AS avbtav
                    ON avbtav.object_id = av.id
WHERE in_attribute_context_v1($4, av)
  AND avbtav.belongs_to_id = $3
ORDER BY attribute_context_prop_id,
         COALESCE(key, ''),
         attribute_context_internal_provider_id DESC,
         attribute_context_external_provider_id DESC,
         attribute_context_component_id DESC,
         av.tenancy_universal -- bools sort false first ascending.
